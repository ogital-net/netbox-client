//! Methods for the `ipam` tag of the NetBox REST API.
//!
//! # Example
//!
//! ```no_run
//! # async fn example() -> netbox_client::Result<()> {
//! use futures_util::TryStreamExt as _;
//! use netbox_client::ipam::{IpAddressFilter, PrefixFilter};
//!
//! let client = netbox_client::NetboxClient::new("https://netbox.example.com", "abc123")?;
//!
//! // Stream all active IP addresses
//! let filter = IpAddressFilter { status: vec!["active".into()], ..Default::default() };
//! let ips: Vec<_> = client.ip_addresses(&filter).try_collect().await?;
//! println!("Found {} active IP addresses", ips.len());
//!
//! // Stream all active prefixes in a specific VRF
//! let filter = PrefixFilter { vrf_id: vec![1], ..Default::default() };
//! let prefixes: Vec<_> = client.prefixes(&filter).try_collect().await?;
//! println!("Found {} prefixes", prefixes.len());
//! # Ok(()) }
//! ```

// Page sizes from NetBox are always far smaller than 2^32 items.
#![allow(clippy::cast_possible_truncation)]

use std::collections::VecDeque;

use futures_core::stream::BoxStream;
use futures_util::stream::try_unfold;

use crate::RequestBuilderExt as _;
use crate::{
    Aggregate, AggregatePatchRequest, AggregateRequest, IpAddress, IpAddressPatchRequest,
    IpAddressRequest, NetboxClient, Paginated, Prefix, PrefixPatchRequest, PrefixRequest, Role,
    RolePatchRequest, RoleRequest,
};

const PAGE_SIZE: u32 = 50;

// ── Filter types ─────────────────────────────────────────────────────────────

/// Filters for the [`NetboxClient::ip_addresses_list`] / [`NetboxClient::ip_addresses`] endpoints.
#[derive(Debug, Clone, Default)]
pub struct IpAddressFilter {
    /// Free-text search.
    pub q: Option<String>,
    /// Limit results to these IDs.
    pub id: Vec<i64>,
    /// Filter by address (exact, e.g. `192.0.2.1/24`).
    pub address: Vec<String>,
    /// Filter by IP family (4 or 6).
    pub family: Option<i64>,
    /// Filter by status (`active`, `reserved`, `deprecated`, `dhcp`, `slaac`).
    pub status: Vec<String>,
    /// Filter by role (`loopback`, `secondary`, `anycast`, `vip`, etc.).
    pub role: Vec<String>,
    /// Filter by VRF slug.
    pub vrf: Vec<String>,
    /// Filter by VRF ID.
    pub vrf_id: Vec<i64>,
    /// Filter by tenant slug.
    pub tenant: Vec<String>,
    /// Filter by tenant ID.
    pub tenant_id: Vec<i64>,
    /// Filter by DNS name (exact).
    pub dns_name: Vec<String>,
    /// Filter to addresses present in VRF slug.
    pub present_in_vrf: Option<String>,
    /// Filter to addresses present in VRF ID.
    pub present_in_vrf_id: Option<i64>,
    /// Filter by tag.
    pub tag: Vec<String>,
}

impl IpAddressFilter {
    fn as_query(&self) -> Vec<(String, String)> {
        let mut p = Vec::new();
        if let Some(q) = &self.q {
            p.push(("q".into(), q.clone()));
        }
        for v in &self.id {
            p.push(("id".into(), v.to_string()));
        }
        for v in &self.address {
            p.push(("address".into(), v.clone()));
        }
        if let Some(f) = self.family {
            p.push(("family".into(), f.to_string()));
        }
        for v in &self.status {
            p.push(("status".into(), v.clone()));
        }
        for v in &self.role {
            p.push(("role".into(), v.clone()));
        }
        for v in &self.vrf {
            p.push(("vrf".into(), v.clone()));
        }
        for v in &self.vrf_id {
            p.push(("vrf_id".into(), v.to_string()));
        }
        for v in &self.tenant {
            p.push(("tenant".into(), v.clone()));
        }
        for v in &self.tenant_id {
            p.push(("tenant_id".into(), v.to_string()));
        }
        for v in &self.dns_name {
            p.push(("dns_name".into(), v.clone()));
        }
        if let Some(v) = &self.present_in_vrf {
            p.push(("present_in_vrf".into(), v.clone()));
        }
        if let Some(v) = self.present_in_vrf_id {
            p.push(("present_in_vrf_id".into(), v.to_string()));
        }
        for v in &self.tag {
            p.push(("tag".into(), v.clone()));
        }
        p
    }
}

/// Filters for the [`NetboxClient::prefixes_list`] / [`NetboxClient::prefixes`] endpoints.
#[derive(Debug, Clone, Default)]
pub struct PrefixFilter {
    /// Free-text search.
    pub q: Option<String>,
    /// Limit results to these IDs.
    pub id: Vec<i64>,
    /// Filter by prefix (exact, e.g. `192.0.2.0/24`).
    pub prefix: Vec<String>,
    /// Filter by IP family (4 or 6).
    pub family: Option<i64>,
    /// Filter by status (`container`, `active`, `reserved`, `deprecated`).
    pub status: Vec<String>,
    /// Filter by role slug.
    pub role: Vec<String>,
    /// Filter by role ID.
    pub role_id: Vec<i64>,
    /// Filter by VRF slug.
    pub vrf: Vec<String>,
    /// Filter by VRF ID.
    pub vrf_id: Vec<i64>,
    /// Filter by tenant slug.
    pub tenant: Vec<String>,
    /// Filter by tenant ID.
    pub tenant_id: Vec<i64>,
    /// Filter to prefixes present in VRF slug.
    pub present_in_vrf: Option<String>,
    /// Filter to prefixes present in VRF ID.
    pub present_in_vrf_id: Option<i64>,
    /// Filter by tag.
    pub tag: Vec<String>,
}

impl PrefixFilter {
    fn as_query(&self) -> Vec<(String, String)> {
        let mut p = Vec::new();
        if let Some(q) = &self.q {
            p.push(("q".into(), q.clone()));
        }
        for v in &self.id {
            p.push(("id".into(), v.to_string()));
        }
        for v in &self.prefix {
            p.push(("prefix".into(), v.clone()));
        }
        if let Some(f) = self.family {
            p.push(("family".into(), f.to_string()));
        }
        for v in &self.status {
            p.push(("status".into(), v.clone()));
        }
        for v in &self.role {
            p.push(("role".into(), v.clone()));
        }
        for v in &self.role_id {
            p.push(("role_id".into(), v.to_string()));
        }
        for v in &self.vrf {
            p.push(("vrf".into(), v.clone()));
        }
        for v in &self.vrf_id {
            p.push(("vrf_id".into(), v.to_string()));
        }
        for v in &self.tenant {
            p.push(("tenant".into(), v.clone()));
        }
        for v in &self.tenant_id {
            p.push(("tenant_id".into(), v.to_string()));
        }
        if let Some(v) = &self.present_in_vrf {
            p.push(("present_in_vrf".into(), v.clone()));
        }
        if let Some(v) = self.present_in_vrf_id {
            p.push(("present_in_vrf_id".into(), v.to_string()));
        }
        for v in &self.tag {
            p.push(("tag".into(), v.clone()));
        }
        p
    }
}

/// Filters for the [`NetboxClient::aggregates_list`] / [`NetboxClient::aggregates`] endpoints.
#[derive(Debug, Clone, Default)]
pub struct AggregateFilter {
    /// Free-text search.
    pub q: Option<String>,
    /// Limit results to these IDs.
    pub id: Vec<i64>,
    /// Filter by prefix (exact, e.g. `10.0.0.0/8`).
    pub prefix: Vec<String>,
    /// Filter by IP family (4 or 6).
    pub family: Option<i64>,
    /// Filter by RIR slug.
    pub rir: Vec<String>,
    /// Filter by RIR ID.
    pub rir_id: Vec<i64>,
    /// Filter by tenant slug.
    pub tenant: Vec<String>,
    /// Filter by tenant ID.
    pub tenant_id: Vec<i64>,
    /// Filter by tag.
    pub tag: Vec<String>,
}

impl AggregateFilter {
    fn as_query(&self) -> Vec<(String, String)> {
        let mut p = Vec::new();
        if let Some(q) = &self.q {
            p.push(("q".into(), q.clone()));
        }
        for v in &self.id {
            p.push(("id".into(), v.to_string()));
        }
        for v in &self.prefix {
            p.push(("prefix".into(), v.clone()));
        }
        if let Some(f) = self.family {
            p.push(("family".into(), f.to_string()));
        }
        for v in &self.rir {
            p.push(("rir".into(), v.clone()));
        }
        for v in &self.rir_id {
            p.push(("rir_id".into(), v.to_string()));
        }
        for v in &self.tenant {
            p.push(("tenant".into(), v.clone()));
        }
        for v in &self.tenant_id {
            p.push(("tenant_id".into(), v.to_string()));
        }
        for v in &self.tag {
            p.push(("tag".into(), v.clone()));
        }
        p
    }
}

/// Filters for the [`NetboxClient::roles_list`] / [`NetboxClient::roles`] endpoints.
#[derive(Debug, Clone, Default)]
pub struct RoleFilter {
    /// Free-text search.
    pub q: Option<String>,
    /// Limit results to these IDs.
    pub id: Vec<i64>,
    /// Filter by name.
    pub name: Vec<String>,
    /// Filter by slug.
    pub slug: Vec<String>,
    /// Filter by tag.
    pub tag: Vec<String>,
}

impl RoleFilter {
    fn as_query(&self) -> Vec<(String, String)> {
        let mut p = Vec::new();
        if let Some(q) = &self.q {
            p.push(("q".into(), q.clone()));
        }
        for v in &self.id {
            p.push(("id".into(), v.to_string()));
        }
        for v in &self.name {
            p.push(("name".into(), v.clone()));
        }
        for v in &self.slug {
            p.push(("slug".into(), v.clone()));
        }
        for v in &self.tag {
            p.push(("tag".into(), v.clone()));
        }
        p
    }
}

// ── NetboxClient implementation ───────────────────────────────────────────────

impl NetboxClient {
    // ── IP Addresses ──────────────────────────────────────────────────────────

    /// Returns a single page of IP addresses.
    ///
    /// Maps to `GET /api/ipam/ip-addresses/`
    /// (`operationId`: `ipam_ip_addresses_list`).
    ///
    /// # Errors
    ///
    /// Returns an error on HTTP or deserialization failure.
    pub async fn ip_addresses_list(
        &self,
        limit: u32,
        offset: u32,
        filter: &IpAddressFilter,
    ) -> crate::Result<Paginated<IpAddress>> {
        let url = format!("{}/api/ipam/ip-addresses/", self.base_url);
        let mut query = filter.as_query();
        query.push(("limit".into(), limit.to_string()));
        query.push(("offset".into(), offset.to_string()));
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .query(&query)
            .send_json::<Paginated<IpAddress>>()
            .await
    }

    /// Streams all IP addresses matching `filter`, auto-paginating.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> netbox_client::Result<()> {
    /// use futures_util::TryStreamExt as _;
    /// use netbox_client::ipam::IpAddressFilter;
    /// let client = netbox_client::NetboxClient::new("https://netbox.example.com", "token")?;
    /// let ips: Vec<_> = client.ip_addresses(&IpAddressFilter::default()).try_collect().await?;
    /// # Ok(()) }
    /// ```
    #[must_use]
    pub fn ip_addresses<'a>(
        &'a self,
        filter: &'a IpAddressFilter,
    ) -> BoxStream<'a, crate::Result<IpAddress>> {
        Box::pin(try_unfold(
            (Some(0u32), VecDeque::<IpAddress>::new()),
            move |(next_offset, mut buf)| async move {
                if let Some(item) = buf.pop_front() {
                    return Ok(Some((item, (next_offset, buf))));
                }
                let Some(offset) = next_offset else {
                    return Ok(None);
                };
                let page = self.ip_addresses_list(PAGE_SIZE, offset, filter).await?;
                let new_next = page
                    .next
                    .is_some()
                    .then_some(offset + page.results.len() as u32);
                let mut buf: VecDeque<IpAddress> = page.results.into_iter().collect();
                match buf.pop_front() {
                    Some(item) => Ok(Some((item, (new_next, buf)))),
                    None => Ok(None),
                }
            },
        ))
    }

    /// Returns a single IP address by ID.
    ///
    /// Maps to `GET /api/ipam/ip-addresses/{id}/`
    /// (`operationId`: `ipam_ip_addresses_retrieve`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 404 if the IP address does not exist.
    pub async fn ip_address(&self, id: i64) -> crate::Result<IpAddress> {
        let url = format!("{}/api/ipam/ip-addresses/{id}/", self.base_url);
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .send_json::<IpAddress>()
            .await
    }

    /// Creates a new IP address.
    ///
    /// Maps to `POST /api/ipam/ip-addresses/`
    /// (`operationId`: `ipam_ip_addresses_create`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn ip_address_create(&self, body: &IpAddressRequest) -> crate::Result<IpAddress> {
        let url = format!("{}/api/ipam/ip-addresses/", self.base_url);
        self.http
            .post(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<IpAddress>()
            .await
    }

    /// Replaces an IP address (full update).
    ///
    /// Maps to `PUT /api/ipam/ip-addresses/{id}/`
    /// (`operationId`: `ipam_ip_addresses_update`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn ip_address_update(
        &self,
        id: i64,
        body: &IpAddressRequest,
    ) -> crate::Result<IpAddress> {
        let url = format!("{}/api/ipam/ip-addresses/{id}/", self.base_url);
        self.http
            .put(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<IpAddress>()
            .await
    }

    /// Partially updates an IP address.
    ///
    /// Maps to `PATCH /api/ipam/ip-addresses/{id}/`
    /// (`operationId`: `ipam_ip_addresses_partial_update`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn ip_address_patch(
        &self,
        id: i64,
        body: &IpAddressPatchRequest,
    ) -> crate::Result<IpAddress> {
        let url = format!("{}/api/ipam/ip-addresses/{id}/", self.base_url);
        self.http
            .patch(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<IpAddress>()
            .await
    }

    /// Deletes an IP address.
    ///
    /// Maps to `DELETE /api/ipam/ip-addresses/{id}/`
    /// (`operationId`: `ipam_ip_addresses_destroy`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 404 if the IP address does not exist.
    pub async fn ip_address_delete(&self, id: i64) -> crate::Result<()> {
        let url = format!("{}/api/ipam/ip-addresses/{id}/", self.base_url);
        crate::delete_no_content(&self.http, &url, &self.token).await
    }

    // ── Prefixes ──────────────────────────────────────────────────────────────

    /// Returns a single page of prefixes.
    ///
    /// Maps to `GET /api/ipam/prefixes/`
    /// (`operationId`: `ipam_prefixes_list`).
    ///
    /// # Errors
    ///
    /// Returns an error on HTTP or deserialization failure.
    pub async fn prefixes_list(
        &self,
        limit: u32,
        offset: u32,
        filter: &PrefixFilter,
    ) -> crate::Result<Paginated<Prefix>> {
        let url = format!("{}/api/ipam/prefixes/", self.base_url);
        let mut query = filter.as_query();
        query.push(("limit".into(), limit.to_string()));
        query.push(("offset".into(), offset.to_string()));
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .query(&query)
            .send_json::<Paginated<Prefix>>()
            .await
    }

    /// Streams all prefixes matching `filter`, auto-paginating.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> netbox_client::Result<()> {
    /// use futures_util::TryStreamExt as _;
    /// use netbox_client::ipam::PrefixFilter;
    /// let client = netbox_client::NetboxClient::new("https://netbox.example.com", "token")?;
    /// let prefixes: Vec<_> = client.prefixes(&PrefixFilter::default()).try_collect().await?;
    /// # Ok(()) }
    /// ```
    #[must_use]
    pub fn prefixes<'a>(
        &'a self,
        filter: &'a PrefixFilter,
    ) -> BoxStream<'a, crate::Result<Prefix>> {
        Box::pin(try_unfold(
            (Some(0u32), VecDeque::<Prefix>::new()),
            move |(next_offset, mut buf)| async move {
                if let Some(item) = buf.pop_front() {
                    return Ok(Some((item, (next_offset, buf))));
                }
                let Some(offset) = next_offset else {
                    return Ok(None);
                };
                let page = self.prefixes_list(PAGE_SIZE, offset, filter).await?;
                let new_next = page
                    .next
                    .is_some()
                    .then_some(offset + page.results.len() as u32);
                let mut buf: VecDeque<Prefix> = page.results.into_iter().collect();
                match buf.pop_front() {
                    Some(item) => Ok(Some((item, (new_next, buf)))),
                    None => Ok(None),
                }
            },
        ))
    }

    /// Returns a single prefix by ID.
    ///
    /// Maps to `GET /api/ipam/prefixes/{id}/`
    /// (`operationId`: `ipam_prefixes_retrieve`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 404 if the prefix does not exist.
    pub async fn prefix(&self, id: i64) -> crate::Result<Prefix> {
        let url = format!("{}/api/ipam/prefixes/{id}/", self.base_url);
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .send_json::<Prefix>()
            .await
    }

    /// Creates a new prefix.
    ///
    /// Maps to `POST /api/ipam/prefixes/`
    /// (`operationId`: `ipam_prefixes_create`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn prefix_create(&self, body: &PrefixRequest) -> crate::Result<Prefix> {
        let url = format!("{}/api/ipam/prefixes/", self.base_url);
        self.http
            .post(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<Prefix>()
            .await
    }

    /// Replaces a prefix (full update).
    ///
    /// Maps to `PUT /api/ipam/prefixes/{id}/`
    /// (`operationId`: `ipam_prefixes_update`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn prefix_update(&self, id: i64, body: &PrefixRequest) -> crate::Result<Prefix> {
        let url = format!("{}/api/ipam/prefixes/{id}/", self.base_url);
        self.http
            .put(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<Prefix>()
            .await
    }

    /// Partially updates a prefix.
    ///
    /// Maps to `PATCH /api/ipam/prefixes/{id}/`
    /// (`operationId`: `ipam_prefixes_partial_update`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn prefix_patch(&self, id: i64, body: &PrefixPatchRequest) -> crate::Result<Prefix> {
        let url = format!("{}/api/ipam/prefixes/{id}/", self.base_url);
        self.http
            .patch(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<Prefix>()
            .await
    }

    /// Deletes a prefix.
    ///
    /// Maps to `DELETE /api/ipam/prefixes/{id}/`
    /// (`operationId`: `ipam_prefixes_destroy`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 404 if the prefix does not exist.
    pub async fn prefix_delete(&self, id: i64) -> crate::Result<()> {
        let url = format!("{}/api/ipam/prefixes/{id}/", self.base_url);
        crate::delete_no_content(&self.http, &url, &self.token).await
    }

    // ── Aggregates ────────────────────────────────────────────────────────────

    /// Returns a single page of aggregates.
    ///
    /// Maps to `GET /api/ipam/aggregates/`
    /// (`operationId`: `ipam_aggregates_list`).
    ///
    /// # Errors
    ///
    /// Returns an error on HTTP or deserialization failure.
    pub async fn aggregates_list(
        &self,
        limit: u32,
        offset: u32,
        filter: &AggregateFilter,
    ) -> crate::Result<Paginated<Aggregate>> {
        let url = format!("{}/api/ipam/aggregates/", self.base_url);
        let mut query = filter.as_query();
        query.push(("limit".into(), limit.to_string()));
        query.push(("offset".into(), offset.to_string()));
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .query(&query)
            .send_json::<Paginated<Aggregate>>()
            .await
    }

    /// Streams all aggregates matching `filter`, auto-paginating.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> netbox_client::Result<()> {
    /// use futures_util::TryStreamExt as _;
    /// use netbox_client::ipam::AggregateFilter;
    /// let client = netbox_client::NetboxClient::new("https://netbox.example.com", "token")?;
    /// let aggs: Vec<_> = client.aggregates(&AggregateFilter::default()).try_collect().await?;
    /// # Ok(()) }
    /// ```
    #[must_use]
    pub fn aggregates<'a>(
        &'a self,
        filter: &'a AggregateFilter,
    ) -> BoxStream<'a, crate::Result<Aggregate>> {
        Box::pin(try_unfold(
            (Some(0u32), VecDeque::<Aggregate>::new()),
            move |(next_offset, mut buf)| async move {
                if let Some(item) = buf.pop_front() {
                    return Ok(Some((item, (next_offset, buf))));
                }
                let Some(offset) = next_offset else {
                    return Ok(None);
                };
                let page = self.aggregates_list(PAGE_SIZE, offset, filter).await?;
                let new_next = page
                    .next
                    .is_some()
                    .then_some(offset + page.results.len() as u32);
                let mut buf: VecDeque<Aggregate> = page.results.into_iter().collect();
                match buf.pop_front() {
                    Some(item) => Ok(Some((item, (new_next, buf)))),
                    None => Ok(None),
                }
            },
        ))
    }

    /// Returns a single aggregate by ID.
    ///
    /// Maps to `GET /api/ipam/aggregates/{id}/`
    /// (`operationId`: `ipam_aggregates_retrieve`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 404 if the aggregate does not exist.
    pub async fn aggregate(&self, id: i64) -> crate::Result<Aggregate> {
        let url = format!("{}/api/ipam/aggregates/{id}/", self.base_url);
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .send_json::<Aggregate>()
            .await
    }

    /// Creates a new aggregate.
    ///
    /// Maps to `POST /api/ipam/aggregates/`
    /// (`operationId`: `ipam_aggregates_create`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn aggregate_create(&self, body: &AggregateRequest) -> crate::Result<Aggregate> {
        let url = format!("{}/api/ipam/aggregates/", self.base_url);
        self.http
            .post(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<Aggregate>()
            .await
    }

    /// Replaces an aggregate (full update).
    ///
    /// Maps to `PUT /api/ipam/aggregates/{id}/`
    /// (`operationId`: `ipam_aggregates_update`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn aggregate_update(
        &self,
        id: i64,
        body: &AggregateRequest,
    ) -> crate::Result<Aggregate> {
        let url = format!("{}/api/ipam/aggregates/{id}/", self.base_url);
        self.http
            .put(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<Aggregate>()
            .await
    }

    /// Partially updates an aggregate.
    ///
    /// Maps to `PATCH /api/ipam/aggregates/{id}/`
    /// (`operationId`: `ipam_aggregates_partial_update`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn aggregate_patch(
        &self,
        id: i64,
        body: &AggregatePatchRequest,
    ) -> crate::Result<Aggregate> {
        let url = format!("{}/api/ipam/aggregates/{id}/", self.base_url);
        self.http
            .patch(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<Aggregate>()
            .await
    }

    /// Deletes an aggregate.
    ///
    /// Maps to `DELETE /api/ipam/aggregates/{id}/`
    /// (`operationId`: `ipam_aggregates_destroy`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 404 if the aggregate does not exist.
    pub async fn aggregate_delete(&self, id: i64) -> crate::Result<()> {
        let url = format!("{}/api/ipam/aggregates/{id}/", self.base_url);
        crate::delete_no_content(&self.http, &url, &self.token).await
    }

    // ── Roles ─────────────────────────────────────────────────────────────────

    /// Returns a single page of IPAM roles.
    ///
    /// Maps to `GET /api/ipam/roles/`
    /// (`operationId`: `ipam_roles_list`).
    ///
    /// # Errors
    ///
    /// Returns an error on HTTP or deserialization failure.
    pub async fn roles_list(
        &self,
        limit: u32,
        offset: u32,
        filter: &RoleFilter,
    ) -> crate::Result<Paginated<Role>> {
        let url = format!("{}/api/ipam/roles/", self.base_url);
        let mut query = filter.as_query();
        query.push(("limit".into(), limit.to_string()));
        query.push(("offset".into(), offset.to_string()));
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .query(&query)
            .send_json::<Paginated<Role>>()
            .await
    }

    /// Streams all IPAM roles matching `filter`, auto-paginating.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> netbox_client::Result<()> {
    /// use futures_util::TryStreamExt as _;
    /// use netbox_client::ipam::RoleFilter;
    /// let client = netbox_client::NetboxClient::new("https://netbox.example.com", "token")?;
    /// let roles: Vec<_> = client.roles(&RoleFilter::default()).try_collect().await?;
    /// # Ok(()) }
    /// ```
    #[must_use]
    pub fn roles<'a>(&'a self, filter: &'a RoleFilter) -> BoxStream<'a, crate::Result<Role>> {
        Box::pin(try_unfold(
            (Some(0u32), VecDeque::<Role>::new()),
            move |(next_offset, mut buf)| async move {
                if let Some(item) = buf.pop_front() {
                    return Ok(Some((item, (next_offset, buf))));
                }
                let Some(offset) = next_offset else {
                    return Ok(None);
                };
                let page = self.roles_list(PAGE_SIZE, offset, filter).await?;
                let new_next = page
                    .next
                    .is_some()
                    .then_some(offset + page.results.len() as u32);
                let mut buf: VecDeque<Role> = page.results.into_iter().collect();
                match buf.pop_front() {
                    Some(item) => Ok(Some((item, (new_next, buf)))),
                    None => Ok(None),
                }
            },
        ))
    }

    /// Returns a single IPAM role by ID.
    ///
    /// Maps to `GET /api/ipam/roles/{id}/`
    /// (`operationId`: `ipam_roles_retrieve`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 404 if the role does not exist.
    pub async fn role(&self, id: i64) -> crate::Result<Role> {
        let url = format!("{}/api/ipam/roles/{id}/", self.base_url);
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .send_json::<Role>()
            .await
    }

    /// Creates a new IPAM role.
    ///
    /// Maps to `POST /api/ipam/roles/`
    /// (`operationId`: `ipam_roles_create`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn role_create(&self, body: &RoleRequest) -> crate::Result<Role> {
        let url = format!("{}/api/ipam/roles/", self.base_url);
        self.http
            .post(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<Role>()
            .await
    }

    /// Replaces an IPAM role (full update).
    ///
    /// Maps to `PUT /api/ipam/roles/{id}/`
    /// (`operationId`: `ipam_roles_update`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn role_update(&self, id: i64, body: &RoleRequest) -> crate::Result<Role> {
        let url = format!("{}/api/ipam/roles/{id}/", self.base_url);
        self.http
            .put(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<Role>()
            .await
    }

    /// Partially updates an IPAM role.
    ///
    /// Maps to `PATCH /api/ipam/roles/{id}/`
    /// (`operationId`: `ipam_roles_partial_update`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn role_patch(&self, id: i64, body: &RolePatchRequest) -> crate::Result<Role> {
        let url = format!("{}/api/ipam/roles/{id}/", self.base_url);
        self.http
            .patch(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<Role>()
            .await
    }

    /// Deletes an IPAM role.
    ///
    /// Maps to `DELETE /api/ipam/roles/{id}/`
    /// (`operationId`: `ipam_roles_destroy`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 404 if the role does not exist.
    pub async fn role_delete(&self, id: i64) -> crate::Result<()> {
        let url = format!("{}/api/ipam/roles/{id}/", self.base_url);
        crate::delete_no_content(&self.http, &url, &self.token).await
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{
        matchers::{header, method, path, query_param},
        Mock, MockServer, ResponseTemplate,
    };

    // ── helpers ───────────────────────────────────────────────────────────────

    fn ip_address_json(id: i64, address: &str) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "url": format!("https://netbox.example.com/api/ipam/ip-addresses/{id}/"),
            "display_url": format!("https://netbox.example.com/ipam/ip-addresses/{id}/"),
            "display": address,
            "family": {"value": 4, "label": "IPv4"},
            "address": address,
            "vrf": null,
            "tenant": null,
            "status": {"value": "active", "label": "Active"},
            "role": null,
            "assigned_object_type": null,
            "assigned_object_id": null,
            "assigned_object": null,
            "nat_inside": null,
            "nat_outside": [],
            "dns_name": null,
            "description": "",
            "owner": null,
            "comments": "",
            "tags": [],
            "custom_fields": {},
            "created": "2024-01-01T00:00:00Z",
            "last_updated": "2024-01-01T00:00:00Z"
        })
    }

    fn prefix_json(id: i64, prefix: &str) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "url": format!("https://netbox.example.com/api/ipam/prefixes/{id}/"),
            "display_url": format!("https://netbox.example.com/ipam/prefixes/{id}/"),
            "display": prefix,
            "family": {"value": 4, "label": "IPv4"},
            "prefix": prefix,
            "vrf": null,
            "scope_type": null,
            "scope_id": null,
            "scope": null,
            "tenant": null,
            "vlan": null,
            "status": {"value": "active", "label": "Active"},
            "role": null,
            "is_pool": false,
            "mark_utilized": false,
            "description": "",
            "owner": null,
            "comments": "",
            "tags": [],
            "custom_fields": {},
            "created": "2024-01-01T00:00:00Z",
            "last_updated": "2024-01-01T00:00:00Z",
            "children": 0,
            "_depth": 0
        })
    }

    fn aggregate_json(id: i64, prefix: &str) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "url": format!("https://netbox.example.com/api/ipam/aggregates/{id}/"),
            "display_url": format!("https://netbox.example.com/ipam/aggregates/{id}/"),
            "display": prefix,
            "family": {"value": 4, "label": "IPv4"},
            "prefix": prefix,
            "rir": {
                "id": 1,
                "url": "https://netbox.example.com/api/ipam/rirs/1/",
                "display": "ARIN",
                "name": "ARIN",
                "slug": "arin",
                "description": ""
            },
            "tenant": null,
            "date_added": null,
            "description": "",
            "owner": null,
            "comments": "",
            "tags": [],
            "custom_fields": {},
            "created": "2024-01-01T00:00:00Z",
            "last_updated": "2024-01-01T00:00:00Z"
        })
    }

    fn role_json(id: i64, name: &str, slug: &str) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "url": format!("https://netbox.example.com/api/ipam/roles/{id}/"),
            "display_url": format!("https://netbox.example.com/ipam/roles/{id}/"),
            "display": name,
            "name": name,
            "slug": slug,
            "weight": 1000,
            "description": "",
            "owner": null,
            "comments": "",
            "tags": [],
            "custom_fields": {},
            "created": "2024-01-01T00:00:00Z",
            "last_updated": "2024-01-01T00:00:00Z"
        })
    }

    // ── IP address tests ──────────────────────────────────────────────────────

    #[tokio::test]
    async fn ip_addresses_list_returns_page() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/ipam/ip-addresses/"))
            .and(header("Authorization", "Token secret"))
            .and(query_param("limit", "50"))
            .and(query_param("offset", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 1,
                "next": null,
                "previous": null,
                "results": [ip_address_json(1, "192.0.2.1/24")]
            })))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let page = client
            .ip_addresses_list(50, 0, &IpAddressFilter::default())
            .await
            .unwrap();
        assert_eq!(page.count, 1);
        assert_eq!(page.results[0].address, "192.0.2.1/24");
    }

    #[tokio::test]
    async fn ip_addresses_stream_walks_two_pages() {
        use futures_util::TryStreamExt as _;

        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/ipam/ip-addresses/"))
            .and(query_param("offset", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 2,
                "next": format!("{}/api/ipam/ip-addresses/?limit=50&offset=1", server.uri()),
                "previous": null,
                "results": [ip_address_json(1, "192.0.2.1/24")]
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/api/ipam/ip-addresses/"))
            .and(query_param("offset", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 2,
                "next": null,
                "previous": null,
                "results": [ip_address_json(2, "192.0.2.2/24")]
            })))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let ips: Vec<IpAddress> = client
            .ip_addresses(&IpAddressFilter::default())
            .try_collect()
            .await
            .unwrap();
        assert_eq!(ips.len(), 2);
        assert_eq!(ips[0].address, "192.0.2.1/24");
        assert_eq!(ips[1].address, "192.0.2.2/24");
    }

    #[tokio::test]
    async fn ip_address_retrieve_returns_object() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/ipam/ip-addresses/1/"))
            .and(header("Authorization", "Token secret"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(ip_address_json(1, "192.0.2.1/24")),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let ip = client.ip_address(1).await.unwrap();
        assert_eq!(ip.id, 1);
        assert_eq!(ip.address, "192.0.2.1/24");
    }

    #[tokio::test]
    async fn ip_address_retrieve_404_returns_api_error() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/ipam/ip-addresses/999/"))
            .respond_with(
                ResponseTemplate::new(404)
                    .set_body_json(serde_json::json!({"detail": "Not found."})),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let err = client.ip_address(999).await.unwrap_err();
        match err {
            crate::Error::Api { status, body } => {
                assert_eq!(status, reqwest::StatusCode::NOT_FOUND);
                assert_eq!(body.detail.as_deref(), Some("Not found."));
            }
            other => panic!("expected Api error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn ip_address_create_sends_correct_body() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/ipam/ip-addresses/"))
            .and(header("Authorization", "Token secret"))
            .and(header("Content-Type", "application/json"))
            .respond_with(
                ResponseTemplate::new(201).set_body_json(ip_address_json(3, "10.0.0.1/24")),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let body = IpAddressRequest {
            address: "10.0.0.1/24".into(),
            status: Some("active".into()),
            ..Default::default()
        };
        let ip = client.ip_address_create(&body).await.unwrap();
        assert_eq!(ip.address, "10.0.0.1/24");
    }

    #[tokio::test]
    async fn ip_address_create_400_returns_api_error() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/ipam/ip-addresses/"))
            .respond_with(
                ResponseTemplate::new(400)
                    .set_body_json(serde_json::json!({"address": ["Enter a valid value."]})),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let body = IpAddressRequest {
            address: "not-an-ip".into(),
            ..Default::default()
        };
        let err = client.ip_address_create(&body).await.unwrap_err();
        assert!(matches!(err, crate::Error::Api { .. }));
    }

    #[tokio::test]
    async fn ip_address_update_sends_put() {
        let server = MockServer::start().await;

        Mock::given(method("PUT"))
            .and(path("/api/ipam/ip-addresses/1/"))
            .and(header("Authorization", "Token secret"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(ip_address_json(1, "10.0.0.1/24")),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let body = IpAddressRequest {
            address: "10.0.0.1/24".into(),
            ..Default::default()
        };
        let ip = client.ip_address_update(1, &body).await.unwrap();
        assert_eq!(ip.address, "10.0.0.1/24");
    }

    #[tokio::test]
    async fn ip_address_patch_sends_patch() {
        let server = MockServer::start().await;

        Mock::given(method("PATCH"))
            .and(path("/api/ipam/ip-addresses/1/"))
            .and(header("Authorization", "Token secret"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(ip_address_json(1, "10.0.0.1/24")),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let body = IpAddressPatchRequest {
            dns_name: Some("host.example.com".into()),
            ..Default::default()
        };
        let ip = client.ip_address_patch(1, &body).await.unwrap();
        assert_eq!(ip.id, 1);
    }

    #[tokio::test]
    async fn ip_address_delete_sends_delete() {
        let server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/api/ipam/ip-addresses/1/"))
            .and(header("Authorization", "Token secret"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        client.ip_address_delete(1).await.unwrap();
    }

    #[tokio::test]
    async fn ip_address_delete_404_returns_api_error() {
        let server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/api/ipam/ip-addresses/999/"))
            .respond_with(
                ResponseTemplate::new(404)
                    .set_body_json(serde_json::json!({"detail": "Not found."})),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let err = client.ip_address_delete(999).await.unwrap_err();
        assert!(matches!(err, crate::Error::Api { .. }));
    }

    // ── Prefix tests ──────────────────────────────────────────────────────────

    #[tokio::test]
    async fn prefixes_list_returns_page() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/ipam/prefixes/"))
            .and(header("Authorization", "Token secret"))
            .and(query_param("limit", "50"))
            .and(query_param("offset", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 1,
                "next": null,
                "previous": null,
                "results": [prefix_json(1, "10.0.0.0/8")]
            })))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let page = client
            .prefixes_list(50, 0, &PrefixFilter::default())
            .await
            .unwrap();
        assert_eq!(page.count, 1);
        assert_eq!(page.results[0].prefix, "10.0.0.0/8");
    }

    #[tokio::test]
    async fn prefixes_stream_walks_two_pages() {
        use futures_util::TryStreamExt as _;

        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/ipam/prefixes/"))
            .and(query_param("offset", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 2,
                "next": format!("{}/api/ipam/prefixes/?limit=50&offset=1", server.uri()),
                "previous": null,
                "results": [prefix_json(1, "10.0.0.0/8")]
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/api/ipam/prefixes/"))
            .and(query_param("offset", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 2,
                "next": null,
                "previous": null,
                "results": [prefix_json(2, "172.16.0.0/12")]
            })))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let prefixes: Vec<Prefix> = client
            .prefixes(&PrefixFilter::default())
            .try_collect()
            .await
            .unwrap();
        assert_eq!(prefixes.len(), 2);
        assert_eq!(prefixes[0].prefix, "10.0.0.0/8");
        assert_eq!(prefixes[1].prefix, "172.16.0.0/12");
    }

    #[tokio::test]
    async fn prefix_retrieve_returns_object() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/ipam/prefixes/1/"))
            .and(header("Authorization", "Token secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(prefix_json(1, "10.0.0.0/8")))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let p = client.prefix(1).await.unwrap();
        assert_eq!(p.id, 1);
        assert_eq!(p.prefix, "10.0.0.0/8");
    }

    #[tokio::test]
    async fn prefix_retrieve_404_returns_api_error() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/ipam/prefixes/999/"))
            .respond_with(
                ResponseTemplate::new(404)
                    .set_body_json(serde_json::json!({"detail": "Not found."})),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let err = client.prefix(999).await.unwrap_err();
        assert!(
            matches!(err, crate::Error::Api { status, .. } if status == reqwest::StatusCode::NOT_FOUND)
        );
    }

    #[tokio::test]
    async fn prefix_create_sends_correct_body() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/ipam/prefixes/"))
            .and(header("Authorization", "Token secret"))
            .and(header("Content-Type", "application/json"))
            .respond_with(
                ResponseTemplate::new(201).set_body_json(prefix_json(5, "192.168.1.0/24")),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let body = PrefixRequest {
            prefix: "192.168.1.0/24".into(),
            status: Some("active".into()),
            ..Default::default()
        };
        let p = client.prefix_create(&body).await.unwrap();
        assert_eq!(p.prefix, "192.168.1.0/24");
    }

    #[tokio::test]
    async fn prefix_create_400_returns_api_error() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/ipam/prefixes/"))
            .respond_with(
                ResponseTemplate::new(400)
                    .set_body_json(serde_json::json!({"prefix": ["Enter a valid value."]})),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let body = PrefixRequest {
            prefix: "not-a-prefix".into(),
            ..Default::default()
        };
        let err = client.prefix_create(&body).await.unwrap_err();
        assert!(matches!(err, crate::Error::Api { .. }));
    }

    #[tokio::test]
    async fn prefix_update_sends_put() {
        let server = MockServer::start().await;

        Mock::given(method("PUT"))
            .and(path("/api/ipam/prefixes/1/"))
            .and(header("Authorization", "Token secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(prefix_json(1, "10.0.0.0/8")))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let body = PrefixRequest {
            prefix: "10.0.0.0/8".into(),
            ..Default::default()
        };
        let p = client.prefix_update(1, &body).await.unwrap();
        assert_eq!(p.prefix, "10.0.0.0/8");
    }

    #[tokio::test]
    async fn prefix_patch_sends_patch() {
        let server = MockServer::start().await;

        Mock::given(method("PATCH"))
            .and(path("/api/ipam/prefixes/1/"))
            .and(header("Authorization", "Token secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(prefix_json(1, "10.0.0.0/8")))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let body = PrefixPatchRequest {
            description: Some("Updated".into()),
            ..Default::default()
        };
        let p = client.prefix_patch(1, &body).await.unwrap();
        assert_eq!(p.id, 1);
    }

    #[tokio::test]
    async fn prefix_delete_sends_delete() {
        let server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/api/ipam/prefixes/1/"))
            .and(header("Authorization", "Token secret"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        client.prefix_delete(1).await.unwrap();
    }

    #[tokio::test]
    async fn prefix_delete_404_returns_api_error() {
        let server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/api/ipam/prefixes/999/"))
            .respond_with(
                ResponseTemplate::new(404)
                    .set_body_json(serde_json::json!({"detail": "Not found."})),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let err = client.prefix_delete(999).await.unwrap_err();
        assert!(matches!(err, crate::Error::Api { .. }));
    }

    // ── Aggregate tests ───────────────────────────────────────────────────────

    #[tokio::test]
    async fn aggregates_list_returns_page() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/ipam/aggregates/"))
            .and(header("Authorization", "Token secret"))
            .and(query_param("limit", "50"))
            .and(query_param("offset", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 1,
                "next": null,
                "previous": null,
                "results": [aggregate_json(1, "10.0.0.0/8")]
            })))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let page = client
            .aggregates_list(50, 0, &AggregateFilter::default())
            .await
            .unwrap();
        assert_eq!(page.count, 1);
        assert_eq!(page.results[0].prefix, "10.0.0.0/8");
        assert_eq!(page.results[0].rir.slug, "arin");
    }

    #[tokio::test]
    async fn aggregates_stream_walks_two_pages() {
        use futures_util::TryStreamExt as _;

        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/ipam/aggregates/"))
            .and(query_param("offset", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 2,
                "next": format!("{}/api/ipam/aggregates/?limit=50&offset=1", server.uri()),
                "previous": null,
                "results": [aggregate_json(1, "10.0.0.0/8")]
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/api/ipam/aggregates/"))
            .and(query_param("offset", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 2,
                "next": null,
                "previous": null,
                "results": [aggregate_json(2, "100.64.0.0/10")]
            })))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let aggs: Vec<Aggregate> = client
            .aggregates(&AggregateFilter::default())
            .try_collect()
            .await
            .unwrap();
        assert_eq!(aggs.len(), 2);
        assert_eq!(aggs[0].prefix, "10.0.0.0/8");
        assert_eq!(aggs[1].prefix, "100.64.0.0/10");
    }

    #[tokio::test]
    async fn aggregate_retrieve_returns_object() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/ipam/aggregates/1/"))
            .and(header("Authorization", "Token secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(aggregate_json(1, "10.0.0.0/8")))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let agg = client.aggregate(1).await.unwrap();
        assert_eq!(agg.id, 1);
        assert_eq!(agg.prefix, "10.0.0.0/8");
    }

    #[tokio::test]
    async fn aggregate_retrieve_404_returns_api_error() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/ipam/aggregates/999/"))
            .respond_with(
                ResponseTemplate::new(404)
                    .set_body_json(serde_json::json!({"detail": "Not found."})),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let err = client.aggregate(999).await.unwrap_err();
        assert!(
            matches!(err, crate::Error::Api { status, .. } if status == reqwest::StatusCode::NOT_FOUND)
        );
    }

    #[tokio::test]
    async fn aggregate_create_sends_correct_body() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/ipam/aggregates/"))
            .and(header("Authorization", "Token secret"))
            .and(header("Content-Type", "application/json"))
            .respond_with(
                ResponseTemplate::new(201).set_body_json(aggregate_json(3, "192.0.0.0/8")),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let body = AggregateRequest {
            prefix: "192.0.0.0/8".into(),
            rir: 1,
            ..Default::default()
        };
        let agg = client.aggregate_create(&body).await.unwrap();
        assert_eq!(agg.prefix, "192.0.0.0/8");
    }

    #[tokio::test]
    async fn aggregate_create_400_returns_api_error() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/ipam/aggregates/"))
            .respond_with(
                ResponseTemplate::new(400)
                    .set_body_json(serde_json::json!({"prefix": ["Enter a valid value."]})),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let body = AggregateRequest {
            prefix: "bad".into(),
            rir: 1,
            ..Default::default()
        };
        let err = client.aggregate_create(&body).await.unwrap_err();
        assert!(matches!(err, crate::Error::Api { .. }));
    }

    #[tokio::test]
    async fn aggregate_update_sends_put() {
        let server = MockServer::start().await;

        Mock::given(method("PUT"))
            .and(path("/api/ipam/aggregates/1/"))
            .and(header("Authorization", "Token secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(aggregate_json(1, "10.0.0.0/8")))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let body = AggregateRequest {
            prefix: "10.0.0.0/8".into(),
            rir: 1,
            ..Default::default()
        };
        let agg = client.aggregate_update(1, &body).await.unwrap();
        assert_eq!(agg.prefix, "10.0.0.0/8");
    }

    #[tokio::test]
    async fn aggregate_patch_sends_patch() {
        let server = MockServer::start().await;

        Mock::given(method("PATCH"))
            .and(path("/api/ipam/aggregates/1/"))
            .and(header("Authorization", "Token secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(aggregate_json(1, "10.0.0.0/8")))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let body = AggregatePatchRequest {
            description: Some("Updated".into()),
            ..Default::default()
        };
        let agg = client.aggregate_patch(1, &body).await.unwrap();
        assert_eq!(agg.id, 1);
    }

    #[tokio::test]
    async fn aggregate_delete_sends_delete() {
        let server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/api/ipam/aggregates/1/"))
            .and(header("Authorization", "Token secret"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        client.aggregate_delete(1).await.unwrap();
    }

    #[tokio::test]
    async fn aggregate_delete_404_returns_api_error() {
        let server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/api/ipam/aggregates/999/"))
            .respond_with(
                ResponseTemplate::new(404)
                    .set_body_json(serde_json::json!({"detail": "Not found."})),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let err = client.aggregate_delete(999).await.unwrap_err();
        assert!(matches!(err, crate::Error::Api { .. }));
    }

    // ── Role tests ────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn roles_list_returns_page() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/ipam/roles/"))
            .and(header("Authorization", "Token secret"))
            .and(query_param("limit", "50"))
            .and(query_param("offset", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 1,
                "next": null,
                "previous": null,
                "results": [role_json(1, "Management", "management")]
            })))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let page = client
            .roles_list(50, 0, &RoleFilter::default())
            .await
            .unwrap();
        assert_eq!(page.count, 1);
        assert_eq!(page.results[0].slug, "management");
    }

    #[tokio::test]
    async fn roles_stream_walks_two_pages() {
        use futures_util::TryStreamExt as _;

        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/ipam/roles/"))
            .and(query_param("offset", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 2,
                "next": format!("{}/api/ipam/roles/?limit=50&offset=1", server.uri()),
                "previous": null,
                "results": [role_json(1, "Management", "management")]
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/api/ipam/roles/"))
            .and(query_param("offset", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 2,
                "next": null,
                "previous": null,
                "results": [role_json(2, "Loopback", "loopback")]
            })))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let roles: Vec<Role> = client
            .roles(&RoleFilter::default())
            .try_collect()
            .await
            .unwrap();
        assert_eq!(roles.len(), 2);
        assert_eq!(roles[0].slug, "management");
        assert_eq!(roles[1].slug, "loopback");
    }

    #[tokio::test]
    async fn role_retrieve_returns_object() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/ipam/roles/1/"))
            .and(header("Authorization", "Token secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(role_json(
                1,
                "Management",
                "management",
            )))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let r = client.role(1).await.unwrap();
        assert_eq!(r.id, 1);
        assert_eq!(r.slug, "management");
    }

    #[tokio::test]
    async fn role_retrieve_404_returns_api_error() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/ipam/roles/999/"))
            .respond_with(
                ResponseTemplate::new(404)
                    .set_body_json(serde_json::json!({"detail": "Not found."})),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let err = client.role(999).await.unwrap_err();
        assert!(
            matches!(err, crate::Error::Api { status, .. } if status == reqwest::StatusCode::NOT_FOUND)
        );
    }

    #[tokio::test]
    async fn role_create_sends_correct_body() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/ipam/roles/"))
            .and(header("Authorization", "Token secret"))
            .and(header("Content-Type", "application/json"))
            .respond_with(ResponseTemplate::new(201).set_body_json(role_json(
                4,
                "Infrastructure",
                "infrastructure",
            )))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let body = RoleRequest {
            name: "Infrastructure".into(),
            slug: "infrastructure".into(),
            ..Default::default()
        };
        let r = client.role_create(&body).await.unwrap();
        assert_eq!(r.slug, "infrastructure");
    }

    #[tokio::test]
    async fn role_create_400_returns_api_error() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/ipam/roles/"))
            .respond_with(
                ResponseTemplate::new(400)
                    .set_body_json(serde_json::json!({"slug": ["This field is required."]})),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let body = RoleRequest {
            name: "Bad".into(),
            slug: String::new(),
            ..Default::default()
        };
        let err = client.role_create(&body).await.unwrap_err();
        assert!(matches!(err, crate::Error::Api { .. }));
    }

    #[tokio::test]
    async fn role_update_sends_put() {
        let server = MockServer::start().await;

        Mock::given(method("PUT"))
            .and(path("/api/ipam/roles/1/"))
            .and(header("Authorization", "Token secret"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(role_json(1, "Updated", "updated")),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let body = RoleRequest {
            name: "Updated".into(),
            slug: "updated".into(),
            ..Default::default()
        };
        let r = client.role_update(1, &body).await.unwrap();
        assert_eq!(r.slug, "updated");
    }

    #[tokio::test]
    async fn role_patch_sends_patch() {
        let server = MockServer::start().await;

        Mock::given(method("PATCH"))
            .and(path("/api/ipam/roles/1/"))
            .and(header("Authorization", "Token secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(role_json(
                1,
                "Management",
                "management",
            )))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let body = RolePatchRequest {
            description: Some("Core management".into()),
            ..Default::default()
        };
        let r = client.role_patch(1, &body).await.unwrap();
        assert_eq!(r.id, 1);
    }

    #[tokio::test]
    async fn role_delete_sends_delete() {
        let server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/api/ipam/roles/1/"))
            .and(header("Authorization", "Token secret"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        client.role_delete(1).await.unwrap();
    }

    #[tokio::test]
    async fn role_delete_404_returns_api_error() {
        let server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/api/ipam/roles/999/"))
            .respond_with(
                ResponseTemplate::new(404)
                    .set_body_json(serde_json::json!({"detail": "Not found."})),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let err = client.role_delete(999).await.unwrap_err();
        assert!(matches!(err, crate::Error::Api { .. }));
    }
}
