//! Methods for the `dcim` tag of the NetBox REST API.
//!
//! # Example
//!
//! ```no_run
//! # async fn example() -> netbox_client::Result<()> {
//! use futures_util::TryStreamExt as _;
//! use netbox_client::dcim::SiteFilter;
//!
//! let client = netbox_client::NetboxClient::new("https://netbox.example.com", "abc123")?;
//!
//! // Stream all active sites
//! let filter = SiteFilter { status: vec!["active".into()], ..Default::default() };
//! let sites: Vec<_> = client.sites(&filter).try_collect().await?;
//! println!("Found {} active sites", sites.len());
//! # Ok(()) }
//! ```

// Page sizes from NetBox are always far smaller than 2^32 items.
#![allow(clippy::cast_possible_truncation)]

use std::collections::VecDeque;

use futures_core::stream::BoxStream;
use futures_util::stream::try_unfold;

use crate::RequestBuilderExt as _;
use crate::{
    Interface, InterfacePatchRequest, InterfaceRequest, MACAddress, MACAddressPatchRequest,
    MACAddressRequest, NetboxClient, Paginated, Site, SitePatchRequest, SiteRequest,
};

const PAGE_SIZE: u32 = 50;

// ── Filter types ─────────────────────────────────────────────────────────────

/// Filters for the [`NetboxClient::sites_list`] / [`NetboxClient::sites`] endpoints.
#[derive(Debug, Clone, Default)]
pub struct SiteFilter {
    /// Free-text search.
    pub q: Option<String>,
    /// Limit results to these IDs.
    pub id: Vec<i64>,
    /// Filter by status (e.g. `active`, `planned`).
    pub status: Vec<String>,
    /// Filter by region slug.
    pub region: Vec<String>,
    /// Filter by region ID.
    pub region_id: Vec<i64>,
    /// Filter by site-group slug.
    pub group: Vec<String>,
    /// Filter by site-group ID.
    pub group_id: Vec<i64>,
    /// Filter by tenant slug.
    pub tenant: Vec<String>,
    /// Filter by tenant ID.
    pub tenant_id: Vec<i64>,
    /// Filter by name (exact, case-insensitive).
    pub name: Vec<String>,
    /// Filter by slug.
    pub slug: Vec<String>,
    /// Filter by facility.
    pub facility: Vec<String>,
    /// Filter by ASN ID.
    pub asn_id: Vec<i64>,
}

impl SiteFilter {
    fn as_query(&self) -> Vec<(String, String)> {
        let mut p = Vec::new();
        if let Some(q) = &self.q {
            p.push(("q".into(), q.clone()));
        }
        for v in &self.id {
            p.push(("id".into(), v.to_string()));
        }
        for v in &self.status {
            p.push(("status".into(), v.clone()));
        }
        for v in &self.region {
            p.push(("region".into(), v.clone()));
        }
        for v in &self.region_id {
            p.push(("region_id".into(), v.to_string()));
        }
        for v in &self.group {
            p.push(("group".into(), v.clone()));
        }
        for v in &self.group_id {
            p.push(("group_id".into(), v.to_string()));
        }
        for v in &self.tenant {
            p.push(("tenant".into(), v.clone()));
        }
        for v in &self.tenant_id {
            p.push(("tenant_id".into(), v.to_string()));
        }
        for v in &self.name {
            p.push(("name".into(), v.clone()));
        }
        for v in &self.slug {
            p.push(("slug".into(), v.clone()));
        }
        for v in &self.facility {
            p.push(("facility".into(), v.clone()));
        }
        for v in &self.asn_id {
            p.push(("asn_id".into(), v.to_string()));
        }
        p
    }
}

/// Filters for the [`NetboxClient::interfaces_list`] / [`NetboxClient::interfaces`] endpoints.
#[derive(Debug, Clone, Default)]
pub struct InterfaceFilter {
    /// Free-text search.
    pub q: Option<String>,
    /// Limit results to these IDs.
    pub id: Vec<i64>,
    /// Filter by device name.
    pub device: Vec<String>,
    /// Filter by device ID.
    pub device_id: Vec<i64>,
    /// Filter by interface name.
    pub name: Vec<String>,
    /// Filter by interface type (e.g. `virtual`, `1000base-t`).
    pub r#type: Vec<String>,
    /// Filter by enabled status.
    pub enabled: Option<bool>,
    /// Filter by LAG interface ID.
    pub lag_id: Vec<i64>,
    /// Filter by management-only status.
    pub mgmt_only: Option<bool>,
    /// Filter by VRF ID.
    pub vrf_id: Vec<i64>,
    /// Filter by MAC address.
    pub mac_address: Vec<String>,
}

impl InterfaceFilter {
    fn as_query(&self) -> Vec<(String, String)> {
        let mut p = Vec::new();
        if let Some(q) = &self.q {
            p.push(("q".into(), q.clone()));
        }
        for v in &self.id {
            p.push(("id".into(), v.to_string()));
        }
        for v in &self.device {
            p.push(("device".into(), v.clone()));
        }
        for v in &self.device_id {
            p.push(("device_id".into(), v.to_string()));
        }
        for v in &self.name {
            p.push(("name".into(), v.clone()));
        }
        for v in &self.r#type {
            p.push(("type".into(), v.clone()));
        }
        if let Some(enabled) = self.enabled {
            p.push(("enabled".into(), enabled.to_string()));
        }
        for v in &self.lag_id {
            p.push(("lag_id".into(), v.to_string()));
        }
        if let Some(mgmt_only) = self.mgmt_only {
            p.push(("mgmt_only".into(), mgmt_only.to_string()));
        }
        for v in &self.vrf_id {
            p.push(("vrf_id".into(), v.to_string()));
        }
        for v in &self.mac_address {
            p.push(("mac_address".into(), v.clone()));
        }
        p
    }
}

/// Filters for the [`NetboxClient::mac_addresses_list`] / [`NetboxClient::mac_addresses`] endpoints.
#[derive(Debug, Clone, Default)]
pub struct MACAddressFilter {
    /// Free-text search.
    pub q: Option<String>,
    /// Limit results to these IDs.
    pub id: Vec<i64>,
    /// Filter by MAC address (exact).
    pub mac_address: Vec<String>,
    /// Filter by assigned object content-type (e.g. `dcim.interface`).
    pub assigned_object_type: Vec<String>,
    /// Filter by assigned object ID.
    pub assigned_object_id: Vec<i64>,
    /// Filter by device name (for interfaces assigned to the MAC).
    pub device: Vec<String>,
    /// Filter by device ID.
    pub device_id: Vec<i64>,
    /// Filter by interface name.
    pub interface: Vec<String>,
    /// Filter by interface ID.
    pub interface_id: Vec<i64>,
    /// Filter by tag slug.
    pub tag: Vec<String>,
}

impl MACAddressFilter {
    fn as_query(&self) -> Vec<(String, String)> {
        let mut p = Vec::new();
        if let Some(q) = &self.q {
            p.push(("q".into(), q.clone()));
        }
        for v in &self.id {
            p.push(("id".into(), v.to_string()));
        }
        for v in &self.mac_address {
            p.push(("mac_address".into(), v.clone()));
        }
        for v in &self.assigned_object_type {
            p.push(("assigned_object_type".into(), v.clone()));
        }
        for v in &self.assigned_object_id {
            p.push(("assigned_object_id".into(), v.to_string()));
        }
        for v in &self.device {
            p.push(("device".into(), v.clone()));
        }
        for v in &self.device_id {
            p.push(("device_id".into(), v.to_string()));
        }
        for v in &self.interface {
            p.push(("interface".into(), v.clone()));
        }
        for v in &self.interface_id {
            p.push(("interface_id".into(), v.to_string()));
        }
        for v in &self.tag {
            p.push(("tag".into(), v.clone()));
        }
        p
    }
}

// ── NetboxClient implementation ───────────────────────────────────────────────

impl NetboxClient {
    // ── Sites ─────────────────────────────────────────────────────────────────

    /// Returns a single page of sites.
    ///
    /// Maps to `GET /api/dcim/sites/`
    /// (`operationId`: `dcim_sites_list`).
    ///
    /// # Errors
    ///
    /// Returns an error on HTTP or deserialization failure.
    pub async fn sites_list(
        &self,
        limit: u32,
        offset: u32,
        filter: &SiteFilter,
    ) -> crate::Result<Paginated<Site>> {
        let url = format!("{}/api/dcim/sites/", self.base_url);
        let mut query = filter.as_query();
        query.push(("limit".into(), limit.to_string()));
        query.push(("offset".into(), offset.to_string()));
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .query(&query)
            .send_json::<Paginated<Site>>()
            .await
    }

    /// Streams all sites matching `filter`, auto-paginating.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> netbox_client::Result<()> {
    /// use futures_util::TryStreamExt as _;
    /// use netbox_client::dcim::SiteFilter;
    /// let client = netbox_client::NetboxClient::new("https://netbox.example.com", "token")?;
    /// let all: Vec<_> = client.sites(&SiteFilter::default()).try_collect().await?;
    /// # Ok(()) }
    /// ```
    #[must_use]
    pub fn sites<'a>(&'a self, filter: &'a SiteFilter) -> BoxStream<'a, crate::Result<Site>> {
        Box::pin(try_unfold(
            (Some(0u32), VecDeque::<Site>::new()),
            move |(next_offset, mut buf)| async move {
                if let Some(item) = buf.pop_front() {
                    return Ok(Some((item, (next_offset, buf))));
                }
                let Some(offset) = next_offset else {
                    return Ok(None);
                };
                let page = self.sites_list(PAGE_SIZE, offset, filter).await?;
                let new_next = page
                    .next
                    .is_some()
                    .then_some(offset + page.results.len() as u32);
                let mut buf: VecDeque<Site> = page.results.into_iter().collect();
                match buf.pop_front() {
                    Some(item) => Ok(Some((item, (new_next, buf)))),
                    None => Ok(None),
                }
            },
        ))
    }

    /// Returns a single site by ID.
    ///
    /// Maps to `GET /api/dcim/sites/{id}/`
    /// (`operationId`: `dcim_sites_retrieve`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 404 if the site does not exist.
    pub async fn site(&self, id: i64) -> crate::Result<Site> {
        let url = format!("{}/api/dcim/sites/{id}/", self.base_url);
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .send_json::<Site>()
            .await
    }

    /// Creates a new site.
    ///
    /// Maps to `POST /api/dcim/sites/`
    /// (`operationId`: `dcim_sites_create`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn site_create(&self, body: &SiteRequest) -> crate::Result<Site> {
        let url = format!("{}/api/dcim/sites/", self.base_url);
        self.http
            .post(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<Site>()
            .await
    }

    /// Replaces a site (full update).
    ///
    /// Maps to `PUT /api/dcim/sites/{id}/`
    /// (`operationId`: `dcim_sites_update`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn site_update(&self, id: i64, body: &SiteRequest) -> crate::Result<Site> {
        let url = format!("{}/api/dcim/sites/{id}/", self.base_url);
        self.http
            .put(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<Site>()
            .await
    }

    /// Partially updates a site.
    ///
    /// Maps to `PATCH /api/dcim/sites/{id}/`
    /// (`operationId`: `dcim_sites_partial_update`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn site_patch(&self, id: i64, body: &SitePatchRequest) -> crate::Result<Site> {
        let url = format!("{}/api/dcim/sites/{id}/", self.base_url);
        self.http
            .patch(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<Site>()
            .await
    }

    /// Deletes a site.
    ///
    /// Maps to `DELETE /api/dcim/sites/{id}/`
    /// (`operationId`: `dcim_sites_destroy`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 404 if the site does not exist.
    pub async fn site_delete(&self, id: i64) -> crate::Result<()> {
        let url = format!("{}/api/dcim/sites/{id}/", self.base_url);
        crate::delete_no_content(&self.http, &url, &self.token).await
    }

    // ── Interfaces ────────────────────────────────────────────────────────────

    /// Returns a single page of interfaces.
    ///
    /// Maps to `GET /api/dcim/interfaces/`
    /// (`operationId`: `dcim_interfaces_list`).
    ///
    /// # Errors
    ///
    /// Returns an error on HTTP or deserialization failure.
    pub async fn interfaces_list(
        &self,
        limit: u32,
        offset: u32,
        filter: &InterfaceFilter,
    ) -> crate::Result<Paginated<Interface>> {
        let url = format!("{}/api/dcim/interfaces/", self.base_url);
        let mut query = filter.as_query();
        query.push(("limit".into(), limit.to_string()));
        query.push(("offset".into(), offset.to_string()));
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .query(&query)
            .send_json::<Paginated<Interface>>()
            .await
    }

    /// Streams all interfaces matching `filter`, auto-paginating.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> netbox_client::Result<()> {
    /// use futures_util::TryStreamExt as _;
    /// use netbox_client::dcim::InterfaceFilter;
    /// let client = netbox_client::NetboxClient::new("https://netbox.example.com", "token")?;
    /// let all: Vec<_> = client.interfaces(&InterfaceFilter::default()).try_collect().await?;
    /// # Ok(()) }
    /// ```
    #[must_use]
    pub fn interfaces<'a>(
        &'a self,
        filter: &'a InterfaceFilter,
    ) -> BoxStream<'a, crate::Result<Interface>> {
        Box::pin(try_unfold(
            (Some(0u32), VecDeque::<Interface>::new()),
            move |(next_offset, mut buf)| async move {
                if let Some(item) = buf.pop_front() {
                    return Ok(Some((item, (next_offset, buf))));
                }
                let Some(offset) = next_offset else {
                    return Ok(None);
                };
                let page = self.interfaces_list(PAGE_SIZE, offset, filter).await?;
                let new_next = page
                    .next
                    .is_some()
                    .then_some(offset + page.results.len() as u32);
                let mut buf: VecDeque<Interface> = page.results.into_iter().collect();
                match buf.pop_front() {
                    Some(item) => Ok(Some((item, (new_next, buf)))),
                    None => Ok(None),
                }
            },
        ))
    }

    /// Returns a single interface by ID.
    ///
    /// Maps to `GET /api/dcim/interfaces/{id}/`
    /// (`operationId`: `dcim_interfaces_retrieve`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 404 if the interface does not exist.
    pub async fn interface(&self, id: i64) -> crate::Result<Interface> {
        let url = format!("{}/api/dcim/interfaces/{id}/", self.base_url);
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .send_json::<Interface>()
            .await
    }

    /// Creates a new interface.
    ///
    /// Maps to `POST /api/dcim/interfaces/`
    /// (`operationId`: `dcim_interfaces_create`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn interface_create(&self, body: &InterfaceRequest) -> crate::Result<Interface> {
        let url = format!("{}/api/dcim/interfaces/", self.base_url);
        self.http
            .post(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<Interface>()
            .await
    }

    /// Replaces an interface (full update).
    ///
    /// Maps to `PUT /api/dcim/interfaces/{id}/`
    /// (`operationId`: `dcim_interfaces_update`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn interface_update(
        &self,
        id: i64,
        body: &InterfaceRequest,
    ) -> crate::Result<Interface> {
        let url = format!("{}/api/dcim/interfaces/{id}/", self.base_url);
        self.http
            .put(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<Interface>()
            .await
    }

    /// Partially updates an interface.
    ///
    /// Maps to `PATCH /api/dcim/interfaces/{id}/`
    /// (`operationId`: `dcim_interfaces_partial_update`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn interface_patch(
        &self,
        id: i64,
        body: &InterfacePatchRequest,
    ) -> crate::Result<Interface> {
        let url = format!("{}/api/dcim/interfaces/{id}/", self.base_url);
        self.http
            .patch(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<Interface>()
            .await
    }

    /// Deletes an interface.
    ///
    /// Maps to `DELETE /api/dcim/interfaces/{id}/`
    /// (`operationId`: `dcim_interfaces_destroy`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 404 if the interface does not exist.
    pub async fn interface_delete(&self, id: i64) -> crate::Result<()> {
        let url = format!("{}/api/dcim/interfaces/{id}/", self.base_url);
        crate::delete_no_content(&self.http, &url, &self.token).await
    }

    // ── MAC Addresses ─────────────────────────────────────────────────────────

    /// Return a single page of MAC addresses.
    ///
    /// Use [`mac_addresses`](Self::mac_addresses) to stream all pages automatically.
    ///
    /// # Errors
    ///
    /// Returns an error on HTTP or deserialization failure.
    pub async fn mac_addresses_list(
        &self,
        limit: u32,
        offset: u32,
        filter: &MACAddressFilter,
    ) -> crate::Result<Paginated<MACAddress>> {
        let url = format!("{}/api/dcim/mac-addresses/", self.base_url);
        let mut query = filter.as_query();
        query.push(("limit".into(), limit.to_string()));
        query.push(("offset".into(), offset.to_string()));
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .query(&query)
            .send_json::<Paginated<MACAddress>>()
            .await
    }

    /// Stream all MAC addresses matching `filter`, auto-paginating.
    #[must_use]
    pub fn mac_addresses(
        &self,
        filter: &MACAddressFilter,
    ) -> BoxStream<'_, crate::Result<MACAddress>> {
        let filter = filter.clone();
        Box::pin(try_unfold(
            (Some(0u32), std::collections::VecDeque::<MACAddress>::new()),
            move |(next_offset, mut buf)| {
                let filter = filter.clone();
                async move {
                    if let Some(item) = buf.pop_front() {
                        return Ok(Some((item, (next_offset, buf))));
                    }
                    let offset = match next_offset {
                        Some(o) => o,
                        None => return Ok(None),
                    };
                    let page = self.mac_addresses_list(PAGE_SIZE, offset, &filter).await?;
                    let new_offset = if page.next.is_some() {
                        Some(offset + PAGE_SIZE)
                    } else {
                        None
                    };
                    let mut iter = page.results.into_iter();
                    let first = iter.next();
                    buf.extend(iter);
                    match first {
                        Some(item) => Ok(Some((item, (new_offset, buf)))),
                        None => Ok(None),
                    }
                }
            },
        ))
    }

    /// Retrieve a single MAC address by ID.
    ///
    /// # Errors
    ///
    /// Returns an error on HTTP or deserialization failure, including 404.
    pub async fn mac_address(&self, id: i64) -> crate::Result<MACAddress> {
        let url = format!("{}/api/dcim/mac-addresses/{id}/", self.base_url);
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .send_json::<MACAddress>()
            .await
    }

    /// Create a new MAC address.
    ///
    /// # Errors
    ///
    /// Returns an error on HTTP or deserialization failure.
    pub async fn mac_address_create(&self, body: &MACAddressRequest) -> crate::Result<MACAddress> {
        let url = format!("{}/api/dcim/mac-addresses/", self.base_url);
        self.http
            .post(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<MACAddress>()
            .await
    }

    /// Replace a MAC address (PUT).
    ///
    /// # Errors
    ///
    /// Returns an error on HTTP or deserialization failure.
    pub async fn mac_address_update(
        &self,
        id: i64,
        body: &MACAddressRequest,
    ) -> crate::Result<MACAddress> {
        let url = format!("{}/api/dcim/mac-addresses/{id}/", self.base_url);
        self.http
            .put(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<MACAddress>()
            .await
    }

    /// Partially update a MAC address (PATCH).
    ///
    /// # Errors
    ///
    /// Returns an error on HTTP or deserialization failure.
    pub async fn mac_address_patch(
        &self,
        id: i64,
        body: &MACAddressPatchRequest,
    ) -> crate::Result<MACAddress> {
        let url = format!("{}/api/dcim/mac-addresses/{id}/", self.base_url);
        self.http
            .patch(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<MACAddress>()
            .await
    }

    /// Delete a MAC address.
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 404 if the MAC address does not exist.
    pub async fn mac_address_delete(&self, id: i64) -> crate::Result<()> {
        let url = format!("{}/api/dcim/mac-addresses/{id}/", self.base_url);
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

    fn site_json(id: i64, name: &str, slug: &str) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "url": format!("https://netbox.example.com/api/dcim/sites/{id}/"),
            "display_url": format!("https://netbox.example.com/dcim/sites/{id}/"),
            "display": name,
            "name": name,
            "slug": slug,
            "status": {"value": "active", "label": "Active"},
            "region": null,
            "group": null,
            "tenant": null,
            "facility": "",
            "time_zone": null,
            "description": "",
            "physical_address": "",
            "shipping_address": "",
            "latitude": null,
            "longitude": null,
            "owner": null,
            "comments": "",
            "asns": [],
            "tags": [],
            "custom_fields": {},
            "created": "2024-01-01T00:00:00Z",
            "last_updated": "2024-01-01T00:00:00Z"
        })
    }

    #[tokio::test]
    async fn sites_list_returns_page() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/dcim/sites/"))
            .and(header("Authorization", "Token secret"))
            .and(query_param("limit", "50"))
            .and(query_param("offset", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 1,
                "next": null,
                "previous": null,
                "results": [site_json(1, "Site A", "site-a")]
            })))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let page = client
            .sites_list(50, 0, &SiteFilter::default())
            .await
            .unwrap();
        assert_eq!(page.count, 1);
        assert_eq!(page.results[0].name, "Site A");
    }

    #[tokio::test]
    async fn sites_stream_walks_two_pages() {
        use futures_util::TryStreamExt as _;

        let server = MockServer::start().await;

        // page 1
        Mock::given(method("GET"))
            .and(path("/api/dcim/sites/"))
            .and(query_param("offset", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 2,
                "next": format!("{}/api/dcim/sites/?limit=50&offset=1", server.uri()),
                "previous": null,
                "results": [site_json(1, "Site A", "site-a")]
            })))
            .mount(&server)
            .await;

        // page 2
        Mock::given(method("GET"))
            .and(path("/api/dcim/sites/"))
            .and(query_param("offset", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 2,
                "next": null,
                "previous": null,
                "results": [site_json(2, "Site B", "site-b")]
            })))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let sites: Vec<Site> = client
            .sites(&SiteFilter::default())
            .try_collect()
            .await
            .unwrap();
        assert_eq!(sites.len(), 2);
        assert_eq!(sites[0].name, "Site A");
        assert_eq!(sites[1].name, "Site B");
    }

    #[tokio::test]
    async fn site_retrieve_returns_site() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/dcim/sites/1/"))
            .and(header("Authorization", "Token secret"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(site_json(1, "Site A", "site-a")),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let site = client.site(1).await.unwrap();
        assert_eq!(site.id, 1);
        assert_eq!(site.slug, "site-a");
    }

    #[tokio::test]
    async fn site_retrieve_404_returns_api_error() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/dcim/sites/999/"))
            .respond_with(
                ResponseTemplate::new(404)
                    .set_body_json(serde_json::json!({"detail": "Not found."})),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let err = client.site(999).await.unwrap_err();
        match err {
            crate::Error::Api { status, body } => {
                assert_eq!(status, reqwest::StatusCode::NOT_FOUND);
                assert_eq!(body.detail.as_deref(), Some("Not found."));
            }
            other => panic!("expected Api error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn site_create_sends_correct_body() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/dcim/sites/"))
            .and(header("Authorization", "Token secret"))
            .and(header("Content-Type", "application/json"))
            .respond_with(
                ResponseTemplate::new(201).set_body_json(site_json(5, "New Site", "new-site")),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let body = SiteRequest {
            name: "New Site".into(),
            slug: "new-site".into(),
            status: Some("active".into()),
            ..Default::default()
        };
        let site = client.site_create(&body).await.unwrap();
        assert_eq!(site.name, "New Site");
    }

    #[tokio::test]
    async fn site_create_400_returns_api_error() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/dcim/sites/"))
            .respond_with(
                ResponseTemplate::new(400)
                    .set_body_json(serde_json::json!({"slug": ["This field is required."]})),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let body = SiteRequest {
            name: "Bad Site".into(),
            slug: String::new(),
            ..Default::default()
        };
        let err = client.site_create(&body).await.unwrap_err();
        assert!(matches!(err, crate::Error::Api { .. }));
    }

    #[tokio::test]
    async fn site_update_sends_put() {
        let server = MockServer::start().await;

        Mock::given(method("PUT"))
            .and(path("/api/dcim/sites/3/"))
            .and(header("Authorization", "Token secret"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(site_json(3, "Updated", "updated")),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let body = SiteRequest {
            name: "Updated".into(),
            slug: "updated".into(),
            ..Default::default()
        };
        let site = client.site_update(3, &body).await.unwrap();
        assert_eq!(site.slug, "updated");
    }

    #[tokio::test]
    async fn site_patch_sends_patch() {
        let server = MockServer::start().await;

        Mock::given(method("PATCH"))
            .and(path("/api/dcim/sites/4/"))
            .and(header("Authorization", "Token secret"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(site_json(4, "Patched", "patched")),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let body = SitePatchRequest {
            name: Some("Patched".into()),
            ..Default::default()
        };
        let site = client.site_patch(4, &body).await.unwrap();
        assert_eq!(site.name, "Patched");
    }

    #[tokio::test]
    async fn site_delete_sends_delete() {
        let server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/api/dcim/sites/7/"))
            .and(header("Authorization", "Token secret"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        client.site_delete(7).await.unwrap();
    }

    #[tokio::test]
    async fn site_delete_404_returns_api_error() {
        let server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/api/dcim/sites/999/"))
            .respond_with(
                ResponseTemplate::new(404)
                    .set_body_json(serde_json::json!({"detail": "Not found."})),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let err = client.site_delete(999).await.unwrap_err();
        assert!(matches!(err, crate::Error::Api { .. }));
    }

    // ── Interface tests ───────────────────────────────────────────────────────

    fn device_json(id: i64, name: &str) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "url": format!("https://netbox.example.com/api/dcim/devices/{id}/"),
            "display": name,
            "name": name,
            "description": ""
        })
    }

    fn iface_json(id: i64, name: &str) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "url": format!("https://netbox.example.com/api/dcim/interfaces/{id}/"),
            "display_url": format!("https://netbox.example.com/dcim/interfaces/{id}/"),
            "display": name,
            "device": device_json(1, "router-1"),
            "vdcs": [],
            "module": null,
            "name": name,
            "label": "",
            "type": {"value": "1000base-t", "label": "1000BASE-T (1GE)"},
            "enabled": true,
            "parent": null,
            "bridge": null,
            "bridge_interfaces": [],
            "lag": null,
            "mtu": null,
            "mac_address": null,
            "primary_mac_address": null,
            "mac_addresses": [],
            "speed": null,
            "duplex": null,
            "wwn": null,
            "mgmt_only": false,
            "description": "",
            "mode": null,
            "rf_role": null,
            "rf_channel": null,
            "poe_mode": null,
            "poe_type": null,
            "rf_channel_frequency": null,
            "rf_channel_width": null,
            "tx_power": null,
            "untagged_vlan": null,
            "tagged_vlans": [],
            "qinq_svlan": null,
            "vlan_translation_policy": null,
            "mark_connected": false,
            "cable": null,
            "cable_end": "",
            "wireless_link": null,
            "link_peers": [],
            "link_peers_type": null,
            "wireless_lans": [],
            "vrf": null,
            "l2vpn_termination": null,
            "connected_endpoints": null,
            "connected_endpoints_type": null,
            "connected_endpoints_reachable": false,
            "owner": null,
            "tags": [],
            "custom_fields": {},
            "created": "2024-01-01T00:00:00Z",
            "last_updated": "2024-01-01T00:00:00Z",
            "count_ipaddresses": 0,
            "count_fhrp_groups": 0,
            "_occupied": false
        })
    }

    #[tokio::test]
    async fn interfaces_list_returns_page() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/dcim/interfaces/"))
            .and(header("Authorization", "Token secret"))
            .and(query_param("limit", "50"))
            .and(query_param("offset", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 1,
                "next": null,
                "previous": null,
                "results": [iface_json(1, "eth0")]
            })))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let page = client
            .interfaces_list(50, 0, &InterfaceFilter::default())
            .await
            .unwrap();
        assert_eq!(page.count, 1);
        assert_eq!(page.results[0].name, "eth0");
    }

    #[tokio::test]
    async fn interfaces_stream_walks_two_pages() {
        use futures_util::TryStreamExt as _;

        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/dcim/interfaces/"))
            .and(query_param("offset", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 2,
                "next": format!("{}/api/dcim/interfaces/?limit=50&offset=1", server.uri()),
                "previous": null,
                "results": [iface_json(1, "eth0")]
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/api/dcim/interfaces/"))
            .and(query_param("offset", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 2,
                "next": null,
                "previous": null,
                "results": [iface_json(2, "eth1")]
            })))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let ifaces: Vec<Interface> = client
            .interfaces(&InterfaceFilter::default())
            .try_collect()
            .await
            .unwrap();
        assert_eq!(ifaces.len(), 2);
        assert_eq!(ifaces[0].name, "eth0");
        assert_eq!(ifaces[1].name, "eth1");
    }

    #[tokio::test]
    async fn interface_retrieve_returns_interface() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/dcim/interfaces/1/"))
            .and(header("Authorization", "Token secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(iface_json(1, "eth0")))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let iface = client.interface(1).await.unwrap();
        assert_eq!(iface.id, 1);
        assert_eq!(iface.name, "eth0");
    }

    #[tokio::test]
    async fn interface_retrieve_404_returns_api_error() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/dcim/interfaces/999/"))
            .respond_with(
                ResponseTemplate::new(404)
                    .set_body_json(serde_json::json!({"detail": "Not found."})),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let err = client.interface(999).await.unwrap_err();
        match err {
            crate::Error::Api { status, body } => {
                assert_eq!(status, reqwest::StatusCode::NOT_FOUND);
                assert_eq!(body.detail.as_deref(), Some("Not found."));
            }
            other => panic!("expected Api error, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn interface_create_sends_correct_body() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/dcim/interfaces/"))
            .and(header("Authorization", "Token secret"))
            .and(header("Content-Type", "application/json"))
            .respond_with(ResponseTemplate::new(201).set_body_json(iface_json(5, "eth0")))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let body = InterfaceRequest {
            device: 1,
            name: "eth0".into(),
            r#type: "1000base-t".into(),
            ..Default::default()
        };
        let iface = client.interface_create(&body).await.unwrap();
        assert_eq!(iface.name, "eth0");
    }

    #[tokio::test]
    async fn interface_create_400_returns_api_error() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/dcim/interfaces/"))
            .respond_with(
                ResponseTemplate::new(400)
                    .set_body_json(serde_json::json!({"name": ["This field is required."]})),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let body = InterfaceRequest {
            device: 1,
            name: String::new(),
            r#type: "virtual".into(),
            ..Default::default()
        };
        let err = client.interface_create(&body).await.unwrap_err();
        assert!(matches!(err, crate::Error::Api { .. }));
    }

    #[tokio::test]
    async fn interface_update_sends_put() {
        let server = MockServer::start().await;

        Mock::given(method("PUT"))
            .and(path("/api/dcim/interfaces/3/"))
            .and(header("Authorization", "Token secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(iface_json(3, "eth2")))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let body = InterfaceRequest {
            device: 1,
            name: "eth2".into(),
            r#type: "virtual".into(),
            ..Default::default()
        };
        let iface = client.interface_update(3, &body).await.unwrap();
        assert_eq!(iface.name, "eth2");
    }

    #[tokio::test]
    async fn interface_patch_sends_patch() {
        let server = MockServer::start().await;

        Mock::given(method("PATCH"))
            .and(path("/api/dcim/interfaces/4/"))
            .and(header("Authorization", "Token secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(iface_json(4, "mgmt0")))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let body = InterfacePatchRequest {
            name: Some("mgmt0".into()),
            mgmt_only: Some(true),
            ..Default::default()
        };
        let iface = client.interface_patch(4, &body).await.unwrap();
        assert_eq!(iface.name, "mgmt0");
    }

    #[tokio::test]
    async fn interface_delete_sends_delete() {
        let server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/api/dcim/interfaces/7/"))
            .and(header("Authorization", "Token secret"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        client.interface_delete(7).await.unwrap();
    }

    #[tokio::test]
    async fn interface_delete_404_returns_api_error() {
        let server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/api/dcim/interfaces/999/"))
            .respond_with(
                ResponseTemplate::new(404)
                    .set_body_json(serde_json::json!({"detail": "Not found."})),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let err = client.interface_delete(999).await.unwrap_err();
        assert!(matches!(err, crate::Error::Api { .. }));
    }

    // ── MAC address tests ──────────────────────────────────────────────────────

    fn mac_json(id: i64, mac: &str) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "url": format!("https://netbox.example.com/api/dcim/mac-addresses/{id}/"),
            "display_url": format!("https://netbox.example.com/dcim/mac-addresses/{id}/"),
            "display": mac,
            "mac_address": mac,
            "assigned_object_type": null,
            "assigned_object_id": null,
            "assigned_object": null,
            "description": "",
            "comments": "",
            "tags": [],
            "custom_fields": null,
            "created": "2024-01-01T00:00:00.000000Z",
            "last_updated": "2024-01-01T00:00:00.000000Z"
        })
    }

    #[tokio::test]
    async fn mac_addresses_list_returns_results() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/dcim/mac-addresses/"))
            .and(header("Authorization", "Token secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 1,
                "next": null,
                "previous": null,
                "results": [mac_json(1, "aa:bb:cc:dd:ee:ff")]
            })))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let page = client
            .mac_addresses_list(50, 0, &MACAddressFilter::default())
            .await
            .unwrap();
        assert_eq!(page.count, 1);
        assert_eq!(page.results[0].mac_address, "aa:bb:cc:dd:ee:ff");
    }

    #[tokio::test]
    async fn mac_addresses_stream_walks_two_pages() {
        use futures_util::TryStreamExt as _;

        let server = MockServer::start().await;

        // first page — has a `next`
        Mock::given(method("GET"))
            .and(path("/api/dcim/mac-addresses/"))
            .and(wiremock::matchers::query_param("offset", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 2,
                "next": "https://netbox.example.com/api/dcim/mac-addresses/?offset=50",
                "previous": null,
                "results": [mac_json(1, "aa:bb:cc:dd:ee:ff")]
            })))
            .mount(&server)
            .await;

        // second page — no next
        Mock::given(method("GET"))
            .and(path("/api/dcim/mac-addresses/"))
            .and(wiremock::matchers::query_param("offset", "50"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 2,
                "next": null,
                "previous": "https://netbox.example.com/api/dcim/mac-addresses/?offset=0",
                "results": [mac_json(2, "11:22:33:44:55:66")]
            })))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let all: Vec<_> = client
            .mac_addresses(&MACAddressFilter::default())
            .try_collect()
            .await
            .unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[1].mac_address, "11:22:33:44:55:66");
    }

    #[tokio::test]
    async fn mac_address_retrieve_returns_object() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/dcim/mac-addresses/42/"))
            .and(header("Authorization", "Token secret"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(mac_json(42, "de:ad:be:ef:00:01")),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let mac = client.mac_address(42).await.unwrap();
        assert_eq!(mac.id, 42);
        assert_eq!(mac.mac_address, "de:ad:be:ef:00:01");
    }

    #[tokio::test]
    async fn mac_address_retrieve_404_returns_api_error() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/dcim/mac-addresses/999/"))
            .respond_with(
                ResponseTemplate::new(404)
                    .set_body_json(serde_json::json!({"detail": "Not found."})),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let err = client.mac_address(999).await.unwrap_err();
        assert!(matches!(err, crate::Error::Api { .. }));
    }

    #[tokio::test]
    async fn mac_address_create_sends_body() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/dcim/mac-addresses/"))
            .and(header("Authorization", "Token secret"))
            .and(wiremock::matchers::body_json(serde_json::json!({
                "mac_address": "aa:bb:cc:dd:ee:ff",
                "assigned_object_type": "dcim.interface",
                "assigned_object_id": 10
            })))
            .respond_with(
                ResponseTemplate::new(201).set_body_json(mac_json(5, "aa:bb:cc:dd:ee:ff")),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let body = MACAddressRequest {
            mac_address: "aa:bb:cc:dd:ee:ff".into(),
            assigned_object_type: Some("dcim.interface".into()),
            assigned_object_id: Some(10),
            ..Default::default()
        };
        let mac = client.mac_address_create(&body).await.unwrap();
        assert_eq!(mac.id, 5);
    }

    #[tokio::test]
    async fn mac_address_patch_sends_body() {
        let server = MockServer::start().await;

        Mock::given(method("PATCH"))
            .and(path("/api/dcim/mac-addresses/5/"))
            .and(header("Authorization", "Token secret"))
            .and(wiremock::matchers::body_json(serde_json::json!({
                "description": "primary uplink"
            })))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(mac_json(5, "aa:bb:cc:dd:ee:ff")),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let patch = MACAddressPatchRequest {
            description: Some("primary uplink".into()),
            ..Default::default()
        };
        let mac = client.mac_address_patch(5, &patch).await.unwrap();
        assert_eq!(mac.id, 5);
    }

    #[tokio::test]
    async fn mac_address_delete_succeeds() {
        let server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/api/dcim/mac-addresses/5/"))
            .and(header("Authorization", "Token secret"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        client.mac_address_delete(5).await.unwrap();
    }

    #[tokio::test]
    async fn mac_address_delete_404_returns_api_error() {
        let server = MockServer::start().await;

        Mock::given(method("DELETE"))
            .and(path("/api/dcim/mac-addresses/999/"))
            .respond_with(
                ResponseTemplate::new(404)
                    .set_body_json(serde_json::json!({"detail": "Not found."})),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let err = client.mac_address_delete(999).await.unwrap_err();
        assert!(matches!(err, crate::Error::Api { .. }));
    }
}
