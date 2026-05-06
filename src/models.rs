// Public request/response types for the NetBox REST API.
//
// Add model structs here as new endpoints are implemented.
// Re-exported at the crate root via `pub use models::*`.

// ── Shared types ────────────────────────────────────────────────────────────

/// Generic pagination envelope returned by all NetBox list endpoints.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Paginated<T> {
    /// Total number of objects matching the filter.
    pub count: u64,
    /// URL of the next page, if any.
    pub next: Option<String>,
    /// URL of the previous page, if any.
    pub previous: Option<String>,
    /// Objects on this page.
    pub results: Vec<T>,
}

/// Tag as returned in API responses.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NestedTag {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this tag.
    pub url: String,
    /// URL for the detail view.
    pub display_url: String,
    /// Human-readable representation.
    pub display: String,
    /// Tag name.
    pub name: String,
    /// URL-safe identifier.
    pub slug: String,
    /// Hex color code (e.g. `ff0000`).
    #[serde(default)]
    pub color: String,
}

/// Tag sent in write requests.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct NestedTagRequest {
    /// Tag name.
    pub name: String,
    /// URL-safe identifier.
    pub slug: String,
    /// Hex color code (e.g. `ff0000`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

/// Generic value/label pair used for choice fields (status, priority, role, etc.).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StatusValue {
    /// Machine-readable choice key (e.g. `active`).
    pub value: Option<String>,
    /// Human-readable label (e.g. `Active`).
    pub label: Option<String>,
}

/// Nested tenant as returned in API responses.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BriefTenant {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this tenant.
    pub url: String,
    /// Human-readable representation.
    pub display: String,
    /// Full name of the tenant.
    pub name: String,
    /// URL-safe identifier.
    pub slug: String,
    /// Short description.
    #[serde(default)]
    pub description: String,
}

/// Nested owner as returned in API responses.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BriefOwner {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this owner.
    pub url: String,
    /// Human-readable representation.
    pub display: String,
    /// Full name.
    pub name: String,
    /// Short description.
    #[serde(default)]
    pub description: String,
}

/// Nested device as returned in API responses.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BriefDevice {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this device.
    pub url: String,
    /// Human-readable representation.
    pub display: String,
    /// Device hostname.
    pub name: Option<String>,
    /// Short description.
    #[serde(default)]
    pub description: String,
}

/// Nested interface as returned in API responses.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BriefInterface {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this interface.
    pub url: String,
    /// Human-readable representation.
    pub display: String,
    /// Device that owns this interface.
    pub device: BriefDevice,
    /// Interface name.
    pub name: String,
    /// Short description.
    #[serde(default)]
    pub description: String,
    /// Attached cable, if any.
    pub cable: Option<crate::JsonValue>,
    /// Whether a cable or peer link occupies this interface.
    #[serde(rename = "_occupied")]
    pub occupied: bool,
}

// ── Circuit-specific nested types ───────────────────────────────────────────

/// Condensed provider as returned in circuit-related responses.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BriefProvider {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this provider.
    pub url: String,
    /// Human-readable representation.
    pub display: String,
    /// Full name of the provider.
    pub name: String,
    /// URL-safe identifier.
    pub slug: String,
    /// Short description.
    #[serde(default)]
    pub description: String,
    /// Number of circuits with this provider.
    #[serde(default)]
    pub circuit_count: i64,
}

/// Condensed circuit type as returned in circuit responses.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BriefCircuitType {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this circuit type.
    pub url: String,
    /// Human-readable representation.
    pub display: String,
    /// Circuit type name.
    pub name: String,
    /// URL-safe identifier.
    pub slug: String,
    /// Short description.
    #[serde(default)]
    pub description: String,
    /// Number of circuits of this type.
    #[serde(default)]
    pub circuit_count: i64,
}

/// Condensed circuit group as returned in assignment responses.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BriefCircuitGroup {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this circuit group.
    pub url: String,
    /// Human-readable representation.
    pub display: String,
    /// Group name.
    pub name: String,
}

/// Condensed provider network as returned in virtual-circuit responses.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BriefProviderNetwork {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this provider network.
    pub url: String,
    /// Human-readable representation.
    pub display: String,
    /// Provider network name.
    pub name: String,
    /// Short description.
    #[serde(default)]
    pub description: String,
}

/// Condensed provider account as returned in circuit responses.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BriefProviderAccount {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this provider account.
    pub url: String,
    /// Human-readable representation.
    pub display: String,
    /// Optional display name for the account.
    pub name: Option<String>,
    /// Account identifier string.
    pub account: String,
    /// Short description.
    #[serde(default)]
    pub description: String,
}

/// Condensed circuit as returned in termination responses.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BriefCircuit {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this circuit.
    pub url: String,
    /// Human-readable representation.
    pub display: String,
    /// Unique circuit ID string.
    pub cid: String,
    /// Provider for this circuit.
    pub provider: BriefProvider,
    /// Short description.
    #[serde(default)]
    pub description: String,
}

/// Condensed virtual circuit as returned in virtual-circuit-termination responses.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BriefVirtualCircuit {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this virtual circuit.
    pub url: String,
    /// Human-readable representation.
    pub display: String,
    /// Unique circuit ID string.
    pub cid: String,
    /// Provider network for this virtual circuit.
    pub provider_network: BriefProviderNetwork,
    /// Short description.
    #[serde(default)]
    pub description: String,
}

/// Condensed virtual circuit type as returned in virtual-circuit responses.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BriefVirtualCircuitType {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this virtual circuit type.
    pub url: String,
    /// Human-readable representation.
    pub display: String,
    /// Type name.
    pub name: String,
    /// URL-safe identifier.
    pub slug: String,
    /// Short description.
    #[serde(default)]
    pub description: String,
    /// Number of virtual circuits of this type.
    #[serde(default)]
    pub virtual_circuit_count: i64,
}

/// Condensed circuit termination embedded directly in a `Circuit` response
/// (without the back-reference to the parent circuit).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CircuitCircuitTermination {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this termination.
    pub url: String,
    /// URL for the detail view.
    pub display_url: String,
    /// Human-readable representation.
    pub display: String,
    /// Content-type string for the termination object (e.g. `dcim.site`).
    pub termination_type: Option<String>,
    /// Primary key of the termination object.
    pub termination_id: Option<i64>,
    /// Termination object (site, provider network, etc.).
    pub termination: Option<crate::JsonValue>,
    /// Physical circuit speed in Kbps.
    pub port_speed: Option<i64>,
    /// Upstream speed in Kbps, if different from port speed.
    pub upstream_speed: Option<i64>,
    /// ID of the local cross-connect.
    #[serde(default)]
    pub xconnect_id: String,
    /// Short description.
    #[serde(default)]
    pub description: String,
}

/// Condensed circuit group assignment embedded in `Circuit` responses.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BriefCircuitGroupAssignment {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this assignment.
    pub url: String,
    /// Human-readable representation.
    pub display: String,
    /// Circuit group.
    pub group: BriefCircuitGroup,
    /// Assignment priority.
    pub priority: Option<StatusValue>,
}

// ── Full response types ─────────────────────────────────────────────────────

/// A physical or virtual circuit between two termination points.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Circuit {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this circuit.
    pub url: String,
    /// URL for the detail view.
    pub display_url: String,
    /// Human-readable representation.
    pub display: String,
    /// Unique circuit ID string.
    pub cid: String,
    /// Provider for this circuit.
    pub provider: BriefProvider,
    /// Provider account, if any.
    pub provider_account: Option<BriefProviderAccount>,
    /// Circuit type.
    #[serde(rename = "type")]
    pub r#type: BriefCircuitType,
    /// Operational status.
    pub status: Option<StatusValue>,
    /// Tenant this circuit belongs to.
    pub tenant: Option<BriefTenant>,
    /// Date the circuit was installed (`YYYY-MM-DD`).
    pub install_date: Option<String>,
    /// Date the circuit terminates (`YYYY-MM-DD`).
    pub termination_date: Option<String>,
    /// Committed rate in Kbps.
    pub commit_rate: Option<i64>,
    /// Short description.
    #[serde(default)]
    pub description: String,
    /// Distance value.
    pub distance: Option<f64>,
    /// Unit for the distance field.
    pub distance_unit: Option<StatusValue>,
    /// A-side termination.
    pub termination_a: Option<CircuitCircuitTermination>,
    /// Z-side termination.
    pub termination_z: Option<CircuitCircuitTermination>,
    /// Owner object.
    pub owner: Option<BriefOwner>,
    /// Freeform comments.
    #[serde(default)]
    pub comments: String,
    /// Tags attached to this circuit.
    #[serde(default)]
    pub tags: Vec<NestedTag>,
    /// Custom fields.
    pub custom_fields: Option<crate::JsonValue>,
    /// Date/time this object was created.
    pub created: Option<json_ts::JsonTimestamp>,
    /// Date/time this object was last updated.
    pub last_updated: Option<json_ts::JsonTimestamp>,
    /// Circuit group assignments.
    #[serde(default)]
    pub assignments: Vec<BriefCircuitGroupAssignment>,
}

/// A category of circuit (e.g. Dark Fiber, MPLS, Internet).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CircuitType {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this circuit type.
    pub url: String,
    /// URL for the detail view.
    pub display_url: String,
    /// Human-readable representation.
    pub display: String,
    /// Circuit type name.
    pub name: String,
    /// URL-safe identifier.
    pub slug: String,
    /// Hex color code (e.g. `ff0000`).
    #[serde(default)]
    pub color: String,
    /// Short description.
    #[serde(default)]
    pub description: String,
    /// Owner object.
    pub owner: Option<BriefOwner>,
    /// Freeform comments.
    #[serde(default)]
    pub comments: String,
    /// Tags attached to this circuit type.
    #[serde(default)]
    pub tags: Vec<NestedTag>,
    /// Custom fields.
    pub custom_fields: Option<crate::JsonValue>,
    /// Date/time this object was created.
    pub created: Option<json_ts::JsonTimestamp>,
    /// Date/time this object was last updated.
    pub last_updated: Option<json_ts::JsonTimestamp>,
    /// Number of circuits of this type.
    #[serde(default)]
    pub circuit_count: i64,
}

/// A physical termination point on a circuit.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CircuitTermination {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this termination.
    pub url: String,
    /// URL for the detail view.
    pub display_url: String,
    /// Human-readable representation.
    pub display: String,
    /// Circuit this termination belongs to.
    pub circuit: BriefCircuit,
    /// Which side of the circuit this termination is on (`A` or `Z`).
    pub term_side: String,
    /// Content-type string for the termination object (e.g. `dcim.site`).
    pub termination_type: Option<String>,
    /// Primary key of the termination object.
    pub termination_id: Option<i64>,
    /// Termination object (site, provider network, etc.).
    pub termination: Option<crate::JsonValue>,
    /// Physical circuit speed in Kbps.
    pub port_speed: Option<i64>,
    /// Upstream speed in Kbps, if different from port speed.
    pub upstream_speed: Option<i64>,
    /// ID of the local cross-connect.
    #[serde(default)]
    pub xconnect_id: String,
    /// Patch panel ID and port number(s).
    #[serde(default)]
    pub pp_info: String,
    /// Short description.
    #[serde(default)]
    pub description: String,
    /// Treat as if a cable is connected.
    pub mark_connected: bool,
    /// Attached cable, if any.
    pub cable: Option<crate::JsonValue>,
    /// Which end of the cable this termination is on.
    pub cable_end: String,
    /// Peer link terminations.
    #[serde(default)]
    pub link_peers: Vec<crate::JsonValue>,
    /// Type of the peer link terminations.
    pub link_peers_type: Option<String>,
    /// Tags attached to this termination.
    #[serde(default)]
    pub tags: Vec<NestedTag>,
    /// Custom fields.
    pub custom_fields: Option<crate::JsonValue>,
    /// Date/time this object was created.
    pub created: Option<json_ts::JsonTimestamp>,
    /// Date/time this object was last updated.
    pub last_updated: Option<json_ts::JsonTimestamp>,
    /// Whether a cable or peer link occupies this termination.
    #[serde(rename = "_occupied")]
    pub occupied: bool,
}

/// A network service provider.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Provider {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this provider.
    pub url: String,
    /// URL for the detail view.
    pub display_url: String,
    /// Human-readable representation.
    pub display: String,
    /// Full name of the provider.
    pub name: String,
    /// URL-safe identifier.
    pub slug: String,
    /// Provider accounts.
    #[serde(default)]
    pub accounts: Vec<BriefProviderAccount>,
    /// Short description.
    #[serde(default)]
    pub description: String,
    /// Owner object.
    pub owner: Option<BriefOwner>,
    /// Freeform comments.
    #[serde(default)]
    pub comments: String,
    /// ASNs associated with this provider.
    #[serde(default)]
    pub asns: Vec<crate::JsonValue>,
    /// Tags attached to this provider.
    #[serde(default)]
    pub tags: Vec<NestedTag>,
    /// Custom fields.
    pub custom_fields: Option<crate::JsonValue>,
    /// Date/time this object was created.
    pub created: Option<json_ts::JsonTimestamp>,
    /// Date/time this object was last updated.
    pub last_updated: Option<json_ts::JsonTimestamp>,
    /// Number of circuits with this provider.
    #[serde(default)]
    pub circuit_count: i64,
}

/// A named network within a provider (e.g. MPLS backbone, internet peering).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProviderNetwork {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this provider network.
    pub url: String,
    /// URL for the detail view.
    pub display_url: String,
    /// Human-readable representation.
    pub display: String,
    /// Provider that operates this network.
    pub provider: BriefProvider,
    /// Provider network name.
    pub name: String,
    /// Service identifier from the provider.
    #[serde(default)]
    pub service_id: String,
    /// Short description.
    #[serde(default)]
    pub description: String,
    /// Owner object.
    pub owner: Option<BriefOwner>,
    /// Freeform comments.
    #[serde(default)]
    pub comments: String,
    /// Tags attached to this provider network.
    #[serde(default)]
    pub tags: Vec<NestedTag>,
    /// Custom fields.
    pub custom_fields: Option<crate::JsonValue>,
    /// Date/time this object was created.
    pub created: Option<json_ts::JsonTimestamp>,
    /// Date/time this object was last updated.
    pub last_updated: Option<json_ts::JsonTimestamp>,
}

/// A billing or contract account with a provider.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProviderAccount {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this provider account.
    pub url: String,
    /// URL for the detail view.
    pub display_url: String,
    /// Human-readable representation.
    pub display: String,
    /// Provider this account belongs to.
    pub provider: BriefProvider,
    /// Optional display name for the account.
    pub name: Option<String>,
    /// Account identifier string.
    pub account: String,
    /// Short description.
    #[serde(default)]
    pub description: String,
    /// Owner object.
    pub owner: Option<BriefOwner>,
    /// Freeform comments.
    #[serde(default)]
    pub comments: String,
    /// Tags attached to this provider account.
    #[serde(default)]
    pub tags: Vec<NestedTag>,
    /// Custom fields.
    pub custom_fields: Option<crate::JsonValue>,
    /// Date/time this object was created.
    pub created: Option<json_ts::JsonTimestamp>,
    /// Date/time this object was last updated.
    pub last_updated: Option<json_ts::JsonTimestamp>,
}

/// A logical grouping of circuits.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CircuitGroup {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this circuit group.
    pub url: String,
    /// URL for the detail view.
    pub display_url: String,
    /// Human-readable representation.
    pub display: String,
    /// Group name.
    pub name: String,
    /// URL-safe identifier.
    pub slug: String,
    /// Short description.
    #[serde(default)]
    pub description: String,
    /// Tenant this group belongs to.
    pub tenant: Option<BriefTenant>,
    /// Owner object.
    pub owner: Option<BriefOwner>,
    /// Freeform comments.
    #[serde(default)]
    pub comments: String,
    /// Tags attached to this circuit group.
    #[serde(default)]
    pub tags: Vec<NestedTag>,
    /// Custom fields.
    pub custom_fields: Option<crate::JsonValue>,
    /// Date/time this object was created.
    pub created: Option<json_ts::JsonTimestamp>,
    /// Date/time this object was last updated.
    pub last_updated: Option<json_ts::JsonTimestamp>,
    /// Number of circuits in this group.
    #[serde(default)]
    pub circuit_count: i64,
}

/// An assignment of a circuit (or other member) to a circuit group.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CircuitGroupAssignment {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this assignment.
    pub url: String,
    /// URL for the detail view.
    pub display_url: String,
    /// Human-readable representation.
    pub display: String,
    /// Circuit group.
    pub group: BriefCircuitGroup,
    /// Content-type string for the member object.
    pub member_type: String,
    /// Primary key of the member object.
    pub member_id: i64,
    /// Member object.
    pub member: Option<crate::JsonValue>,
    /// Assignment priority.
    pub priority: Option<StatusValue>,
    /// Tags attached to this assignment.
    #[serde(default)]
    pub tags: Vec<NestedTag>,
    /// Date/time this object was created.
    pub created: Option<json_ts::JsonTimestamp>,
    /// Date/time this object was last updated.
    pub last_updated: Option<json_ts::JsonTimestamp>,
}

/// A virtual circuit within a provider network.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VirtualCircuit {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this virtual circuit.
    pub url: String,
    /// URL for the detail view.
    pub display_url: String,
    /// Human-readable representation.
    pub display: String,
    /// Unique circuit ID string.
    pub cid: String,
    /// Provider network this circuit runs over.
    pub provider_network: BriefProviderNetwork,
    /// Provider account, if any.
    pub provider_account: Option<BriefProviderAccount>,
    /// Virtual circuit type.
    #[serde(rename = "type")]
    pub r#type: BriefVirtualCircuitType,
    /// Operational status.
    pub status: Option<StatusValue>,
    /// Tenant this circuit belongs to.
    pub tenant: Option<BriefTenant>,
    /// Short description.
    #[serde(default)]
    pub description: String,
    /// Owner object.
    pub owner: Option<BriefOwner>,
    /// Freeform comments.
    #[serde(default)]
    pub comments: String,
    /// Tags attached to this virtual circuit.
    #[serde(default)]
    pub tags: Vec<NestedTag>,
    /// Custom fields.
    pub custom_fields: Option<crate::JsonValue>,
    /// Date/time this object was created.
    pub created: Option<json_ts::JsonTimestamp>,
    /// Date/time this object was last updated.
    pub last_updated: Option<json_ts::JsonTimestamp>,
}

/// A category of virtual circuit.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VirtualCircuitType {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this virtual circuit type.
    pub url: String,
    /// URL for the detail view.
    pub display_url: String,
    /// Human-readable representation.
    pub display: String,
    /// Type name.
    pub name: String,
    /// URL-safe identifier.
    pub slug: String,
    /// Hex color code (e.g. `ff0000`).
    #[serde(default)]
    pub color: String,
    /// Short description.
    #[serde(default)]
    pub description: String,
    /// Owner object.
    pub owner: Option<BriefOwner>,
    /// Freeform comments.
    #[serde(default)]
    pub comments: String,
    /// Tags attached to this virtual circuit type.
    #[serde(default)]
    pub tags: Vec<NestedTag>,
    /// Custom fields.
    pub custom_fields: Option<crate::JsonValue>,
    /// Date/time this object was created.
    pub created: Option<json_ts::JsonTimestamp>,
    /// Date/time this object was last updated.
    pub last_updated: Option<json_ts::JsonTimestamp>,
    /// Number of virtual circuits of this type.
    #[serde(default)]
    pub virtual_circuit_count: i64,
}

/// A termination point for a virtual circuit.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VirtualCircuitTermination {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this termination.
    pub url: String,
    /// URL for the detail view.
    pub display_url: String,
    /// Human-readable representation.
    pub display: String,
    /// Virtual circuit this termination belongs to.
    pub virtual_circuit: BriefVirtualCircuit,
    /// Termination role.
    pub role: Option<StatusValue>,
    /// Interface at this termination.
    pub interface: BriefInterface,
    /// Short description.
    #[serde(default)]
    pub description: String,
    /// Tags attached to this termination.
    #[serde(default)]
    pub tags: Vec<NestedTag>,
    /// Custom fields.
    pub custom_fields: Option<crate::JsonValue>,
    /// Date/time this object was created.
    pub created: Option<json_ts::JsonTimestamp>,
    /// Date/time this object was last updated.
    pub last_updated: Option<json_ts::JsonTimestamp>,
}

// ── Request types ────────────────────────────────────────────────────────────

/// Request body for creating or replacing a circuit (POST / PUT).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CircuitRequest {
    /// Unique circuit ID string.
    pub cid: String,
    /// Provider (ID).
    pub provider: i64,
    /// Provider account (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_account: Option<i64>,
    /// Circuit type (ID).
    #[serde(rename = "type")]
    pub r#type: i64,
    /// Operational status (`planned`, `provisioning`, `active`, `offline`,
    /// `deprovisioning`, `decommissioned`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Tenant (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant: Option<i64>,
    /// Date the circuit was installed (`YYYY-MM-DD`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_date: Option<String>,
    /// Date the circuit terminates (`YYYY-MM-DD`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub termination_date: Option<String>,
    /// Committed rate in Kbps.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_rate: Option<i64>,
    /// Short description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Distance value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance: Option<f64>,
    /// Unit for the distance field (`km`, `m`, `mi`, `ft`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance_unit: Option<String>,
    /// Owner (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<i64>,
    /// Freeform comments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
    /// Tags to attach.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<NestedTagRequest>,
    /// Custom fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<crate::JsonValue>,
}

/// Request body for partially updating a circuit (PATCH).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CircuitPatchRequest {
    /// Unique circuit ID string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cid: Option<String>,
    /// Provider (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<i64>,
    /// Provider account (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_account: Option<i64>,
    /// Circuit type (ID).
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<i64>,
    /// Operational status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Tenant (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant: Option<i64>,
    /// Date the circuit was installed (`YYYY-MM-DD`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_date: Option<String>,
    /// Date the circuit terminates (`YYYY-MM-DD`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub termination_date: Option<String>,
    /// Committed rate in Kbps.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_rate: Option<i64>,
    /// Short description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Distance value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance: Option<f64>,
    /// Unit for the distance field (`km`, `m`, `mi`, `ft`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance_unit: Option<String>,
    /// Owner (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<i64>,
    /// Freeform comments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
    /// Tags to attach.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<NestedTagRequest>,
    /// Custom fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<crate::JsonValue>,
}

/// Request body for creating or replacing a circuit type (POST / PUT).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CircuitTypeRequest {
    /// Circuit type name.
    pub name: String,
    /// URL-safe identifier.
    pub slug: String,
    /// Hex color code (e.g. `ff0000`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Short description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Owner (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<i64>,
    /// Freeform comments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
    /// Tags to attach.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<NestedTagRequest>,
    /// Custom fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<crate::JsonValue>,
}

/// Request body for partially updating a circuit type (PATCH).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CircuitTypePatchRequest {
    /// Circuit type name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// URL-safe identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    /// Hex color code (e.g. `ff0000`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Short description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Owner (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<i64>,
    /// Freeform comments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
    /// Tags to attach.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<NestedTagRequest>,
    /// Custom fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<crate::JsonValue>,
}

/// Request body for creating or replacing a circuit termination (POST / PUT).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CircuitTerminationRequest {
    /// Circuit (ID).
    pub circuit: i64,
    /// Which side of the circuit this is (`A` or `Z`).
    pub term_side: String,
    /// Content-type string for the termination object (e.g. `dcim.site`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub termination_type: Option<String>,
    /// Primary key of the termination object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub termination_id: Option<i64>,
    /// Physical circuit speed in Kbps.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port_speed: Option<i64>,
    /// Upstream speed in Kbps, if different from port speed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upstream_speed: Option<i64>,
    /// ID of the local cross-connect.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xconnect_id: Option<String>,
    /// Patch panel ID and port number(s).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pp_info: Option<String>,
    /// Short description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Treat as if a cable is connected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mark_connected: Option<bool>,
    /// Tags to attach.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<NestedTagRequest>,
    /// Custom fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<crate::JsonValue>,
}

/// Request body for partially updating a circuit termination (PATCH).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CircuitTerminationPatchRequest {
    /// Circuit (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub circuit: Option<i64>,
    /// Which side of the circuit this is (`A` or `Z`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub term_side: Option<String>,
    /// Content-type string for the termination object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub termination_type: Option<String>,
    /// Primary key of the termination object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub termination_id: Option<i64>,
    /// Physical circuit speed in Kbps.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port_speed: Option<i64>,
    /// Upstream speed in Kbps, if different from port speed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upstream_speed: Option<i64>,
    /// ID of the local cross-connect.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xconnect_id: Option<String>,
    /// Patch panel ID and port number(s).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pp_info: Option<String>,
    /// Short description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Treat as if a cable is connected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mark_connected: Option<bool>,
    /// Tags to attach.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<NestedTagRequest>,
    /// Custom fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<crate::JsonValue>,
}

/// Request body for creating or replacing a provider (POST / PUT).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProviderRequest {
    /// Full name of the provider.
    pub name: String,
    /// URL-safe identifier.
    pub slug: String,
    /// Provider account IDs.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub accounts: Vec<i64>,
    /// Short description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Owner (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<i64>,
    /// Freeform comments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
    /// ASN IDs.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub asns: Vec<i64>,
    /// Tags to attach.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<NestedTagRequest>,
    /// Custom fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<crate::JsonValue>,
}

/// Request body for partially updating a provider (PATCH).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ProviderPatchRequest {
    /// Full name of the provider.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// URL-safe identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    /// Provider account IDs.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub accounts: Vec<i64>,
    /// Short description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Owner (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<i64>,
    /// Freeform comments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
    /// ASN IDs.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub asns: Vec<i64>,
    /// Tags to attach.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<NestedTagRequest>,
    /// Custom fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<crate::JsonValue>,
}

/// Request body for creating or replacing a provider network (POST / PUT).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProviderNetworkRequest {
    /// Provider (ID).
    pub provider: i64,
    /// Provider network name.
    pub name: String,
    /// Service identifier from the provider.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
    /// Short description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Owner (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<i64>,
    /// Freeform comments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
    /// Tags to attach.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<NestedTagRequest>,
    /// Custom fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<crate::JsonValue>,
}

/// Request body for partially updating a provider network (PATCH).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ProviderNetworkPatchRequest {
    /// Provider (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<i64>,
    /// Provider network name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Service identifier from the provider.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_id: Option<String>,
    /// Short description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Owner (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<i64>,
    /// Freeform comments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
    /// Tags to attach.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<NestedTagRequest>,
    /// Custom fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<crate::JsonValue>,
}

/// Request body for creating or replacing a provider account (POST / PUT).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProviderAccountRequest {
    /// Provider (ID).
    pub provider: i64,
    /// Optional display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Account identifier string.
    pub account: String,
    /// Short description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Owner (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<i64>,
    /// Freeform comments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
    /// Tags to attach.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<NestedTagRequest>,
    /// Custom fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<crate::JsonValue>,
}

/// Request body for partially updating a provider account (PATCH).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ProviderAccountPatchRequest {
    /// Provider (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<i64>,
    /// Optional display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Account identifier string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    /// Short description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Owner (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<i64>,
    /// Freeform comments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
    /// Tags to attach.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<NestedTagRequest>,
    /// Custom fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<crate::JsonValue>,
}

/// Request body for creating or replacing a circuit group (POST / PUT).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CircuitGroupRequest {
    /// Group name.
    pub name: String,
    /// URL-safe identifier.
    pub slug: String,
    /// Short description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Tenant (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant: Option<i64>,
    /// Owner (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<i64>,
    /// Freeform comments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
    /// Tags to attach.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<NestedTagRequest>,
    /// Custom fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<crate::JsonValue>,
}

/// Request body for partially updating a circuit group (PATCH).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CircuitGroupPatchRequest {
    /// Group name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// URL-safe identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    /// Short description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Tenant (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant: Option<i64>,
    /// Owner (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<i64>,
    /// Freeform comments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
    /// Tags to attach.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<NestedTagRequest>,
    /// Custom fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<crate::JsonValue>,
}

/// Request body for creating or replacing a circuit group assignment (POST / PUT).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CircuitGroupAssignmentRequest {
    /// Circuit group (ID).
    pub group: i64,
    /// Content-type string of the member object (e.g. `circuits.circuit`).
    pub member_type: String,
    /// Primary key of the member object.
    pub member_id: i64,
    /// Assignment priority (`primary`, `secondary`, `tertiary`, `inactive`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    /// Tags to attach.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<NestedTagRequest>,
}

/// Request body for partially updating a circuit group assignment (PATCH).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CircuitGroupAssignmentPatchRequest {
    /// Circuit group (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<i64>,
    /// Content-type string of the member object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member_type: Option<String>,
    /// Primary key of the member object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member_id: Option<i64>,
    /// Assignment priority.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    /// Tags to attach.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<NestedTagRequest>,
}

/// Request body for creating or replacing a virtual circuit (POST / PUT).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VirtualCircuitRequest {
    /// Unique circuit ID string.
    pub cid: String,
    /// Provider network (ID).
    pub provider_network: i64,
    /// Provider account (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_account: Option<i64>,
    /// Virtual circuit type (ID).
    #[serde(rename = "type")]
    pub r#type: i64,
    /// Operational status (`planned`, `provisioning`, `active`, `offline`,
    /// `deprovisioning`, `decommissioned`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Tenant (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant: Option<i64>,
    /// Short description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Owner (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<i64>,
    /// Freeform comments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
    /// Tags to attach.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<NestedTagRequest>,
    /// Custom fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<crate::JsonValue>,
}

/// Request body for partially updating a virtual circuit (PATCH).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct VirtualCircuitPatchRequest {
    /// Unique circuit ID string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cid: Option<String>,
    /// Provider network (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_network: Option<i64>,
    /// Provider account (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_account: Option<i64>,
    /// Virtual circuit type (ID).
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<i64>,
    /// Operational status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Tenant (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant: Option<i64>,
    /// Short description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Owner (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<i64>,
    /// Freeform comments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
    /// Tags to attach.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<NestedTagRequest>,
    /// Custom fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<crate::JsonValue>,
}

/// Request body for creating or replacing a virtual circuit type (POST / PUT).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VirtualCircuitTypeRequest {
    /// Type name.
    pub name: String,
    /// URL-safe identifier.
    pub slug: String,
    /// Hex color code (e.g. `ff0000`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Short description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Owner (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<i64>,
    /// Freeform comments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
    /// Tags to attach.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<NestedTagRequest>,
    /// Custom fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<crate::JsonValue>,
}

/// Request body for partially updating a virtual circuit type (PATCH).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct VirtualCircuitTypePatchRequest {
    /// Type name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// URL-safe identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    /// Hex color code (e.g. `ff0000`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Short description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Owner (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<i64>,
    /// Freeform comments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
    /// Tags to attach.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<NestedTagRequest>,
    /// Custom fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<crate::JsonValue>,
}

/// Request body for creating or replacing a virtual circuit termination (POST / PUT).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VirtualCircuitTerminationRequest {
    /// Virtual circuit (ID).
    pub virtual_circuit: i64,
    /// Termination role (`peer`, `hub`, `spoke`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Interface (ID).
    pub interface: i64,
    /// Short description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Tags to attach.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<NestedTagRequest>,
    /// Custom fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<crate::JsonValue>,
}

/// Request body for partially updating a virtual circuit termination (PATCH).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct VirtualCircuitTerminationPatchRequest {
    /// Virtual circuit (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub virtual_circuit: Option<i64>,
    /// Termination role (`peer`, `hub`, `spoke`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Interface (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interface: Option<i64>,
    /// Short description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Tags to attach.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<NestedTagRequest>,
    /// Custom fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<crate::JsonValue>,
}

// ── IPAM types ────────────────────────────────────────────────────────────────

/// Condensed RIR as returned in aggregate responses.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BriefRIR {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this RIR.
    pub url: String,
    /// Human-readable representation.
    pub display: String,
    /// RIR name.
    pub name: String,
    /// URL-safe identifier.
    pub slug: String,
    /// Short description.
    #[serde(default)]
    pub description: String,
    /// Number of aggregates assigned to this RIR.
    #[serde(default)]
    pub aggregate_count: i64,
}

/// Condensed VRF as returned in IP-address and prefix responses.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BriefVRF {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this VRF.
    pub url: String,
    /// Human-readable representation.
    pub display: String,
    /// VRF name.
    pub name: String,
    /// Route distinguisher.
    pub rd: Option<String>,
    /// Short description.
    #[serde(default)]
    pub description: String,
    /// Number of IP prefixes in this VRF.
    #[serde(default)]
    pub prefix_count: i64,
}

/// Condensed VLAN as returned in prefix responses.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BriefVLAN {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this VLAN.
    pub url: String,
    /// Human-readable representation.
    pub display: String,
    /// VLAN ID (1–4094).
    pub vid: i64,
    /// VLAN name.
    pub name: String,
    /// Short description.
    #[serde(default)]
    pub description: String,
}

/// Condensed IP address as returned in nat-inside/nat-outside fields.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BriefIPAddress {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this IP address.
    pub url: String,
    /// Human-readable representation.
    pub display: String,
    /// IP address family.
    pub family: FamilyValue,
    /// Address with prefix length (e.g. `192.0.2.1/24`).
    pub address: String,
    /// Short description.
    #[serde(default)]
    pub description: String,
}

/// Condensed IPAM role as returned in prefix responses.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BriefRole {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this role.
    pub url: String,
    /// Human-readable representation.
    pub display: String,
    /// Role name.
    pub name: String,
    /// URL-safe identifier.
    pub slug: String,
    /// Short description.
    #[serde(default)]
    pub description: String,
    /// Number of prefixes with this role.
    #[serde(default)]
    pub prefix_count: i64,
    /// Number of VLANs with this role.
    #[serde(default)]
    pub vlan_count: i64,
}

/// IP address family (IPv4 or IPv6) as returned by the API.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FamilyValue {
    /// Numeric IP version (4 or 6).
    pub value: Option<i64>,
    /// Human-readable label (`IPv4` or `IPv6`).
    pub label: Option<String>,
}

/// An IP address (with prefix-length mask).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IpAddress {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this IP address.
    pub url: String,
    /// URL for the detail view.
    pub display_url: String,
    /// Human-readable representation.
    pub display: String,
    /// IP address family.
    pub family: FamilyValue,
    /// IP address with prefix length (e.g. `192.0.2.1/24`).
    pub address: String,
    /// VRF this address belongs to.
    pub vrf: Option<BriefVRF>,
    /// Tenant this address is assigned to.
    pub tenant: Option<BriefTenant>,
    /// Operational status.
    pub status: Option<StatusValue>,
    /// Functional role of this IP address.
    pub role: Option<StatusValue>,
    /// Content-type of the assigned object (e.g. `dcim.interface`).
    pub assigned_object_type: Option<String>,
    /// Primary key of the assigned object.
    pub assigned_object_id: Option<i64>,
    /// Assigned object (raw JSON — type depends on `assigned_object_type`).
    pub assigned_object: Option<crate::JsonValue>,
    /// NAT inside address this IP is a translation of.
    pub nat_inside: Option<BriefIPAddress>,
    /// NAT outside addresses derived from this IP.
    #[serde(default)]
    pub nat_outside: Vec<BriefIPAddress>,
    /// DNS hostname or FQDN (not case-sensitive).
    pub dns_name: Option<String>,
    /// Short description.
    #[serde(default)]
    pub description: String,
    /// Owner object.
    pub owner: Option<BriefOwner>,
    /// Freeform comments.
    #[serde(default)]
    pub comments: String,
    /// Tags attached to this IP address.
    #[serde(default)]
    pub tags: Vec<NestedTag>,
    /// Custom fields.
    pub custom_fields: Option<crate::JsonValue>,
    /// Date/time this object was created.
    pub created: Option<json_ts::JsonTimestamp>,
    /// Date/time this object was last updated.
    pub last_updated: Option<json_ts::JsonTimestamp>,
}

/// Request body for creating or replacing an IP address (POST / PUT).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct IpAddressRequest {
    /// IP address with prefix length (e.g. `192.0.2.1/24`).
    pub address: String,
    /// VRF (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vrf: Option<i64>,
    /// Tenant (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant: Option<i64>,
    /// Operational status (`active`, `reserved`, `deprecated`, `dhcp`, `slaac`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Functional role (`loopback`, `secondary`, `anycast`, `vip`, `vrrp`, `hsrp`, `glbp`, `carp`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Content-type of the assigned object (e.g. `dcim.interface`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_object_type: Option<String>,
    /// Primary key of the assigned object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_object_id: Option<i64>,
    /// NAT inside address (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nat_inside: Option<i64>,
    /// DNS hostname or FQDN (not case-sensitive).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns_name: Option<String>,
    /// Short description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Owner (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<i64>,
    /// Freeform comments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
    /// Tags to attach.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<NestedTagRequest>,
    /// Custom fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<crate::JsonValue>,
}

/// Request body for partially updating an IP address (PATCH).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct IpAddressPatchRequest {
    /// IP address with prefix length (e.g. `192.0.2.1/24`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    /// VRF (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vrf: Option<i64>,
    /// Tenant (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant: Option<i64>,
    /// Operational status (`active`, `reserved`, `deprecated`, `dhcp`, `slaac`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Functional role (`loopback`, `secondary`, `anycast`, `vip`, `vrrp`, `hsrp`, `glbp`, `carp`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Content-type of the assigned object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_object_type: Option<String>,
    /// Primary key of the assigned object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_object_id: Option<i64>,
    /// NAT inside address (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nat_inside: Option<i64>,
    /// DNS hostname or FQDN (not case-sensitive).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dns_name: Option<String>,
    /// Short description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Owner (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<i64>,
    /// Freeform comments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
    /// Tags to attach.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<NestedTagRequest>,
    /// Custom fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<crate::JsonValue>,
}

/// An IP prefix (network with mask).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Prefix {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this prefix.
    pub url: String,
    /// URL for the detail view.
    pub display_url: String,
    /// Human-readable representation.
    pub display: String,
    /// IP address family.
    pub family: FamilyValue,
    /// IP prefix (e.g. `192.0.2.0/24`).
    pub prefix: String,
    /// VRF this prefix belongs to.
    pub vrf: Option<BriefVRF>,
    /// Scope content-type (e.g. `dcim.site`).
    pub scope_type: Option<String>,
    /// Primary key of the scope object.
    pub scope_id: Option<i64>,
    /// Scope object (raw JSON — type depends on `scope_type`).
    pub scope: Option<crate::JsonValue>,
    /// Tenant this prefix is assigned to.
    pub tenant: Option<BriefTenant>,
    /// VLAN associated with this prefix.
    pub vlan: Option<BriefVLAN>,
    /// Operational status.
    pub status: Option<StatusValue>,
    /// Functional role of this prefix.
    pub role: Option<BriefRole>,
    /// All IP addresses within this prefix are considered usable.
    #[serde(default)]
    pub is_pool: bool,
    /// Treat as fully utilized.
    #[serde(default)]
    pub mark_utilized: bool,
    /// Short description.
    #[serde(default)]
    pub description: String,
    /// Owner object.
    pub owner: Option<BriefOwner>,
    /// Freeform comments.
    #[serde(default)]
    pub comments: String,
    /// Tags attached to this prefix.
    #[serde(default)]
    pub tags: Vec<NestedTag>,
    /// Custom fields.
    pub custom_fields: Option<crate::JsonValue>,
    /// Date/time this object was created.
    pub created: Option<json_ts::JsonTimestamp>,
    /// Date/time this object was last updated.
    pub last_updated: Option<json_ts::JsonTimestamp>,
    /// Number of immediate child prefixes.
    pub children: i64,
    /// Nesting depth in the prefix hierarchy.
    #[serde(rename = "_depth")]
    pub depth: i64,
}

/// Request body for creating or replacing a prefix (POST / PUT).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PrefixRequest {
    /// IP prefix (e.g. `192.0.2.0/24`).
    pub prefix: String,
    /// VRF (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vrf: Option<i64>,
    /// Scope content-type (e.g. `dcim.site`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope_type: Option<String>,
    /// Scope object primary key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope_id: Option<i64>,
    /// Tenant (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant: Option<i64>,
    /// VLAN (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vlan: Option<i64>,
    /// Operational status (`container`, `active`, `reserved`, `deprecated`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Functional role (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<i64>,
    /// All IP addresses within this prefix are considered usable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_pool: Option<bool>,
    /// Treat as fully utilized.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mark_utilized: Option<bool>,
    /// Short description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Owner (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<i64>,
    /// Freeform comments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
    /// Tags to attach.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<NestedTagRequest>,
    /// Custom fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<crate::JsonValue>,
}

/// Request body for partially updating a prefix (PATCH).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PrefixPatchRequest {
    /// IP prefix (e.g. `192.0.2.0/24`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    /// VRF (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vrf: Option<i64>,
    /// Scope content-type (e.g. `dcim.site`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope_type: Option<String>,
    /// Scope object primary key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope_id: Option<i64>,
    /// Tenant (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant: Option<i64>,
    /// VLAN (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vlan: Option<i64>,
    /// Operational status (`container`, `active`, `reserved`, `deprecated`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Functional role (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<i64>,
    /// All IP addresses within this prefix are considered usable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_pool: Option<bool>,
    /// Treat as fully utilized.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mark_utilized: Option<bool>,
    /// Short description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Owner (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<i64>,
    /// Freeform comments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
    /// Tags to attach.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<NestedTagRequest>,
    /// Custom fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<crate::JsonValue>,
}

/// An IP aggregate (a top-level prefix assigned to a RIR).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Aggregate {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this aggregate.
    pub url: String,
    /// URL for the detail view.
    pub display_url: String,
    /// Human-readable representation.
    pub display: String,
    /// IP address family.
    pub family: FamilyValue,
    /// IP prefix (e.g. `10.0.0.0/8`).
    pub prefix: String,
    /// Regional Internet Registry that manages this aggregate.
    pub rir: BriefRIR,
    /// Tenant this aggregate is assigned to.
    pub tenant: Option<BriefTenant>,
    /// Date this aggregate was added (YYYY-MM-DD).
    pub date_added: Option<String>,
    /// Short description.
    #[serde(default)]
    pub description: String,
    /// Owner object.
    pub owner: Option<BriefOwner>,
    /// Freeform comments.
    #[serde(default)]
    pub comments: String,
    /// Tags attached to this aggregate.
    #[serde(default)]
    pub tags: Vec<NestedTag>,
    /// Custom fields.
    pub custom_fields: Option<crate::JsonValue>,
    /// Date/time this object was created.
    pub created: Option<json_ts::JsonTimestamp>,
    /// Date/time this object was last updated.
    pub last_updated: Option<json_ts::JsonTimestamp>,
}

/// Request body for creating or replacing an aggregate (POST / PUT).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AggregateRequest {
    /// IP prefix (e.g. `10.0.0.0/8`).
    pub prefix: String,
    /// RIR (ID).
    pub rir: i64,
    /// Tenant (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant: Option<i64>,
    /// Date this aggregate was added (YYYY-MM-DD).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_added: Option<String>,
    /// Short description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Owner (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<i64>,
    /// Freeform comments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
    /// Tags to attach.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<NestedTagRequest>,
    /// Custom fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<crate::JsonValue>,
}

/// Request body for partially updating an aggregate (PATCH).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AggregatePatchRequest {
    /// IP prefix (e.g. `10.0.0.0/8`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    /// RIR (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rir: Option<i64>,
    /// Tenant (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant: Option<i64>,
    /// Date this aggregate was added (YYYY-MM-DD).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_added: Option<String>,
    /// Short description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Owner (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<i64>,
    /// Freeform comments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
    /// Tags to attach.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<NestedTagRequest>,
    /// Custom fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<crate::JsonValue>,
}

/// An IPAM functional role (e.g. loopback, management, infrastructure).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Role {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this role.
    pub url: String,
    /// URL for the detail view.
    pub display_url: String,
    /// Human-readable representation.
    pub display: String,
    /// Role name.
    pub name: String,
    /// URL-safe identifier.
    pub slug: String,
    /// Sorting weight (lower values sort first).
    pub weight: Option<i64>,
    /// Short description.
    #[serde(default)]
    pub description: String,
    /// Owner object.
    pub owner: Option<BriefOwner>,
    /// Freeform comments.
    #[serde(default)]
    pub comments: String,
    /// Tags attached to this role.
    #[serde(default)]
    pub tags: Vec<NestedTag>,
    /// Custom fields.
    pub custom_fields: Option<crate::JsonValue>,
    /// Date/time this object was created.
    pub created: Option<json_ts::JsonTimestamp>,
    /// Date/time this object was last updated.
    pub last_updated: Option<json_ts::JsonTimestamp>,
    /// Number of prefixes with this role.
    #[serde(default)]
    pub prefix_count: i64,
    /// Number of VLANs with this role.
    #[serde(default)]
    pub vlan_count: i64,
}

/// Request body for creating or replacing a role (POST / PUT).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RoleRequest {
    /// Role name.
    pub name: String,
    /// URL-safe identifier.
    pub slug: String,
    /// Sorting weight (lower values sort first).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<i64>,
    /// Short description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Owner (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<i64>,
    /// Freeform comments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
    /// Tags to attach.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<NestedTagRequest>,
    /// Custom fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<crate::JsonValue>,
}

/// Request body for partially updating a role (PATCH).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RolePatchRequest {
    /// Role name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// URL-safe identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    /// Sorting weight (lower values sort first).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<i64>,
    /// Short description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Owner (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<i64>,
    /// Freeform comments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
    /// Tags to attach.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<NestedTagRequest>,
    /// Custom fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<crate::JsonValue>,
}

// ── DCIM — Site types ────────────────────────────────────────────────────────

/// Condensed region as returned in site responses.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BriefRegion {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this region.
    pub url: String,
    /// Human-readable representation.
    pub display: String,
    /// Region name.
    pub name: String,
    /// URL-safe identifier.
    pub slug: String,
    /// Short description.
    #[serde(default)]
    pub description: String,
    /// Number of sites in this region.
    #[serde(default)]
    pub site_count: i64,
    /// Nesting depth in the region hierarchy.
    #[serde(rename = "_depth")]
    pub depth: i64,
}

/// Condensed site group as returned in site responses.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BriefSiteGroup {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this site group.
    pub url: String,
    /// Human-readable representation.
    pub display: String,
    /// Site group name.
    pub name: String,
    /// URL-safe identifier.
    pub slug: String,
    /// Short description.
    #[serde(default)]
    pub description: String,
    /// Number of sites in this group.
    #[serde(default)]
    pub site_count: i64,
    /// Nesting depth in the site-group hierarchy.
    #[serde(rename = "_depth")]
    pub depth: i64,
}

/// A physical location where network equipment is installed.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Site {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this site.
    pub url: String,
    /// URL for the detail view.
    pub display_url: String,
    /// Human-readable representation.
    pub display: String,
    /// Full name of the site.
    pub name: String,
    /// URL-safe identifier.
    pub slug: String,
    /// Operational status.
    pub status: Option<StatusValue>,
    /// Geographic region this site belongs to.
    pub region: Option<BriefRegion>,
    /// Site group this site belongs to.
    pub group: Option<BriefSiteGroup>,
    /// Tenant this site belongs to.
    pub tenant: Option<BriefTenant>,
    /// Local facility ID or description.
    #[serde(default)]
    pub facility: String,
    /// IANA time zone name (e.g. `America/Chicago`).
    pub time_zone: Option<String>,
    /// Short description.
    #[serde(default)]
    pub description: String,
    /// Physical location of the building.
    #[serde(default)]
    pub physical_address: String,
    /// Shipping address, if different from the physical address.
    #[serde(default)]
    pub shipping_address: String,
    /// GPS latitude in decimal format (−90 to 90).
    pub latitude: Option<f64>,
    /// GPS longitude in decimal format (−180 to 180).
    pub longitude: Option<f64>,
    /// Owner object.
    pub owner: Option<BriefOwner>,
    /// Freeform comments.
    #[serde(default)]
    pub comments: String,
    /// ASNs associated with this site (raw JSON to avoid a full ASN model).
    #[serde(default)]
    pub asns: Vec<crate::JsonValue>,
    /// Tags attached to this site.
    #[serde(default)]
    pub tags: Vec<NestedTag>,
    /// Custom fields.
    pub custom_fields: Option<crate::JsonValue>,
    /// Date/time this object was created.
    pub created: Option<json_ts::JsonTimestamp>,
    /// Date/time this object was last updated.
    pub last_updated: Option<json_ts::JsonTimestamp>,
    /// Number of circuits terminating at this site.
    #[serde(default)]
    pub circuit_count: i64,
    /// Number of devices installed at this site.
    #[serde(default)]
    pub device_count: i64,
    /// Number of IP prefixes assigned to this site.
    #[serde(default)]
    pub prefix_count: i64,
    /// Number of racks at this site.
    #[serde(default)]
    pub rack_count: i64,
    /// Number of virtual machines at this site.
    #[serde(default)]
    pub virtualmachine_count: i64,
    /// Number of VLANs at this site.
    #[serde(default)]
    pub vlan_count: i64,
}

/// Request body for creating or replacing a site (POST / PUT).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SiteRequest {
    /// Full name of the site.
    pub name: String,
    /// URL-safe identifier.
    pub slug: String,
    /// Operational status (`planned`, `staging`, `active`, `decommissioning`, `retired`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Region (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<i64>,
    /// Site group (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<i64>,
    /// Tenant (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant: Option<i64>,
    /// Local facility ID or description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facility: Option<String>,
    /// IANA time zone name (e.g. `America/Chicago`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_zone: Option<String>,
    /// Short description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Physical location of the building.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub physical_address: Option<String>,
    /// Shipping address, if different from the physical address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shipping_address: Option<String>,
    /// GPS latitude in decimal format (−90 to 90).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latitude: Option<f64>,
    /// GPS longitude in decimal format (−180 to 180).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f64>,
    /// Owner (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<i64>,
    /// Freeform comments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
    /// ASN IDs to associate with this site.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub asns: Vec<i64>,
    /// Tags to attach.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<NestedTagRequest>,
    /// Custom fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<crate::JsonValue>,
}

/// Request body for partially updating a site (PATCH).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SitePatchRequest {
    /// Full name of the site.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// URL-safe identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
    /// Operational status (`planned`, `staging`, `active`, `decommissioning`, `retired`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Region (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<i64>,
    /// Site group (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<i64>,
    /// Tenant (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant: Option<i64>,
    /// Local facility ID or description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facility: Option<String>,
    /// IANA time zone name (e.g. `America/Chicago`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_zone: Option<String>,
    /// Short description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Physical location of the building.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub physical_address: Option<String>,
    /// Shipping address, if different from the physical address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shipping_address: Option<String>,
    /// GPS latitude in decimal format (−90 to 90).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latitude: Option<f64>,
    /// GPS longitude in decimal format (−180 to 180).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f64>,
    /// Owner (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<i64>,
    /// Freeform comments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
    /// ASN IDs to associate with this site.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub asns: Vec<i64>,
    /// Tags to attach.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<NestedTagRequest>,
    /// Custom fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<crate::JsonValue>,
}

// ── Interface-related nested types ──────────────────────────────────────────

/// Condensed module bay as returned in `BriefModule` responses.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NestedModuleBay {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this module bay.
    pub url: String,
    /// URL for the detail view.
    pub display_url: String,
    /// Human-readable representation.
    pub display: String,
    /// Module bay name.
    pub name: String,
}

/// Condensed module as returned in interface responses.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BriefModule {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this module.
    pub url: String,
    /// Human-readable representation.
    pub display: String,
    /// Device that contains this module.
    pub device: BriefDevice,
    /// Module bay this module is installed in.
    pub module_bay: NestedModuleBay,
}

/// Condensed MAC address as returned in interface responses.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BriefMACAddress {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this MAC address.
    pub url: String,
    /// Human-readable representation.
    pub display: String,
    /// MAC address in colon-separated hex notation.
    pub mac_address: String,
    /// Short description.
    #[serde(default)]
    pub description: String,
}

/// Condensed cable as returned in interface and termination responses.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BriefCable {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this cable.
    pub url: String,
    /// Human-readable representation.
    pub display: String,
    /// Optional label on the cable.
    #[serde(default)]
    pub label: String,
    /// Short description.
    #[serde(default)]
    pub description: String,
}

/// Condensed L2VPN as returned in L2VPN-termination responses.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BriefL2VPN {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this L2VPN.
    pub url: String,
    /// Human-readable representation.
    pub display: String,
    /// Optional numeric identifier.
    pub identifier: Option<i64>,
    /// L2VPN name.
    pub name: String,
    /// URL-safe identifier.
    pub slug: String,
    /// L2VPN type.
    #[serde(rename = "type")]
    pub r#type: Option<StatusValue>,
    /// Short description.
    #[serde(default)]
    pub description: String,
}

/// Condensed L2VPN termination as returned in interface responses.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BriefL2VPNTermination {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this L2VPN termination.
    pub url: String,
    /// Human-readable representation.
    pub display: String,
    /// L2VPN this termination belongs to.
    pub l2vpn: BriefL2VPN,
}

/// Condensed wireless link as returned in interface responses.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NestedWirelessLink {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this wireless link.
    pub url: String,
    /// URL for the detail view.
    pub display_url: String,
    /// Human-readable representation.
    pub display: String,
    /// Wireless SSID.
    #[serde(default)]
    pub ssid: String,
}

/// Condensed VLAN translation policy as returned in interface responses.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BriefVLANTranslationPolicy {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this translation policy.
    pub url: String,
    /// Human-readable representation.
    pub display: String,
    /// Policy name.
    pub name: String,
    /// Short description.
    #[serde(default)]
    pub description: String,
}

/// Condensed interface as returned in parent/bridge/lag fields.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NestedInterface {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this interface.
    pub url: String,
    /// URL for the detail view.
    pub display_url: String,
    /// Human-readable representation.
    pub display: String,
    /// Device that owns this interface.
    pub device: BriefDevice,
    /// Interface name.
    pub name: String,
    /// Attached cable, if any.
    pub cable: Option<crate::JsonValue>,
    /// Whether a cable or peer link occupies this interface.
    #[serde(rename = "_occupied")]
    pub occupied: bool,
}

/// A virtual device context (VDC) on a device.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VirtualDeviceContext {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this VDC.
    pub url: String,
    /// URL for the detail view.
    pub display_url: String,
    /// Human-readable representation.
    pub display: String,
    /// VDC name.
    pub name: String,
    /// Device this VDC belongs to.
    pub device: BriefDevice,
    /// Optional numeric identifier.
    pub identifier: Option<i64>,
    /// Tenant this VDC belongs to.
    pub tenant: Option<BriefTenant>,
    /// Primary IP address (either v4 or v6).
    pub primary_ip: Option<crate::JsonValue>,
    /// Primary IPv4 address.
    pub primary_ip4: Option<crate::JsonValue>,
    /// Primary IPv6 address.
    pub primary_ip6: Option<crate::JsonValue>,
    /// Operational status.
    pub status: Option<StatusValue>,
    /// Short description.
    #[serde(default)]
    pub description: String,
    /// Owner object.
    pub owner: Option<BriefOwner>,
    /// Freeform comments.
    #[serde(default)]
    pub comments: String,
    /// Tags attached to this VDC.
    #[serde(default)]
    pub tags: Vec<NestedTag>,
    /// Custom fields.
    pub custom_fields: Option<crate::JsonValue>,
    /// Date/time this object was created.
    pub created: Option<json_ts::JsonTimestamp>,
    /// Date/time this object was last updated.
    pub last_updated: Option<json_ts::JsonTimestamp>,
    /// Number of interfaces in this VDC.
    #[serde(default)]
    pub interface_count: i64,
}

/// A physical or virtual network interface on a device.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[allow(clippy::struct_excessive_bools)] // mirrors NetBox API schema directly
pub struct Interface {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this interface.
    pub url: String,
    /// URL for the detail view.
    pub display_url: String,
    /// Human-readable representation.
    pub display: String,
    /// Device this interface belongs to.
    pub device: BriefDevice,
    /// Virtual device contexts assigned to this interface.
    #[serde(default)]
    pub vdcs: Vec<VirtualDeviceContext>,
    /// Module this interface is associated with, if any.
    pub module: Option<BriefModule>,
    /// Interface name.
    pub name: String,
    /// Physical label.
    #[serde(default)]
    pub label: String,
    /// Interface type.
    #[serde(rename = "type")]
    pub r#type: StatusValue,
    /// Whether this interface is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// Parent interface (for sub-interfaces).
    pub parent: Option<NestedInterface>,
    /// Bridge interface, if any.
    pub bridge: Option<NestedInterface>,
    /// Child interfaces bridged to this one (read-only).
    #[serde(default)]
    pub bridge_interfaces: Vec<NestedInterface>,
    /// LAG interface this member belongs to, if any.
    pub lag: Option<NestedInterface>,
    /// Maximum transmission unit in bytes.
    pub mtu: Option<i64>,
    /// MAC address (read-only derived field).
    pub mac_address: Option<String>,
    /// Primary MAC address object, if any.
    pub primary_mac_address: Option<BriefMACAddress>,
    /// All MAC addresses associated with this interface (read-only).
    pub mac_addresses: Option<Vec<BriefMACAddress>>,
    /// Interface speed in Kbps.
    pub speed: Option<i64>,
    /// Duplex mode.
    pub duplex: Option<StatusValue>,
    /// World Wide Name (for Fibre Channel interfaces).
    pub wwn: Option<String>,
    /// Whether this interface is used only for out-of-band management.
    #[serde(default)]
    pub mgmt_only: bool,
    /// Short description.
    #[serde(default)]
    pub description: String,
    /// 802.1Q mode.
    pub mode: Option<StatusValue>,
    /// Wireless RF role.
    pub rf_role: Option<StatusValue>,
    /// Wireless RF channel.
    pub rf_channel: Option<StatusValue>,
    /// PoE mode.
    pub poe_mode: Option<StatusValue>,
    /// PoE type.
    pub poe_type: Option<StatusValue>,
    /// Wireless channel frequency in MHz.
    pub rf_channel_frequency: Option<f64>,
    /// Wireless channel width in MHz.
    pub rf_channel_width: Option<f64>,
    /// Transmit power in dBm.
    pub tx_power: Option<i64>,
    /// Untagged (native) VLAN, if any.
    pub untagged_vlan: Option<BriefVLAN>,
    /// Tagged VLANs allowed on this interface.
    #[serde(default)]
    pub tagged_vlans: Vec<crate::JsonValue>,
    /// Q-in-Q service VLAN, if any.
    pub qinq_svlan: Option<BriefVLAN>,
    /// VLAN translation policy applied to this interface, if any.
    pub vlan_translation_policy: Option<BriefVLANTranslationPolicy>,
    /// Treat as if a cable is connected.
    #[serde(default)]
    pub mark_connected: bool,
    /// Attached cable (read-only).
    pub cable: Option<BriefCable>,
    /// Which end of the cable this interface is on (read-only).
    #[serde(default)]
    pub cable_end: String,
    /// Wireless link (read-only).
    pub wireless_link: Option<NestedWirelessLink>,
    /// Peer link terminations (read-only).
    #[serde(default)]
    pub link_peers: Vec<crate::JsonValue>,
    /// Type of the peer link terminations (read-only).
    pub link_peers_type: Option<String>,
    /// Wireless LANs associated with this interface.
    #[serde(default)]
    pub wireless_lans: Vec<crate::JsonValue>,
    /// VRF this interface belongs to, if any.
    pub vrf: Option<BriefVRF>,
    /// L2VPN termination for this interface (read-only).
    pub l2vpn_termination: Option<BriefL2VPNTermination>,
    /// Connected endpoint objects (read-only).
    pub connected_endpoints: Option<Vec<crate::JsonValue>>,
    /// Type of the connected endpoints (read-only).
    pub connected_endpoints_type: Option<String>,
    /// Whether the connected endpoints are reachable (read-only).
    #[serde(default)]
    pub connected_endpoints_reachable: bool,
    /// Owner object.
    pub owner: Option<BriefOwner>,
    /// Tags attached to this interface.
    #[serde(default)]
    pub tags: Vec<NestedTag>,
    /// Custom fields.
    pub custom_fields: Option<crate::JsonValue>,
    /// Date/time this object was created.
    pub created: Option<json_ts::JsonTimestamp>,
    /// Date/time this object was last updated.
    pub last_updated: Option<json_ts::JsonTimestamp>,
    /// Number of IP addresses assigned to this interface (read-only).
    pub count_ipaddresses: i64,
    /// Number of FHRP groups for this interface (read-only).
    pub count_fhrp_groups: i64,
    /// Whether a cable or peer link occupies this interface (read-only).
    #[serde(rename = "_occupied")]
    pub occupied: bool,
}

/// Request body for creating or replacing an interface (POST / PUT).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct InterfaceRequest {
    /// Device (ID).
    pub device: i64,
    /// Virtual device context IDs.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub vdcs: Vec<i64>,
    /// Module (ID), if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<i64>,
    /// Interface name.
    pub name: String,
    /// Physical label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Interface type (e.g. `virtual`, `1000base-t`, `10gbase-x-sfpp`).
    #[serde(rename = "type")]
    #[serde(default)]
    pub r#type: String,
    /// Whether this interface is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// Parent interface (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<i64>,
    /// Bridge interface (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bridge: Option<i64>,
    /// LAG interface this member belongs to (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lag: Option<i64>,
    /// Maximum transmission unit in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mtu: Option<i64>,
    /// Primary MAC address (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_mac_address: Option<i64>,
    /// Interface speed in Kbps.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<i64>,
    /// Duplex mode (`half`, `full`, `auto`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duplex: Option<String>,
    /// World Wide Name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wwn: Option<String>,
    /// Whether this interface is used only for out-of-band management.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mgmt_only: Option<bool>,
    /// Short description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 802.1Q mode (`access`, `tagged`, `tagged-all`, `q-in-q`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    /// Wireless RF role (`ap`, `station`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rf_role: Option<String>,
    /// Wireless RF channel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rf_channel: Option<String>,
    /// PoE mode (`pd`, `pse`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poe_mode: Option<String>,
    /// PoE type (e.g. `type1-ieee802.3af`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poe_type: Option<String>,
    /// Wireless channel frequency in MHz.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rf_channel_frequency: Option<f64>,
    /// Wireless channel width in MHz.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rf_channel_width: Option<f64>,
    /// Transmit power in dBm.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_power: Option<i64>,
    /// Untagged (native) VLAN (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub untagged_vlan: Option<i64>,
    /// Tagged VLAN IDs.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tagged_vlans: Vec<i64>,
    /// Q-in-Q service VLAN (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qinq_svlan: Option<i64>,
    /// VLAN translation policy (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vlan_translation_policy: Option<i64>,
    /// Treat as if a cable is connected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mark_connected: Option<bool>,
    /// Wireless LAN IDs.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub wireless_lans: Vec<i64>,
    /// VRF (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vrf: Option<i64>,
    /// Owner (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<i64>,
    /// Tags to attach.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<NestedTagRequest>,
    /// Custom fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<crate::JsonValue>,
}

/// Request body for partially updating an interface (PATCH).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct InterfacePatchRequest {
    /// Device (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device: Option<i64>,
    /// Virtual device context IDs.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub vdcs: Vec<i64>,
    /// Module (ID), if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<i64>,
    /// Interface name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Physical label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Interface type.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    /// Whether this interface is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// Parent interface (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<i64>,
    /// Bridge interface (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bridge: Option<i64>,
    /// LAG interface this member belongs to (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lag: Option<i64>,
    /// Maximum transmission unit in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mtu: Option<i64>,
    /// Primary MAC address (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_mac_address: Option<i64>,
    /// Interface speed in Kbps.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<i64>,
    /// Duplex mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duplex: Option<String>,
    /// World Wide Name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wwn: Option<String>,
    /// Whether this interface is used only for out-of-band management.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mgmt_only: Option<bool>,
    /// Short description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 802.1Q mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    /// Wireless RF role.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rf_role: Option<String>,
    /// Wireless RF channel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rf_channel: Option<String>,
    /// PoE mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poe_mode: Option<String>,
    /// PoE type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poe_type: Option<String>,
    /// Wireless channel frequency in MHz.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rf_channel_frequency: Option<f64>,
    /// Wireless channel width in MHz.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rf_channel_width: Option<f64>,
    /// Transmit power in dBm.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_power: Option<i64>,
    /// Untagged (native) VLAN (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub untagged_vlan: Option<i64>,
    /// Tagged VLAN IDs.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tagged_vlans: Vec<i64>,
    /// Q-in-Q service VLAN (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qinq_svlan: Option<i64>,
    /// VLAN translation policy (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vlan_translation_policy: Option<i64>,
    /// Treat as if a cable is connected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mark_connected: Option<bool>,
    /// Wireless LAN IDs.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub wireless_lans: Vec<i64>,
    /// VRF (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vrf: Option<i64>,
    /// Owner (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<i64>,
    /// Tags to attach.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<NestedTagRequest>,
    /// Custom fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<crate::JsonValue>,
}

// ── MAC address types ────────────────────────────────────────────────────────

/// A MAC address object managed by NetBox (`/api/dcim/mac-addresses/`).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MACAddress {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this MAC address.
    pub url: String,
    /// URL for the detail view.
    pub display_url: String,
    /// Human-readable representation.
    pub display: String,
    /// MAC address in colon-separated hex notation (e.g. `aa:bb:cc:dd:ee:ff`).
    pub mac_address: String,
    /// Content-type of the assigned object (e.g. `dcim.interface`).
    pub assigned_object_type: Option<String>,
    /// Primary key of the assigned object.
    pub assigned_object_id: Option<i64>,
    /// Assigned object (raw JSON — type depends on `assigned_object_type`).
    pub assigned_object: Option<crate::JsonValue>,
    /// Short description.
    #[serde(default)]
    pub description: String,
    /// Owner object.
    pub owner: Option<BriefOwner>,
    /// Freeform comments.
    #[serde(default)]
    pub comments: String,
    /// Tags attached to this MAC address.
    #[serde(default)]
    pub tags: Vec<NestedTag>,
    /// Custom fields.
    pub custom_fields: Option<crate::JsonValue>,
    /// Date/time this object was created.
    pub created: Option<json_ts::JsonTimestamp>,
    /// Date/time this object was last updated.
    pub last_updated: Option<json_ts::JsonTimestamp>,
}

// ── Extras types ─────────────────────────────────────────────────────────────

/// Condensed custom-field choice set as returned in `CustomField` responses.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BriefCustomFieldChoiceSet {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this choice set.
    pub url: String,
    /// Human-readable representation.
    pub display: String,
    /// Choice set name.
    pub name: String,
    /// Short description.
    #[serde(default)]
    pub description: String,
    /// Number of choices in this set.
    #[serde(default)]
    pub choices_count: i64,
}

/// A custom field definition in NetBox.
///
/// Custom fields extend the built-in data model by attaching user-defined
/// attributes to specific object types.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CustomField {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this custom field.
    pub url: String,
    /// URL for the detail view.
    pub display_url: String,
    /// Human-readable representation.
    pub display: String,
    /// List of content types this custom field is assigned to (e.g. `["dcim.site"]`).
    pub object_types: Vec<String>,
    /// The type of data this custom field holds.
    #[serde(rename = "type")]
    pub r#type: StatusValue,
    /// For `object` / `multiobject` types, the related content type (e.g. `dcim.site`).
    pub related_object_type: Option<String>,
    /// Computed data type string (read-only).
    pub data_type: String,
    /// Internal field name.
    pub name: String,
    /// Name of the field as displayed to users.
    #[serde(default)]
    pub label: String,
    /// Custom fields within the same group will be displayed together.
    #[serde(default)]
    pub group_name: String,
    /// Short description.
    #[serde(default)]
    pub description: String,
    /// Whether this field is required when creating or editing objects.
    #[serde(default)]
    pub required: bool,
    /// Whether the value of this field must be unique for the assigned object.
    #[serde(default)]
    pub unique: bool,
    /// Weighting for search. Lower values are considered more important. Zero means ignored.
    #[serde(default)]
    pub search_weight: i64,
    /// Loose matches any instance of a given string; exact matches the entire field.
    pub filter_logic: Option<StatusValue>,
    /// Specifies whether the custom field is displayed in the UI.
    pub ui_visible: Option<StatusValue>,
    /// Specifies whether the custom field value can be edited in the UI.
    pub ui_editable: Option<StatusValue>,
    /// Whether this value is replicated when cloning objects.
    #[serde(default)]
    pub is_cloneable: bool,
    /// Default value for the field (must be a JSON value).
    pub default: Option<crate::JsonValue>,
    /// Filter the object selection choices using a query_params dict (a JSON value).
    pub related_object_filter: Option<crate::JsonValue>,
    /// Fields with higher weights appear lower in a form.
    #[serde(default)]
    pub weight: i64,
    /// Minimum allowed value (for numeric fields).
    pub validation_minimum: Option<f64>,
    /// Maximum allowed value (for numeric fields).
    pub validation_maximum: Option<f64>,
    /// Regular expression to enforce on text field values.
    #[serde(default)]
    pub validation_regex: String,
    /// Choice set associated with this field (for `select` / `multiselect` types).
    pub choice_set: Option<BriefCustomFieldChoiceSet>,
    /// Owner object.
    pub owner: Option<BriefOwner>,
    /// Freeform comments.
    #[serde(default)]
    pub comments: String,
    /// Date/time this object was created.
    pub created: Option<json_ts::JsonTimestamp>,
    /// Date/time this object was last updated.
    pub last_updated: Option<json_ts::JsonTimestamp>,
}

/// Request body for creating or replacing a custom field (POST / PUT).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CustomFieldRequest {
    /// List of content types this custom field applies to (e.g. `["dcim.site"]`).
    pub object_types: Vec<String>,
    /// The type of data this custom field holds (e.g. `text`, `integer`, `select`).
    #[serde(rename = "type")]
    pub r#type: String,
    /// Internal field name.
    pub name: String,
    /// For `object` / `multiobject` types, the related content type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_object_type: Option<String>,
    /// Name of the field as displayed to users.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Custom fields within the same group will be displayed together.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_name: Option<String>,
    /// Short description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether this field is required when creating or editing objects.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    /// Whether the value of this field must be unique for the assigned object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unique: Option<bool>,
    /// Weighting for search. Lower values are considered more important. Zero means ignored.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_weight: Option<i64>,
    /// Filter logic: `disabled`, `loose`, or `exact`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_logic: Option<String>,
    /// UI visibility: `always`, `if-set`, or `hidden`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_visible: Option<String>,
    /// UI editability: `yes`, `no`, or `hidden`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_editable: Option<String>,
    /// Whether this value is replicated when cloning objects.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_cloneable: Option<bool>,
    /// Default value for the field (must be a JSON value).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<crate::JsonValue>,
    /// Filter the object selection choices using a query_params dict (a JSON value).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_object_filter: Option<crate::JsonValue>,
    /// Display weight — fields with higher weights appear lower in a form.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<i64>,
    /// Minimum allowed value (for numeric fields).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_minimum: Option<f64>,
    /// Maximum allowed value (for numeric fields).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_maximum: Option<f64>,
    /// Regular expression to enforce on text field values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_regex: Option<String>,
    /// Choice set ID (for `select` / `multiselect` types).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub choice_set: Option<i64>,
    /// Owner ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<i64>,
    /// Freeform comments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
}

/// Request body for partially updating a custom field (PATCH).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CustomFieldPatchRequest {
    /// List of content types this custom field applies to (e.g. `["dcim.site"]`).
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub object_types: Vec<String>,
    /// The type of data this custom field holds (e.g. `text`, `integer`, `select`).
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    /// Internal field name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// For `object` / `multiobject` types, the related content type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_object_type: Option<String>,
    /// Name of the field as displayed to users.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Custom fields within the same group will be displayed together.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_name: Option<String>,
    /// Short description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether this field is required when creating or editing objects.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    /// Whether the value of this field must be unique for the assigned object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unique: Option<bool>,
    /// Weighting for search. Lower values are considered more important. Zero means ignored.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_weight: Option<i64>,
    /// Filter logic: `disabled`, `loose`, or `exact`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_logic: Option<String>,
    /// UI visibility: `always`, `if-set`, or `hidden`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_visible: Option<String>,
    /// UI editability: `yes`, `no`, or `hidden`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_editable: Option<String>,
    /// Whether this value is replicated when cloning objects.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_cloneable: Option<bool>,
    /// Default value for the field (must be a JSON value).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<crate::JsonValue>,
    /// Filter the object selection choices using a query_params dict (a JSON value).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_object_filter: Option<crate::JsonValue>,
    /// Display weight — fields with higher weights appear lower in a form.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<i64>,
    /// Minimum allowed value (for numeric fields).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_minimum: Option<f64>,
    /// Maximum allowed value (for numeric fields).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_maximum: Option<f64>,
    /// Regular expression to enforce on text field values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation_regex: Option<String>,
    /// Choice set ID (for `select` / `multiselect` types).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub choice_set: Option<i64>,
    /// Owner ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<i64>,
    /// Freeform comments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
}

/// Request body for creating or replacing a MAC address (POST / PUT).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct MACAddressRequest {
    /// MAC address in colon-separated hex notation (e.g. `aa:bb:cc:dd:ee:ff`).
    pub mac_address: String,
    /// Content-type of the assigned object (e.g. `dcim.interface`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_object_type: Option<String>,
    /// Primary key of the assigned object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_object_id: Option<i64>,
    /// Short description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Owner (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<i64>,
    /// Freeform comments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
    /// Tags to attach.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<NestedTagRequest>,
    /// Custom fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<crate::JsonValue>,
}

/// Request body for partially updating a MAC address (PATCH).
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct MACAddressPatchRequest {
    /// MAC address in colon-separated hex notation (e.g. `aa:bb:cc:dd:ee:ff`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mac_address: Option<String>,
    /// Content-type of the assigned object (e.g. `dcim.interface`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_object_type: Option<String>,
    /// Primary key of the assigned object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_object_id: Option<i64>,
    /// Short description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Owner (ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<i64>,
    /// Freeform comments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<String>,
    /// Tags to attach.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<NestedTagRequest>,
    /// Custom fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<crate::JsonValue>,
}

// ── Image attachments ────────────────────────────────────────────────────────

/// An image file attached to a NetBox object.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImageAttachment {
    /// Unique numeric ID.
    pub id: i64,
    /// Canonical URL for this image attachment.
    pub url: String,
    /// Human-readable representation.
    pub display: String,
    /// Content-type of the parent object (e.g. `dcim.site`).
    pub object_type: String,
    /// Primary key of the parent object.
    pub object_id: i64,
    /// The parent object. Exact shape varies by content type.
    pub parent: Option<crate::JsonValue>,
    /// Optional display name for this attachment.
    #[serde(default)]
    pub name: String,
    /// URL of the stored image.
    pub image: String,
    /// Short description.
    #[serde(default)]
    pub description: String,
    /// Pixel height of the image.
    pub image_height: i64,
    /// Pixel width of the image.
    pub image_width: i64,
    /// Date/time this object was created.
    pub created: Option<json_ts::JsonTimestamp>,
    /// Date/time this object was last updated.
    pub last_updated: Option<json_ts::JsonTimestamp>,
}

/// Request body for creating or replacing an image attachment (POST / PUT).
///
/// Image data is uploaded as multipart form data; use `image_filename` to supply
/// the name reported to the server (e.g. `"photo.png"`).
#[derive(Debug, Clone)]
pub struct ImageAttachmentUpload {
    /// Content-type of the parent object (e.g. `dcim.site`).
    pub object_type: String,
    /// Primary key of the parent object.
    pub object_id: i64,
    /// Raw image bytes to upload.
    pub image: Vec<u8>,
    /// File name reported in the multipart upload (e.g. `"photo.png"`).
    pub image_filename: String,
    /// Optional display name for this attachment.
    pub name: Option<String>,
    /// Short description.
    pub description: Option<String>,
}

/// Request body for partially updating an image attachment (PATCH).
///
/// Only set the fields you want to change. Supply `image` as `Some((bytes, filename))`
/// to replace the stored image; leave it `None` to keep the existing one.
#[derive(Debug, Clone, Default)]
pub struct ImageAttachmentPatchUpload {
    /// Content-type of the parent object (e.g. `dcim.site`).
    pub object_type: Option<String>,
    /// Primary key of the parent object.
    pub object_id: Option<i64>,
    /// Replacement image as `(bytes, filename)`.
    pub image: Option<(Vec<u8>, String)>,
    /// Optional display name for this attachment.
    pub name: Option<String>,
    /// Short description.
    pub description: Option<String>,
}
