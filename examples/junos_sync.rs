//! Sync Juniper device interfaces into NetBox.
//!
//! This example demonstrates a common network operations workflow: treating a
//! Junos device as the authoritative source of truth for physical interface
//! state and reconciling any drift found in NetBox.
//!
//! ## What it does
//!
//! 1. Connects to a Junos device via NETCONF using [`rustez`].
//! 2. Reads device facts (hostname, model, Junos version).
//! 3. Fetches all physical interfaces via the `get-interface-information` RPC.
//! 4. Connects to NetBox and retrieves the matching device's interfaces.
//! 5. For each physical interface on the device, patches the NetBox record
//!    when any of the following fields have drifted from the live state:
//!    - `description` — the interface description configured on the device
//!    - `enabled`     — whether admin-status is `up`
//!    - `speed`       — negotiated/configured speed in Kbps
//!
//! Interfaces found on the device but absent from NetBox are logged and
//! skipped. Creating missing records is left to the operator.
//!
//! ## Configuration
//!
//! Set these variables in `.env` (or the shell environment) before running:
//!
//! | Variable             | Required | Description |
//! |----------------------|----------|-------------|
//! | `NETBOX_URL`         | ✓ | NetBox base URL, e.g. `https://netbox.example.com` |
//! | `NETBOX_TOKEN`       | ✓ | NetBox API token |
//! | `JUNOS_HOST`         | ✓ | Device IP address or DNS hostname |
//! | `JUNOS_USER`         | ✓ | SSH / NETCONF username |
//! | `JUNOS_PASS`         | ✓ | SSH / NETCONF password |
//! | `NETBOX_DEVICE_NAME` |   | Device name in NetBox; defaults to the Junos `hostname` fact |
//!
//! > **Tip:** The device name in NetBox must match the Junos `hostname` system
//! > setting, or override it explicitly with `NETBOX_DEVICE_NAME`.
//!
//! ## Running
//!
//! ```text
//! cargo run --example junos_sync
//! ```

use std::collections::HashMap;

use futures_util::TryStreamExt as _;
use netbox_client::{dcim::InterfaceFilter, InterfacePatchRequest, NetboxClient};
use rustez::Device;

// ── Junos interface snapshot ──────────────────────────────────────────────────

/// Physical interface data extracted from the `get-interface-information` RPC.
#[derive(Debug)]
struct JunosInterface {
    /// Interface name as reported by Junos (e.g. `ge-0/0/0`, `et-0/0/0`, `lo0`).
    name: String,
    /// Interface description configured on the device, if any.
    description: Option<String>,
    /// `true` when `<admin-status>` is `up`.
    admin_up: bool,
    /// Interface speed in Kbps, or `None` for interfaces that report
    /// `"Unlimited"` (loopback, aggregated bundles, internal) or an
    /// unrecognised format.
    speed_kbps: Option<i64>,
}

// ── XML parsing ───────────────────────────────────────────────────────────────

/// Parse physical interfaces from the XML body returned by the
/// `get-interface-information` NETCONF RPC.
///
/// The input is the inner XML of the `<rpc-reply>` — i.e. the
/// `<interface-information xmlns="...">…</interface-information>` element that
/// rustez returns from [`RpcExecutor::call`].
///
/// Each `<physical-interface>` block is mapped to a [`JunosInterface`].
/// Logical sub-interfaces and all other nested structures are skipped.
fn parse_interface_xml(xml: &str) -> Vec<JunosInterface> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut interfaces: Vec<JunosInterface> = Vec::new();

    // Depth counter inside the current <physical-interface> block:
    //   0  – outside any <physical-interface>
    //   1  – directly inside <physical-interface>, between children
    //   2  – inside a direct child element (the level where we read text)
    //   3+ – deeper nesting (e.g. inside <logical-interface>)
    let mut depth: u32 = 0;

    let mut current: Option<JunosInterface> = None;

    // Which direct-child leaf tag's text we are currently collecting.
    // `None` means we are either outside a physical-interface or inside a
    // nested sub-element we don't care about.
    let mut reading_tag: Option<String> = None;

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let tag = local_name(e.local_name().as_ref());

                if depth == 0 {
                    if tag == "physical-interface" {
                        depth = 1;
                        current = Some(JunosInterface {
                            name: String::new(),
                            description: None,
                            admin_up: true,
                            speed_kbps: None,
                        });
                        reading_tag = None;
                    }
                } else {
                    // depth >= 1: entering a child/descendant of <physical-interface>.
                    if depth == 1
                        && matches!(
                            tag.as_str(),
                            "name" | "description" | "admin-status" | "speed"
                        )
                    {
                        // This is a direct leaf child we care about.
                        reading_tag = Some(tag);
                    } else {
                        // Entering a nested element (e.g. <logical-interface>,
                        // <if-config-flags>, …). Stop capturing leaf text until
                        // we return to depth 1.
                        reading_tag = None;
                    }
                    depth += 1;
                }
            }

            Ok(Event::Text(ref e)) => {
                // depth == 2: we are inside a direct-child leaf of
                // <physical-interface> — the level where text lives.
                if depth == 2 {
                    if let Some(ref tag) = reading_tag {
                        let text = e.unescape().unwrap_or_default().trim().to_owned();
                        if let Some(ref mut iface) = current {
                            match tag.as_str() {
                                "name" => iface.name = text,
                                "description" if !text.is_empty() => {
                                    iface.description = Some(text);
                                }
                                "admin-status" => iface.admin_up = text == "up",
                                "speed" => iface.speed_kbps = junos_speed_kbps(&text),
                                _ => {}
                            }
                        }
                    }
                }
            }

            Ok(Event::End(_)) => {
                if depth > 0 {
                    depth -= 1;
                    if depth == 0 {
                        // Just closed </physical-interface>; commit the record.
                        if let Some(iface) = current.take() {
                            if !iface.name.is_empty() {
                                interfaces.push(iface);
                            }
                        }
                    } else if depth == 1 {
                        // Just closed a direct child element.
                        reading_tag = None;
                    }
                }
            }

            Ok(Event::Eof) => break,
            Err(e) => {
                eprintln!("XML parse error at byte {}: {e}", reader.buffer_position());
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    interfaces
}

/// Extract the local (un-prefixed) element name from a raw byte slice.
///
/// quick-xml's `local_name()` already strips any namespace prefix, so this
/// is just a UTF-8 conversion with a fallback for malformed bytes.
#[inline]
fn local_name(bytes: &[u8]) -> String {
    std::str::from_utf8(bytes).unwrap_or("").to_owned()
}

/// Convert a Junos speed string to Kbps.
///
/// Junos reports speeds as strings such as `"1000mbps"`, `"10Gbps"`, or
/// `"Unlimited"` (loopback / aggregated interfaces).
/// Returns `None` for `"Unlimited"`, `"0"`, and any unrecognised format.
fn junos_speed_kbps(s: &str) -> Option<i64> {
    let lower = s.trim().to_ascii_lowercase();
    if let Some(n_str) = lower.strip_suffix("gbps") {
        let n: f64 = n_str.trim().parse().ok()?;
        Some((n * 1_000_000.0) as i64)
    } else if let Some(n_str) = lower.strip_suffix("mbps") {
        let n: f64 = n_str.trim().parse().ok()?;
        Some((n * 1_000.0) as i64)
    } else if let Some(n_str) = lower.strip_suffix("kbps") {
        let n: f64 = n_str.trim().parse().ok()?;
        Some(n as i64)
    } else {
        // "Unlimited", "0", or unknown — no speed data to sync.
        None
    }
}

// ── Main ──────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    env_logger::init();

    // ── Load configuration ─────────────────────────────────────────────────────
    let netbox_url = std::env::var("NETBOX_URL").expect("NETBOX_URL must be set");
    let netbox_token = std::env::var("NETBOX_TOKEN").expect("NETBOX_TOKEN must be set");
    let junos_host = std::env::var("JUNOS_HOST").expect("JUNOS_HOST must be set");
    let junos_user = std::env::var("JUNOS_USER").expect("JUNOS_USER must be set");
    let junos_pass = std::env::var("JUNOS_PASS").expect("JUNOS_PASS must be set");

    // ── Step 1: Connect to the Juniper device via NETCONF ─────────────────────
    println!("Connecting to {junos_host} …");
    let mut dev = Device::connect(&junos_host)
        .username(&junos_user)
        .password(&junos_pass)
        .open()
        .await?;

    // ── Step 2: Gather device facts ───────────────────────────────────────────
    // facts() returns &Facts (borrows dev), so clone what we need before
    // releasing the borrow for the subsequent rpc() call.
    let (hostname, model, version) = {
        let f = dev.facts().await?;
        (f.hostname.clone(), f.model.clone(), f.version.clone())
    };
    println!("  hostname : {hostname}");
    println!("  model    : {model}");
    println!("  Junos    : {version}");
    println!();

    // Allow the operator to override the NetBox device name.  By default it
    // falls back to the Junos hostname so the two stay in sync automatically.
    let device_name = std::env::var("NETBOX_DEVICE_NAME").unwrap_or(hostname);

    // ── Step 3: Fetch all physical interface state via NETCONF RPC ────────────
    // Underscores in the RPC name are automatically converted to hyphens by
    // rustez, so "get_interface_information" → <get-interface-information/>.
    println!("Fetching interface information from device …");
    let iface_xml = dev.rpc()?.call("get_interface_information", &[]).await?;
    let junos_ifaces = parse_interface_xml(&iface_xml);
    println!("  {} physical interfaces found", junos_ifaces.len());
    println!();

    dev.close().await?;
    println!("NETCONF session closed.");
    println!();

    // ── Step 4: Connect to NetBox ─────────────────────────────────────────────
    let nb = NetboxClient::new(&netbox_url, &netbox_token)?;

    // ── Step 5: Fetch all NetBox interfaces for this device ───────────────────
    println!("Fetching NetBox interfaces for device '{device_name}' …");
    let filter = InterfaceFilter {
        device: vec![device_name.clone()],
        ..Default::default()
    };
    // interfaces() auto-paginates so this collects every page.
    let nb_ifaces: Vec<_> = nb.interfaces(&filter).try_collect().await?;
    println!("  {} interfaces found in NetBox", nb_ifaces.len());
    println!();

    // Index by interface name for O(1) look-up during reconciliation.
    let nb_by_name: HashMap<&str, &netbox_client::Interface> =
        nb_ifaces.iter().map(|i| (i.name.as_str(), i)).collect();

    // ── Step 6: Reconcile — device state wins ────────────────────────────────
    println!("Reconciling …");
    println!("{:-<60}", "");

    let mut n_updated = 0usize;
    let mut n_unchanged = 0usize;
    let mut n_missing = 0usize;

    for junos_if in &junos_ifaces {
        let Some(&nb_if) = nb_by_name.get(junos_if.name.as_str()) else {
            println!("  [SKIP]  {:<28} not found in NetBox", junos_if.name);
            n_missing += 1;
            continue;
        };

        // Determine which fields have drifted from the device truth.
        //
        // NetBox stores description as an empty string when unset; Junos
        // omits the <description> element entirely, which we map to "".
        let want_desc = junos_if.description.as_deref().unwrap_or("");
        let desc_drift = nb_if.description != want_desc;
        let enabled_drift = nb_if.enabled != junos_if.admin_up;
        let speed_drift = nb_if.speed != junos_if.speed_kbps;

        if !desc_drift && !enabled_drift && !speed_drift {
            n_unchanged += 1;
            continue;
        }

        // Build a minimal PATCH — only include fields that actually changed.
        // Fields left as `None` are omitted from the JSON body by
        // `#[serde(skip_serializing_if = "Option::is_none")]`.
        let patch = InterfacePatchRequest {
            description: desc_drift.then(|| want_desc.to_owned()),
            enabled: enabled_drift.then_some(junos_if.admin_up),
            speed: if speed_drift {
                junos_if.speed_kbps
            } else {
                None
            },
            ..Default::default()
        };

        nb.interface_patch(nb_if.id, &patch).await?;

        let changed: Vec<&str> = [
            desc_drift.then_some("description"),
            enabled_drift.then_some("enabled"),
            speed_drift.then_some("speed"),
        ]
        .iter()
        .flatten()
        .copied()
        .collect();

        println!(
            "  [SYNC]  {:<28} updated: {}",
            junos_if.name,
            changed.join(", ")
        );
        n_updated += 1;
    }

    println!("{:-<60}", "");
    println!("Done.  Updated: {n_updated}  Unchanged: {n_unchanged}  Not in NetBox: {n_missing}");

    Ok(())
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── junos_speed_kbps ──────────────────────────────────────────────────────

    #[test]
    fn speed_gigabit() {
        assert_eq!(junos_speed_kbps("1Gbps"), Some(1_000_000));
        assert_eq!(junos_speed_kbps("10Gbps"), Some(10_000_000));
        assert_eq!(junos_speed_kbps("40Gbps"), Some(40_000_000));
        assert_eq!(junos_speed_kbps("100Gbps"), Some(100_000_000));
    }

    #[test]
    fn speed_megabit() {
        assert_eq!(junos_speed_kbps("1000mbps"), Some(1_000_000));
        assert_eq!(junos_speed_kbps("100mbps"), Some(100_000));
        assert_eq!(junos_speed_kbps("10mbps"), Some(10_000));
    }

    #[test]
    fn speed_kilobit() {
        assert_eq!(junos_speed_kbps("56kbps"), Some(56));
    }

    #[test]
    fn speed_case_insensitive() {
        assert_eq!(junos_speed_kbps("1GBPS"), Some(1_000_000));
        assert_eq!(junos_speed_kbps("100MBPS"), Some(100_000));
    }

    #[test]
    fn speed_unlimited_and_unknown() {
        assert_eq!(junos_speed_kbps("Unlimited"), None);
        assert_eq!(junos_speed_kbps("0"), None);
        assert_eq!(junos_speed_kbps(""), None);
    }

    // ── parse_interface_xml ───────────────────────────────────────────────────

    #[test]
    fn parse_basic_interfaces() {
        // Minimal <interface-information> payload similar to what Junos returns
        // for `get-interface-information` on a two-interface device.
        let xml = r#"
<interface-information xmlns="http://xml.juniper.net/junos/24.4R1/junos-interface">
  <physical-interface>
    <name>ge-0/0/0</name>
    <admin-status>up</admin-status>
    <oper-status>up</oper-status>
    <description>uplink to core-sw</description>
    <speed>1000mbps</speed>
    <logical-interface>
      <name>ge-0/0/0.0</name>
    </logical-interface>
  </physical-interface>
  <physical-interface>
    <name>ge-0/0/1</name>
    <admin-status>down</admin-status>
    <oper-status>down</oper-status>
    <speed>1000mbps</speed>
  </physical-interface>
  <physical-interface>
    <name>lo0</name>
    <admin-status>up</admin-status>
    <oper-status>up</oper-status>
    <speed>Unlimited</speed>
  </physical-interface>
</interface-information>
"#;

        let ifaces = parse_interface_xml(xml);
        assert_eq!(ifaces.len(), 3);

        let ge000 = &ifaces[0];
        assert_eq!(ge000.name, "ge-0/0/0");
        assert_eq!(ge000.description.as_deref(), Some("uplink to core-sw"));
        assert!(ge000.admin_up);
        assert_eq!(ge000.speed_kbps, Some(1_000_000));

        let ge001 = &ifaces[1];
        assert_eq!(ge001.name, "ge-0/0/1");
        assert_eq!(ge001.description, None);
        assert!(!ge001.admin_up);
        assert_eq!(ge001.speed_kbps, Some(1_000_000));

        let lo0 = &ifaces[2];
        assert_eq!(lo0.name, "lo0");
        assert!(lo0.admin_up);
        assert_eq!(lo0.speed_kbps, None); // "Unlimited" → None
    }

    #[test]
    fn logical_interface_name_not_captured() {
        // The <name> inside <logical-interface> must NOT overwrite the physical
        // interface name.
        let xml = r#"
<interface-information>
  <physical-interface>
    <name>ge-0/0/0</name>
    <admin-status>up</admin-status>
    <speed>1000mbps</speed>
    <logical-interface>
      <name>ge-0/0/0.100</name>
    </logical-interface>
  </physical-interface>
</interface-information>
"#;
        let ifaces = parse_interface_xml(xml);
        assert_eq!(ifaces.len(), 1);
        assert_eq!(ifaces[0].name, "ge-0/0/0");
    }

    #[test]
    fn empty_xml_returns_empty_vec() {
        assert!(parse_interface_xml("").is_empty());
        assert!(parse_interface_xml("<interface-information/>").is_empty());
    }
}
