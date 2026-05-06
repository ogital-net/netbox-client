//! Sync device interfaces and IP addresses into NetBox via SNMP.
//!
//! This example walks the standard IF-MIB and IP-MIB tables to collect the
//! live state of a device's interfaces, then reconciles any drift found in
//! NetBox. It is protocol-agnostic — any device with an SNMP agent works, not
//! just Juniper.
//!
//! ## What it collects via SNMP
//!
//! | MIB table   | Data                                              |
//! |-------------|---------------------------------------------------|
//! | `ifTable`   | interface name (`ifDescr`), speed (`ifSpeed`), MAC address (`ifPhysAddress`), admin status (`ifAdminStatus`) |
//! | `ifXTable`  | canonical name (`ifName`), description (`ifAlias`), high-speed (`ifHighSpeed`) |
//! | `ipAddrTable` | IPv4 address, subnet mask, and owning interface index |
//!
//! ## What it reconciles in NetBox
//!
//! 1. **Interface fields** — `description`, `enabled`, and `speed` are patched
//!    when they differ from the device.
//! 2. **IP addresses** — each address found on the device is looked up in
//!    NetBox. If it is missing it is created and assigned to the matching
//!    interface. If it exists but is assigned to the wrong interface it is
//!    re-assigned.
//!
//! 3. **MAC addresses** — each interface MAC address found via SNMP is looked up
//!    in NetBox (`/api/dcim/mac-addresses/`). If the address is missing it is
//!    created and assigned to the matching interface. Existing entries that are
//!    already correctly assigned are left untouched.
//!
//! ## Configuration
//!
//! Set these variables in `.env` or the shell:
//!
//! | Variable             | Required | Description |
//! |----------------------|----------|-------------|
//! | `NETBOX_URL`         | ✓ | NetBox base URL |
//! | `NETBOX_TOKEN`       | ✓ | NetBox API token |
//! | `SNMP_HOST`          | ✓ | Device IP or hostname |
//! | `SNMP_PORT`          | | SNMP UDP port (default `161`) |
//! | `NETBOX_DEVICE_NAME` | ✓ | Name of the device in NetBox |
//!
//! **SNMPv2c (default):**
//!
//! | Variable         | Default   | Description |
//! |------------------|-----------|-------------|
//! | `SNMP_COMMUNITY` | `public`  | Community string |
//!
//! **SNMPv3 (set `SNMP_USER` to activate):**
//!
//! | Variable          | Default   | Description |
//! |-------------------|-----------|-------------|
//! | `SNMP_USER`       | —         | USM username; presence enables v3 |
//! | `SNMP_AUTH_PASS`  | —         | Auth passphrase (required with v3) |
//! | `SNMP_PRIV_PASS`  | —         | Privacy passphrase (required with v3) |
//! | `SNMP_AUTH_PROTO` | `sha256`  | `sha256`, `sha1`, or `md5` |
//! | `SNMP_PRIV_PROTO` | `aes128`  | `aes128` or `aes256` |
//!
//! ## Running
//!
//! ```text
//! cargo run --example snmp_sync
//! ```

use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::time::Duration;

use async_snmp::v3::{AuthProtocol, PrivProtocol};
use async_snmp::{oid, Auth, Client};
use futures_util::TryStreamExt as _;
use netbox_client::dcim::{InterfaceFilter, MACAddressFilter};
use netbox_client::ipam::IpAddressFilter;
use netbox_client::{
    InterfacePatchRequest, IpAddressPatchRequest, IpAddressRequest, MACAddressRequest, NetboxClient,
};

// ── IF-MIB column numbers (ifTable: 1.3.6.1.2.1.2.2.1) ───────────────────────
const IF_DESCR: u32 = 2;
const IF_SPEED: u32 = 5;
const IF_PHYS_ADDRESS: u32 = 6;
const IF_ADMIN_STATUS: u32 = 7;

// ── IF-MIB column numbers (ifXTable: 1.3.6.1.2.1.31.1.1.1) ──────────────────
const IF_NAME: u32 = 1;
const IF_HIGH_SPEED: u32 = 15;
const IF_ALIAS: u32 = 18;

// ── IP-MIB column numbers (ipAddrTable: 1.3.6.1.2.1.4.20.1) ─────────────────
const IP_AD_ENT_IF_INDEX: u32 = 2;
const IP_AD_ENT_NET_MASK: u32 = 3;

// ── Data structures ───────────────────────────────────────────────────────────

/// Interface data collected from IF-MIB.
#[derive(Debug, Default)]
struct IfSnmp {
    /// Interface description from `ifDescr` (old-style, used as fallback name).
    if_descr: String,
    /// Canonical name from `ifName` (ifXTable), falls back to `ifDescr`.
    if_name: Option<String>,
    /// Operator description from `ifAlias` (ifXTable).
    if_alias: Option<String>,
    /// Admin-up state: `true` when `ifAdminStatus == 1`.
    admin_up: bool,
    /// Speed in Kbps, derived from `ifHighSpeed` (Mbps) or `ifSpeed` (bps).
    speed_kbps: Option<i64>,
    /// MAC address formatted as `aa:bb:cc:dd:ee:ff`, if available.
    mac: Option<String>,
}

impl IfSnmp {
    /// Return the best available interface name: `ifName` if present, else `ifDescr`.
    fn name(&self) -> &str {
        self.if_name.as_deref().unwrap_or(&self.if_descr)
    }

    /// Return the best available description: `ifAlias` if present, else empty.
    fn description(&self) -> &str {
        self.if_alias.as_deref().unwrap_or("")
    }
}

// ── OID helpers ───────────────────────────────────────────────────────────────

/// Extract the numeric arcs from an OID's dot-separated string representation.
fn oid_arcs(oid: &async_snmp::Oid) -> Vec<u32> {
    oid.to_string()
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect()
}

// ── Value conversion helpers ──────────────────────────────────────────────────

/// Convert an `ifSpeed` (bps, Gauge32) or `ifHighSpeed` (Mbps, Gauge32) value
/// to Kbps for NetBox.
///
/// - `ifHighSpeed` is preferred when non-zero (it handles interfaces > ~4 Gbps
///    that overflow the 32-bit `ifSpeed`).
/// - `ifSpeed` falls back: divide by 1 000.
/// - Zero or missing speed is returned as `None`.
fn speed_kbps(if_speed_bps: u32, if_high_speed_mbps: u32) -> Option<i64> {
    if if_high_speed_mbps > 0 {
        Some(if_high_speed_mbps as i64 * 1_000)
    } else if if_speed_bps > 0 {
        Some(if_speed_bps as i64 / 1_000)
    } else {
        None
    }
}

/// Convert an IPv4 subnet mask to a CIDR prefix length.
fn mask_to_prefix_len(mask: Ipv4Addr) -> u8 {
    mask.octets().iter().map(|&b| b.count_ones() as u8).sum()
}

// ── SNMP collection ───────────────────────────────────────────────────────────

/// Walk IF-MIB `ifTable` (1.3.6.1.2.1.2.2) and populate the map with
/// `ifDescr`, `ifSpeed`, `ifPhysAddress`, and `ifAdminStatus`.
///
/// The table prefix has 8 arcs. Each returned OID has the form
/// `1.3.6.1.2.1.2.2.1.{column}.{ifIndex}`, so `arcs[9]` is the column and
/// `arcs[10]` is the index.
async fn walk_if_table(
    client: &async_snmp::UdpClient,
    map: &mut HashMap<u32, IfSnmp>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut walk = client.walk(oid!(1, 3, 6, 1, 2, 1, 2, 2))?;
    while let Some(vb) = walk.next().await {
        let vb = vb?;
        let arcs = oid_arcs(&vb.oid);
        if arcs.len() < 11 {
            continue;
        }
        let col = arcs[9];
        let idx = arcs[10];
        let entry = map.entry(idx).or_default();

        match col {
            IF_DESCR => {
                if let Some(s) = vb.value.as_str() {
                    entry.if_descr = s.trim().to_owned();
                }
            }
            IF_SPEED => {
                // Store raw bps value; will be reconciled with ifHighSpeed later.
                if let Some(bps) = vb.value.as_u32() {
                    // Temporarily encode bps into the speed_kbps slot as negative
                    // so we can distinguish "not set yet" from "already upgraded by
                    // ifHighSpeed". Use a side-channel via if_descr length instead:
                    // simpler — just update only if ifHighSpeed hasn't been set yet.
                    if entry.speed_kbps.is_none() {
                        entry.speed_kbps = speed_kbps(bps, 0);
                    }
                }
            }
            IF_PHYS_ADDRESS => {
                // Use DISPLAY-HINT "1x:" to format as "aa:bb:cc:dd:ee:ff".
                if let Some(mac) = vb.value.format_with_hint("1x:") {
                    if mac.len() == 17 {
                        entry.mac = Some(mac);
                    }
                }
            }
            IF_ADMIN_STATUS => {
                // 1 = up, 2 = down, 3 = testing
                entry.admin_up = vb.value.as_i32() == Some(1);
            }
            _ => {}
        }
    }
    Ok(())
}

/// Walk IF-MIB `ifXTable` (1.3.6.1.2.1.31.1.1) and populate `ifName`,
/// `ifAlias`, and `ifHighSpeed`.
///
/// The table prefix has 9 arcs. Each returned OID has the form
/// `1.3.6.1.2.1.31.1.1.1.{column}.{ifIndex}`, so `arcs[10]` is the column
/// and `arcs[11]` is the index.
async fn walk_if_x_table(
    client: &async_snmp::UdpClient,
    map: &mut HashMap<u32, IfSnmp>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut walk = client.walk(oid!(1, 3, 6, 1, 2, 1, 31, 1, 1))?;
    while let Some(vb) = walk.next().await {
        let vb = vb?;
        let arcs = oid_arcs(&vb.oid);
        if arcs.len() < 12 {
            continue;
        }
        let col = arcs[10];
        let idx = arcs[11];
        let entry = map.entry(idx).or_default();

        match col {
            IF_NAME => {
                if let Some(s) = vb.value.as_str() {
                    entry.if_name = Some(s.trim().to_owned());
                }
            }
            IF_HIGH_SPEED => {
                // ifHighSpeed is in Mbps; it supersedes the ifSpeed value.
                if let Some(mbps) = vb.value.as_u32() {
                    entry.speed_kbps = speed_kbps(0, mbps);
                }
            }
            IF_ALIAS => {
                if let Some(s) = vb.value.as_str() {
                    let alias = s.trim().to_owned();
                    if !alias.is_empty() {
                        entry.if_alias = Some(alias);
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// Walk IP-MIB `ipAddrTable` (1.3.6.1.2.1.4.20) and return a list of
/// `(address, prefix_length, ifIndex)` tuples for all unicast IPv4 addresses.
///
/// The table prefix has 8 arcs. Each returned OID has the form
/// `1.3.6.1.2.1.4.20.1.{column}.{a}.{b}.{c}.{d}`, so `arcs[9]` is the
/// column and `arcs[10..=13]` are the four IP octets.
async fn walk_ip_addr_table(
    client: &async_snmp::UdpClient,
) -> Result<Vec<(Ipv4Addr, u8, u32)>, Box<dyn std::error::Error>> {
    // ip → ifIndex
    let mut ip_to_ifindex: HashMap<Ipv4Addr, u32> = HashMap::new();
    // ip → netmask
    let mut ip_to_mask: HashMap<Ipv4Addr, Ipv4Addr> = HashMap::new();

    let mut walk = client.walk(oid!(1, 3, 6, 1, 2, 1, 4, 20))?;
    while let Some(vb) = walk.next().await {
        let vb = vb?;
        let arcs = oid_arcs(&vb.oid);
        if arcs.len() < 14 {
            continue;
        }
        let col = arcs[9];
        let ip = Ipv4Addr::new(
            arcs[10] as u8,
            arcs[11] as u8,
            arcs[12] as u8,
            arcs[13] as u8,
        );

        match col {
            IP_AD_ENT_IF_INDEX => {
                if let Some(idx) = vb.value.as_u32() {
                    ip_to_ifindex.insert(ip, idx);
                }
            }
            IP_AD_ENT_NET_MASK => {
                if let Some(mask) = vb.value.as_ip() {
                    ip_to_mask.insert(ip, mask);
                }
            }
            _ => {}
        }
    }

    let mut entries = Vec::new();
    for (ip, ifindex) in &ip_to_ifindex {
        // Skip the IPv4 loopback address; it is never synced to NetBox.
        if ip.is_loopback() {
            continue;
        }
        let prefix_len = ip_to_mask
            .get(ip)
            .map(|m| mask_to_prefix_len(*m))
            .unwrap_or(32);
        entries.push((*ip, prefix_len, *ifindex));
    }
    Ok(entries)
}

// ── Auth builder ──────────────────────────────────────────────────────────────

/// Build the SNMP auth configuration from environment variables.
///
/// Uses v3 (USM) when `SNMP_USER` is set; falls back to v2c otherwise.
fn snmp_auth_v2c() -> Auth {
    let community = std::env::var("SNMP_COMMUNITY").unwrap_or_else(|_| "public".into());
    Auth::v2c(community)
}

fn snmp_auth_v3(user: &str) -> Auth {
    let auth_pass = std::env::var("SNMP_AUTH_PASS").expect("SNMP_AUTH_PASS required for v3");
    let priv_pass = std::env::var("SNMP_PRIV_PASS").expect("SNMP_PRIV_PASS required for v3");

    let auth_proto = match std::env::var("SNMP_AUTH_PROTO")
        .unwrap_or_else(|_| "sha256".into())
        .to_ascii_lowercase()
        .as_str()
    {
        "sha1" => AuthProtocol::Sha1,
        "md5" => AuthProtocol::Md5,
        _ => AuthProtocol::Sha256,
    };

    let priv_proto = match std::env::var("SNMP_PRIV_PROTO")
        .unwrap_or_else(|_| "aes128".into())
        .to_ascii_lowercase()
        .as_str()
    {
        "aes256" => PrivProtocol::Aes256,
        _ => PrivProtocol::Aes128,
    };

    Auth::usm(user)
        .auth(auth_proto, auth_pass.as_str())
        .privacy(priv_proto, priv_pass.as_str())
        .into()
}

// ── Main ──────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    env_logger::init();

    // ── Configuration ──────────────────────────────────────────────────────────
    let netbox_url = std::env::var("NETBOX_URL").expect("NETBOX_URL must be set");
    let netbox_token = std::env::var("NETBOX_TOKEN").expect("NETBOX_TOKEN must be set");
    let snmp_host = std::env::var("SNMP_HOST").expect("SNMP_HOST must be set");
    let snmp_port: u16 = std::env::var("SNMP_PORT")
        .unwrap_or_else(|_| "161".into())
        .parse()
        .expect("SNMP_PORT must be a valid port number");
    let device_name = std::env::var("NETBOX_DEVICE_NAME").expect("NETBOX_DEVICE_NAME must be set");

    // ── Step 1: Build SNMP client ──────────────────────────────────────────────
    let target = (snmp_host.as_str(), snmp_port);
    let snmp_user = std::env::var("SNMP_USER").ok();

    let snmp_client: async_snmp::UdpClient = if let Some(ref user) = snmp_user {
        println!("Connecting to {snmp_host}:{snmp_port} with SNMPv3 (user: {user}) …");
        Client::builder(target, snmp_auth_v3(user))
            .timeout(Duration::from_secs(10))
            .connect()
            .await?
    } else {
        println!("Connecting to {snmp_host}:{snmp_port} with SNMPv2c …");
        Client::builder(target, snmp_auth_v2c())
            .timeout(Duration::from_secs(10))
            .connect()
            .await?
    };

    // ── Step 2: Walk the IF-MIB and IP-MIB tables ─────────────────────────────
    println!("Walking IF-MIB ifTable …");
    let mut if_map: HashMap<u32, IfSnmp> = HashMap::new();
    walk_if_table(&snmp_client, &mut if_map).await?;
    println!("  {} ifTable entries", if_map.len());

    println!("Walking IF-MIB ifXTable …");
    walk_if_x_table(&snmp_client, &mut if_map).await?;

    println!("Walking IP-MIB ipAddrTable …");
    let ip_entries = walk_ip_addr_table(&snmp_client).await?;
    println!("  {} IPv4 addresses found", ip_entries.len());
    println!();

    // Build (ifIndex → Vec<(Ipv4Addr, prefix_len)>) for the IP reconciliation loop.
    let mut ips_by_ifindex: HashMap<u32, Vec<(Ipv4Addr, u8)>> = HashMap::new();
    for (addr, prefix_len, ifindex) in &ip_entries {
        ips_by_ifindex
            .entry(*ifindex)
            .or_default()
            .push((*addr, *prefix_len));
    }

    println!("Interfaces found on device:");
    println!("{:-<60}", "");
    for (idx, iface) in &if_map {
        let mac_str = iface.mac.as_deref().unwrap_or("(no MAC)");
        let speed_str = iface
            .speed_kbps
            .map(|k| format!("{} Kbps", k))
            .unwrap_or_else(|| "unknown".into());
        let ips = ips_by_ifindex
            .get(idx)
            .map(|v| {
                v.iter()
                    .map(|(a, p)| format!("{a}/{p}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();
        println!(
            "  {:>4}  {:<28}  {}  {}  [{}]  {}",
            idx,
            iface.name(),
            if iface.admin_up { "up  " } else { "down" },
            speed_str,
            mac_str,
            ips,
        );
    }
    println!();

    // ── Step 3: Fetch NetBox interfaces ───────────────────────────────────────
    println!("Fetching NetBox interfaces for '{device_name}' …");
    let nb = NetboxClient::new(&netbox_url, &netbox_token)?;
    let filter = InterfaceFilter {
        device: vec![device_name.clone()],
        ..Default::default()
    };
    let nb_ifaces: Vec<_> = nb.interfaces(&filter).try_collect().await?;
    println!("  {} interfaces in NetBox", nb_ifaces.len());
    println!();

    // Build ifIndex-keyed lookup by matching ifName to NetBox name.
    // NetBox doesn't expose ifIndex, so we match by interface name.
    let nb_by_name: HashMap<&str, &netbox_client::Interface> =
        nb_ifaces.iter().map(|i| (i.name.as_str(), i)).collect();

    // Build a reverse map: NetBox interface name → ifIndex (for IP reconciliation).
    let mut ifindex_by_name: HashMap<&str, u32> = HashMap::new();
    for (idx, iface) in &if_map {
        ifindex_by_name.insert(iface.name(), *idx);
    }

    // ── Step 4: Reconcile interfaces ──────────────────────────────────────────
    println!("Reconciling interfaces …");
    println!("{:-<60}", "");
    let mut n_iface_updated = 0usize;
    let mut n_iface_unchanged = 0usize;
    let mut n_iface_missing = 0usize;

    for iface in if_map.values() {
        let Some(&nb_if) = nb_by_name.get(iface.name()) else {
            println!("  [SKIP]  {:<28} not in NetBox", iface.name());
            n_iface_missing += 1;
            continue;
        };

        let want_desc = iface.description();
        let desc_drift = nb_if.description != want_desc;
        let enabled_drift = nb_if.enabled != iface.admin_up;
        let speed_drift = nb_if.speed != iface.speed_kbps;

        if !desc_drift && !enabled_drift && !speed_drift {
            n_iface_unchanged += 1;
            continue;
        }

        let patch = InterfacePatchRequest {
            description: desc_drift.then(|| want_desc.to_owned()),
            enabled: enabled_drift.then_some(iface.admin_up),
            speed: if speed_drift { iface.speed_kbps } else { None },
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
            iface.name(),
            changed.join(", ")
        );
        n_iface_updated += 1;
    }

    println!();

    // ── Step 5: Reconcile IP addresses ────────────────────────────────────────
    println!("Reconciling IP addresses …");
    println!("{:-<60}", "");
    let mut n_ip_created = 0usize;
    let mut n_ip_reassigned = 0usize;
    let mut n_ip_ok = 0usize;
    let mut n_ip_no_iface = 0usize;

    for (addr, prefix_len, ifindex) in &ip_entries {
        // Determine the NetBox interface this IP lives on.
        let snmp_iface = match if_map.get(ifindex) {
            Some(i) => i,
            None => {
                println!("  [SKIP]  {addr}/{prefix_len}  unknown ifIndex {ifindex}");
                n_ip_no_iface += 1;
                continue;
            }
        };

        let Some(&nb_if) = nb_by_name.get(snmp_iface.name()) else {
            println!(
                "  [SKIP]  {addr}/{prefix_len}  interface '{}' not in NetBox",
                snmp_iface.name()
            );
            n_ip_no_iface += 1;
            continue;
        };

        let address_cidr = format!("{addr}/{prefix_len}");

        // Look up this address in NetBox (exact match).
        let existing = nb
            .ip_addresses(&IpAddressFilter {
                address: vec![address_cidr.clone()],
                ..Default::default()
            })
            .try_collect::<Vec<_>>()
            .await?;

        if existing.is_empty() {
            // Not in NetBox — create and assign to the interface.
            nb.ip_address_create(&IpAddressRequest {
                address: address_cidr.clone(),
                assigned_object_type: Some("dcim.interface".into()),
                assigned_object_id: Some(nb_if.id),
                status: Some("active".into()),
                ..Default::default()
            })
            .await?;
            println!("  [CREATE] {address_cidr:<30} → {}", snmp_iface.name());
            n_ip_created += 1;
        } else {
            let ip_obj = &existing[0];
            let already_assigned = ip_obj.assigned_object_type.as_deref() == Some("dcim.interface")
                && ip_obj.assigned_object_id == Some(nb_if.id);

            if already_assigned {
                n_ip_ok += 1;
            } else {
                // Re-assign to the correct interface.
                nb.ip_address_patch(
                    ip_obj.id,
                    &IpAddressPatchRequest {
                        assigned_object_type: Some("dcim.interface".into()),
                        assigned_object_id: Some(nb_if.id),
                        ..Default::default()
                    },
                )
                .await?;
                println!(
                    "  [ASSIGN] {address_cidr:<30} reassigned → {}",
                    snmp_iface.name()
                );
                n_ip_reassigned += 1;
            }
        }
    }

    // ── Step 6: Reconcile MAC addresses ───────────────────────────────────────
    println!("Reconciling MAC addresses …");
    println!("{:-<60}", "");
    let mut n_mac_created = 0usize;
    let mut n_mac_ok = 0usize;
    let mut n_mac_no_iface = 0usize;

    for iface in if_map.values() {
        let Some(mac_str) = &iface.mac else {
            continue;
        };

        let Some(&nb_if) = nb_by_name.get(iface.name()) else {
            n_mac_no_iface += 1;
            continue;
        };

        // Check if this exact MAC is already in NetBox and assigned to this interface.
        let existing = nb
            .mac_addresses(&MACAddressFilter {
                mac_address: vec![mac_str.clone()],
                interface_id: vec![nb_if.id],
                ..Default::default()
            })
            .try_collect::<Vec<_>>()
            .await?;

        if existing.is_empty() {
            nb.mac_address_create(&MACAddressRequest {
                mac_address: mac_str.clone(),
                assigned_object_type: Some("dcim.interface".into()),
                assigned_object_id: Some(nb_if.id),
                ..Default::default()
            })
            .await?;
            println!("  [CREATE] {mac_str:<20} → {}", iface.name());
            n_mac_created += 1;
        } else {
            n_mac_ok += 1;
        }
    }

    // ── Summary ───────────────────────────────────────────────────────────────
    println!("{:-<60}", "");
    println!("Interfaces — Updated: {n_iface_updated}  Unchanged: {n_iface_unchanged}  Not in NetBox: {n_iface_missing}");
    println!("IPs        — Created: {n_ip_created}  Reassigned: {n_ip_reassigned}  OK: {n_ip_ok}  No interface: {n_ip_no_iface}");
    println!(
        "MACs       — Created: {n_mac_created}  OK: {n_mac_ok}  No interface: {n_mac_no_iface}"
    );

    Ok(())
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── speed_kbps ────────────────────────────────────────────────────────────

    #[test]
    fn speed_high_speed_takes_precedence() {
        // ifHighSpeed 1000 Mbps → 1 000 000 Kbps (1 Gbps)
        assert_eq!(speed_kbps(0, 1_000), Some(1_000_000));
        // ifHighSpeed wins even when ifSpeed is also set
        assert_eq!(speed_kbps(100_000_000, 1_000), Some(1_000_000));
    }

    #[test]
    fn speed_falls_back_to_if_speed_bps() {
        // ifHighSpeed not set, ifSpeed 100 000 000 bps → 100 000 Kbps (100 Mbps)
        assert_eq!(speed_kbps(100_000_000, 0), Some(100_000));
        // 1 000 000 000 bps (1 Gbps) → 1 000 000 Kbps
        assert_eq!(speed_kbps(1_000_000_000, 0), Some(1_000_000));
    }

    #[test]
    fn speed_zero_returns_none() {
        assert_eq!(speed_kbps(0, 0), None);
    }

    #[test]
    fn speed_high_speed_100g() {
        // 100 Gbps = 100 000 Mbps → 100 000 000 Kbps
        assert_eq!(speed_kbps(0, 100_000), Some(100_000_000));
    }

    // ── mask_to_prefix_len ────────────────────────────────────────────────────

    #[test]
    fn mask_slash_24() {
        assert_eq!(mask_to_prefix_len(Ipv4Addr::new(255, 255, 255, 0)), 24);
    }

    #[test]
    fn mask_slash_32() {
        assert_eq!(mask_to_prefix_len(Ipv4Addr::new(255, 255, 255, 255)), 32);
    }

    #[test]
    fn mask_slash_0() {
        assert_eq!(mask_to_prefix_len(Ipv4Addr::new(0, 0, 0, 0)), 0);
    }

    #[test]
    fn mask_slash_30() {
        assert_eq!(mask_to_prefix_len(Ipv4Addr::new(255, 255, 255, 252)), 30);
    }

    #[test]
    fn mask_slash_16() {
        assert_eq!(mask_to_prefix_len(Ipv4Addr::new(255, 255, 0, 0)), 16);
    }

    // ── IfSnmp helper methods ─────────────────────────────────────────────────

    #[test]
    fn if_snmp_name_prefers_if_name() {
        let iface = IfSnmp {
            if_descr: "GigabitEthernet0".into(),
            if_name: Some("Gi0".into()),
            ..Default::default()
        };
        assert_eq!(iface.name(), "Gi0");
    }

    #[test]
    fn if_snmp_name_falls_back_to_if_descr() {
        let iface = IfSnmp {
            if_descr: "GigabitEthernet0".into(),
            if_name: None,
            ..Default::default()
        };
        assert_eq!(iface.name(), "GigabitEthernet0");
    }

    #[test]
    fn if_snmp_description_uses_alias() {
        let iface = IfSnmp {
            if_alias: Some("uplink to core".into()),
            ..Default::default()
        };
        assert_eq!(iface.description(), "uplink to core");
    }

    #[test]
    fn if_snmp_description_empty_when_no_alias() {
        let iface = IfSnmp::default();
        assert_eq!(iface.description(), "");
    }
}
