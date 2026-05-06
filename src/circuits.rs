//! Methods for the `circuits` tag of the NetBox REST API.
//!
//! # Example
//!
//! ```no_run
//! # async fn example() -> netbox_client::Result<()> {
//! use futures_util::TryStreamExt as _;
//! use netbox_client::circuits::CircuitFilter;
//!
//! let client = netbox_client::NetboxClient::new("https://netbox.example.com", "abc123")?;
//!
//! // Stream all active circuits
//! let filter = CircuitFilter { status: vec!["active".into()], ..Default::default() };
//! let circuits: Vec<_> = client.circuits(&filter).try_collect().await?;
//! println!("Found {} active circuits", circuits.len());
//! # Ok(()) }
//! ```

// Page sizes from NetBox are always far smaller than 2^32 items.
#![allow(clippy::cast_possible_truncation)]

use std::collections::VecDeque;

use futures_core::stream::BoxStream;
use futures_util::stream::try_unfold;

use crate::RequestBuilderExt as _;
use crate::{
    Circuit, CircuitGroup, CircuitGroupAssignment, CircuitGroupAssignmentPatchRequest,
    CircuitGroupAssignmentRequest, CircuitGroupPatchRequest, CircuitGroupRequest,
    CircuitPatchRequest, CircuitRequest, CircuitTermination, CircuitTerminationPatchRequest,
    CircuitTerminationRequest, CircuitType, CircuitTypePatchRequest, CircuitTypeRequest, JsonValue,
    NetboxClient, Paginated, Provider, ProviderAccount, ProviderAccountPatchRequest,
    ProviderAccountRequest, ProviderNetwork, ProviderNetworkPatchRequest, ProviderNetworkRequest,
    ProviderPatchRequest, ProviderRequest, VirtualCircuit, VirtualCircuitPatchRequest,
    VirtualCircuitRequest, VirtualCircuitTermination, VirtualCircuitTerminationPatchRequest,
    VirtualCircuitTerminationRequest, VirtualCircuitType, VirtualCircuitTypePatchRequest,
    VirtualCircuitTypeRequest,
};

const PAGE_SIZE: u32 = 50;

// ── Filter types ─────────────────────────────────────────────────────────────

/// Filters for the [`NetboxClient::circuits_list`] / [`NetboxClient::circuits`] endpoints.
#[derive(Debug, Clone, Default)]
pub struct CircuitFilter {
    /// Free-text search.
    pub q: Option<String>,
    /// Limit results to these IDs.
    pub id: Vec<i64>,
    /// Filter by status (e.g. `active`, `planned`).
    pub status: Vec<String>,
    /// Filter by provider slug.
    pub provider: Vec<String>,
    /// Filter by provider ID.
    pub provider_id: Vec<i64>,
    /// Filter by circuit type slug.
    pub r#type: Vec<String>,
    /// Filter by circuit type ID.
    pub type_id: Vec<i64>,
    /// Filter by tenant slug.
    pub tenant: Vec<String>,
    /// Filter by tenant ID.
    pub tenant_id: Vec<i64>,
    /// Filter by circuit ID string (exact).
    pub cid: Vec<String>,
}

impl CircuitFilter {
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
        for v in &self.provider {
            p.push(("provider".into(), v.clone()));
        }
        for v in &self.provider_id {
            p.push(("provider_id".into(), v.to_string()));
        }
        for v in &self.r#type {
            p.push(("type".into(), v.clone()));
        }
        for v in &self.type_id {
            p.push(("type_id".into(), v.to_string()));
        }
        for v in &self.tenant {
            p.push(("tenant".into(), v.clone()));
        }
        for v in &self.tenant_id {
            p.push(("tenant_id".into(), v.to_string()));
        }
        for v in &self.cid {
            p.push(("cid".into(), v.clone()));
        }
        p
    }
}

/// Filters for the circuit-types endpoints.
#[derive(Debug, Clone, Default)]
pub struct CircuitTypeFilter {
    /// Free-text search.
    pub q: Option<String>,
    /// Limit results to these IDs.
    pub id: Vec<i64>,
    /// Filter by name.
    pub name: Vec<String>,
    /// Filter by slug.
    pub slug: Vec<String>,
}

impl CircuitTypeFilter {
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
        p
    }
}

/// Filters for the circuit-terminations endpoints.
#[derive(Debug, Clone, Default)]
pub struct CircuitTerminationFilter {
    /// Free-text search.
    pub q: Option<String>,
    /// Limit results to these IDs.
    pub id: Vec<i64>,
    /// Filter by circuit ID.
    pub circuit_id: Vec<i64>,
    /// Filter by termination side (`A` or `Z`).
    pub term_side: Option<String>,
}

impl CircuitTerminationFilter {
    fn as_query(&self) -> Vec<(String, String)> {
        let mut p = Vec::new();
        if let Some(q) = &self.q {
            p.push(("q".into(), q.clone()));
        }
        for v in &self.id {
            p.push(("id".into(), v.to_string()));
        }
        for v in &self.circuit_id {
            p.push(("circuit_id".into(), v.to_string()));
        }
        if let Some(ts) = &self.term_side {
            p.push(("term_side".into(), ts.clone()));
        }
        p
    }
}

/// Filters for the providers endpoints.
#[derive(Debug, Clone, Default)]
pub struct ProviderFilter {
    /// Free-text search.
    pub q: Option<String>,
    /// Limit results to these IDs.
    pub id: Vec<i64>,
    /// Filter by name.
    pub name: Vec<String>,
    /// Filter by slug.
    pub slug: Vec<String>,
}

impl ProviderFilter {
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
        p
    }
}

/// Filters for the provider-networks endpoints.
#[derive(Debug, Clone, Default)]
pub struct ProviderNetworkFilter {
    /// Free-text search.
    pub q: Option<String>,
    /// Limit results to these IDs.
    pub id: Vec<i64>,
    /// Filter by provider ID.
    pub provider_id: Vec<i64>,
    /// Filter by provider slug.
    pub provider: Vec<String>,
    /// Filter by name.
    pub name: Vec<String>,
}

impl ProviderNetworkFilter {
    fn as_query(&self) -> Vec<(String, String)> {
        let mut p = Vec::new();
        if let Some(q) = &self.q {
            p.push(("q".into(), q.clone()));
        }
        for v in &self.id {
            p.push(("id".into(), v.to_string()));
        }
        for v in &self.provider_id {
            p.push(("provider_id".into(), v.to_string()));
        }
        for v in &self.provider {
            p.push(("provider".into(), v.clone()));
        }
        for v in &self.name {
            p.push(("name".into(), v.clone()));
        }
        p
    }
}

/// Filters for the provider-accounts endpoints.
#[derive(Debug, Clone, Default)]
pub struct ProviderAccountFilter {
    /// Free-text search.
    pub q: Option<String>,
    /// Limit results to these IDs.
    pub id: Vec<i64>,
    /// Filter by provider ID.
    pub provider_id: Vec<i64>,
    /// Filter by provider slug.
    pub provider: Vec<String>,
}

impl ProviderAccountFilter {
    fn as_query(&self) -> Vec<(String, String)> {
        let mut p = Vec::new();
        if let Some(q) = &self.q {
            p.push(("q".into(), q.clone()));
        }
        for v in &self.id {
            p.push(("id".into(), v.to_string()));
        }
        for v in &self.provider_id {
            p.push(("provider_id".into(), v.to_string()));
        }
        for v in &self.provider {
            p.push(("provider".into(), v.clone()));
        }
        p
    }
}

/// Filters for the circuit-groups endpoints.
#[derive(Debug, Clone, Default)]
pub struct CircuitGroupFilter {
    /// Free-text search.
    pub q: Option<String>,
    /// Limit results to these IDs.
    pub id: Vec<i64>,
    /// Filter by name.
    pub name: Vec<String>,
    /// Filter by slug.
    pub slug: Vec<String>,
}

impl CircuitGroupFilter {
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
        p
    }
}

/// Filters for the circuit-group-assignments endpoints.
#[derive(Debug, Clone, Default)]
pub struct CircuitGroupAssignmentFilter {
    /// Free-text search.
    pub q: Option<String>,
    /// Limit results to these IDs.
    pub id: Vec<i64>,
    /// Filter by circuit group ID.
    pub group_id: Vec<i64>,
}

impl CircuitGroupAssignmentFilter {
    fn as_query(&self) -> Vec<(String, String)> {
        let mut p = Vec::new();
        if let Some(q) = &self.q {
            p.push(("q".into(), q.clone()));
        }
        for v in &self.id {
            p.push(("id".into(), v.to_string()));
        }
        for v in &self.group_id {
            p.push(("group_id".into(), v.to_string()));
        }
        p
    }
}

/// Filters for the virtual-circuits endpoints.
#[derive(Debug, Clone, Default)]
pub struct VirtualCircuitFilter {
    /// Free-text search.
    pub q: Option<String>,
    /// Limit results to these IDs.
    pub id: Vec<i64>,
    /// Filter by status (e.g. `active`, `planned`).
    pub status: Vec<String>,
    /// Filter by provider network ID.
    pub provider_network_id: Vec<i64>,
    /// Filter by circuit ID string (exact).
    pub cid: Vec<String>,
}

impl VirtualCircuitFilter {
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
        for v in &self.provider_network_id {
            p.push(("provider_network_id".into(), v.to_string()));
        }
        for v in &self.cid {
            p.push(("cid".into(), v.clone()));
        }
        p
    }
}

/// Filters for the virtual-circuit-types endpoints.
#[derive(Debug, Clone, Default)]
pub struct VirtualCircuitTypeFilter {
    /// Free-text search.
    pub q: Option<String>,
    /// Limit results to these IDs.
    pub id: Vec<i64>,
    /// Filter by name.
    pub name: Vec<String>,
    /// Filter by slug.
    pub slug: Vec<String>,
}

impl VirtualCircuitTypeFilter {
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
        p
    }
}

/// Filters for the virtual-circuit-terminations endpoints.
#[derive(Debug, Clone, Default)]
pub struct VirtualCircuitTerminationFilter {
    /// Free-text search.
    pub q: Option<String>,
    /// Limit results to these IDs.
    pub id: Vec<i64>,
    /// Filter by virtual circuit ID.
    pub virtual_circuit_id: Vec<i64>,
}

impl VirtualCircuitTerminationFilter {
    fn as_query(&self) -> Vec<(String, String)> {
        let mut p = Vec::new();
        if let Some(q) = &self.q {
            p.push(("q".into(), q.clone()));
        }
        for v in &self.id {
            p.push(("id".into(), v.to_string()));
        }
        for v in &self.virtual_circuit_id {
            p.push(("virtual_circuit_id".into(), v.to_string()));
        }
        p
    }
}

// ── NetboxClient implementation ───────────────────────────────────────────────

impl NetboxClient {
    // ── Providers ────────────────────────────────────────────────────────────

    /// Returns a single page of providers.
    ///
    /// Maps to `GET /api/circuits/providers/`
    /// (`operationId`: `circuits_providers_list`).
    ///
    /// # Errors
    ///
    /// Returns an error on HTTP or deserialization failure.
    pub async fn providers_list(
        &self,
        limit: u32,
        offset: u32,
        filter: &ProviderFilter,
    ) -> crate::Result<Paginated<Provider>> {
        let url = format!("{}/api/circuits/providers/", self.base_url);
        let mut query = filter.as_query();
        query.push(("limit".into(), limit.to_string()));
        query.push(("offset".into(), offset.to_string()));
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .query(&query)
            .send_json::<Paginated<Provider>>()
            .await
    }

    /// Streams all providers matching `filter`, auto-paginating.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> netbox_client::Result<()> {
    /// use futures_util::TryStreamExt as _;
    /// use netbox_client::circuits::ProviderFilter;
    /// let client = netbox_client::NetboxClient::new("https://netbox.example.com", "token")?;
    /// let all: Vec<_> = client.providers(&ProviderFilter::default()).try_collect().await?;
    /// # Ok(()) }
    /// ```
    #[must_use]
    pub fn providers<'a>(
        &'a self,
        filter: &'a ProviderFilter,
    ) -> BoxStream<'a, crate::Result<Provider>> {
        Box::pin(try_unfold(
            (Some(0u32), VecDeque::<Provider>::new()),
            move |(next_offset, mut buf)| async move {
                if let Some(item) = buf.pop_front() {
                    return Ok(Some((item, (next_offset, buf))));
                }
                let Some(offset) = next_offset else {
                    return Ok(None);
                };
                let page = self.providers_list(PAGE_SIZE, offset, filter).await?;
                let new_next = page
                    .next
                    .is_some()
                    .then_some(offset + page.results.len() as u32);
                let mut buf: VecDeque<Provider> = page.results.into_iter().collect();
                match buf.pop_front() {
                    Some(item) => Ok(Some((item, (new_next, buf)))),
                    None => Ok(None),
                }
            },
        ))
    }

    /// Returns a single provider by ID.
    ///
    /// Maps to `GET /api/circuits/providers/{id}/`
    /// (`operationId`: `circuits_providers_retrieve`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 404 if the provider does not exist.
    pub async fn provider(&self, id: i64) -> crate::Result<Provider> {
        let url = format!("{}/api/circuits/providers/{id}/", self.base_url);
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .send_json::<Provider>()
            .await
    }

    /// Creates a new provider.
    ///
    /// Maps to `POST /api/circuits/providers/`
    /// (`operationId`: `circuits_providers_create`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn provider_create(&self, body: &ProviderRequest) -> crate::Result<Provider> {
        let url = format!("{}/api/circuits/providers/", self.base_url);
        self.http
            .post(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<Provider>()
            .await
    }

    /// Replaces a provider (full update).
    ///
    /// Maps to `PUT /api/circuits/providers/{id}/`
    /// (`operationId`: `circuits_providers_update`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn provider_update(
        &self,
        id: i64,
        body: &ProviderRequest,
    ) -> crate::Result<Provider> {
        let url = format!("{}/api/circuits/providers/{id}/", self.base_url);
        self.http
            .put(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<Provider>()
            .await
    }

    /// Partially updates a provider.
    ///
    /// Maps to `PATCH /api/circuits/providers/{id}/`
    /// (`operationId`: `circuits_providers_partial_update`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn provider_patch(
        &self,
        id: i64,
        body: &ProviderPatchRequest,
    ) -> crate::Result<Provider> {
        let url = format!("{}/api/circuits/providers/{id}/", self.base_url);
        self.http
            .patch(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<Provider>()
            .await
    }

    /// Deletes a provider.
    ///
    /// Maps to `DELETE /api/circuits/providers/{id}/`
    /// (`operationId`: `circuits_providers_destroy`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 404 if the provider does not exist.
    pub async fn provider_delete(&self, id: i64) -> crate::Result<()> {
        let url = format!("{}/api/circuits/providers/{id}/", self.base_url);
        // DELETE returns 204 No Content — send_json expects a body, so handle manually.
        let resp = self
            .http
            .delete(&url)
            .header("Authorization", format!("Token {}", self.token))
            .send()
            .await
            .map_err(crate::Error::Http)?;
        if resp.status().is_success() {
            Ok(())
        } else {
            let status = resp.status();
            let bytes = resp.bytes().await.map_err(crate::Error::Http)?;
            let body = crate::json::from_slice::<crate::ApiError>(&bytes).unwrap_or_default();
            Err(crate::Error::Api { status, body })
        }
    }

    // ── Provider Accounts ─────────────────────────────────────────────────────

    /// Returns a single page of provider accounts.
    ///
    /// Maps to `GET /api/circuits/provider-accounts/`
    /// (`operationId`: `circuits_provider_accounts_list`).
    ///
    /// # Errors
    ///
    /// Returns an error on HTTP or deserialization failure.
    pub async fn provider_accounts_list(
        &self,
        limit: u32,
        offset: u32,
        filter: &ProviderAccountFilter,
    ) -> crate::Result<Paginated<ProviderAccount>> {
        let url = format!("{}/api/circuits/provider-accounts/", self.base_url);
        let mut query = filter.as_query();
        query.push(("limit".into(), limit.to_string()));
        query.push(("offset".into(), offset.to_string()));
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .query(&query)
            .send_json::<Paginated<ProviderAccount>>()
            .await
    }

    /// Streams all provider accounts matching `filter`, auto-paginating.
    #[must_use]
    pub fn provider_accounts<'a>(
        &'a self,
        filter: &'a ProviderAccountFilter,
    ) -> BoxStream<'a, crate::Result<ProviderAccount>> {
        Box::pin(try_unfold(
            (Some(0u32), VecDeque::<ProviderAccount>::new()),
            move |(next_offset, mut buf)| async move {
                if let Some(item) = buf.pop_front() {
                    return Ok(Some((item, (next_offset, buf))));
                }
                let Some(offset) = next_offset else {
                    return Ok(None);
                };
                let page = self
                    .provider_accounts_list(PAGE_SIZE, offset, filter)
                    .await?;
                let new_next = page
                    .next
                    .is_some()
                    .then_some(offset + page.results.len() as u32);
                let mut buf: VecDeque<ProviderAccount> = page.results.into_iter().collect();
                match buf.pop_front() {
                    Some(item) => Ok(Some((item, (new_next, buf)))),
                    None => Ok(None),
                }
            },
        ))
    }

    /// Returns a single provider account by ID.
    ///
    /// Maps to `GET /api/circuits/provider-accounts/{id}/`
    /// (`operationId`: `circuits_provider_accounts_retrieve`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 404 if not found.
    pub async fn provider_account(&self, id: i64) -> crate::Result<ProviderAccount> {
        let url = format!("{}/api/circuits/provider-accounts/{id}/", self.base_url);
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .send_json::<ProviderAccount>()
            .await
    }

    /// Creates a new provider account.
    ///
    /// Maps to `POST /api/circuits/provider-accounts/`
    /// (`operationId`: `circuits_provider_accounts_create`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn provider_account_create(
        &self,
        body: &ProviderAccountRequest,
    ) -> crate::Result<ProviderAccount> {
        let url = format!("{}/api/circuits/provider-accounts/", self.base_url);
        self.http
            .post(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<ProviderAccount>()
            .await
    }

    /// Replaces a provider account (full update).
    ///
    /// Maps to `PUT /api/circuits/provider-accounts/{id}/`
    /// (`operationId`: `circuits_provider_accounts_update`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn provider_account_update(
        &self,
        id: i64,
        body: &ProviderAccountRequest,
    ) -> crate::Result<ProviderAccount> {
        let url = format!("{}/api/circuits/provider-accounts/{id}/", self.base_url);
        self.http
            .put(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<ProviderAccount>()
            .await
    }

    /// Partially updates a provider account.
    ///
    /// Maps to `PATCH /api/circuits/provider-accounts/{id}/`
    /// (`operationId`: `circuits_provider_accounts_partial_update`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn provider_account_patch(
        &self,
        id: i64,
        body: &ProviderAccountPatchRequest,
    ) -> crate::Result<ProviderAccount> {
        let url = format!("{}/api/circuits/provider-accounts/{id}/", self.base_url);
        self.http
            .patch(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<ProviderAccount>()
            .await
    }

    /// Deletes a provider account.
    ///
    /// Maps to `DELETE /api/circuits/provider-accounts/{id}/`
    /// (`operationId`: `circuits_provider_accounts_destroy`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 404 if not found.
    pub async fn provider_account_delete(&self, id: i64) -> crate::Result<()> {
        let url = format!("{}/api/circuits/provider-accounts/{id}/", self.base_url);
        crate::delete_no_content(&self.http, &url, &self.token).await
    }

    // ── Provider Networks ─────────────────────────────────────────────────────

    /// Returns a single page of provider networks.
    ///
    /// Maps to `GET /api/circuits/provider-networks/`
    /// (`operationId`: `circuits_provider_networks_list`).
    ///
    /// # Errors
    ///
    /// Returns an error on HTTP or deserialization failure.
    pub async fn provider_networks_list(
        &self,
        limit: u32,
        offset: u32,
        filter: &ProviderNetworkFilter,
    ) -> crate::Result<Paginated<ProviderNetwork>> {
        let url = format!("{}/api/circuits/provider-networks/", self.base_url);
        let mut query = filter.as_query();
        query.push(("limit".into(), limit.to_string()));
        query.push(("offset".into(), offset.to_string()));
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .query(&query)
            .send_json::<Paginated<ProviderNetwork>>()
            .await
    }

    /// Streams all provider networks matching `filter`, auto-paginating.
    #[must_use]
    pub fn provider_networks<'a>(
        &'a self,
        filter: &'a ProviderNetworkFilter,
    ) -> BoxStream<'a, crate::Result<ProviderNetwork>> {
        Box::pin(try_unfold(
            (Some(0u32), VecDeque::<ProviderNetwork>::new()),
            move |(next_offset, mut buf)| async move {
                if let Some(item) = buf.pop_front() {
                    return Ok(Some((item, (next_offset, buf))));
                }
                let Some(offset) = next_offset else {
                    return Ok(None);
                };
                let page = self
                    .provider_networks_list(PAGE_SIZE, offset, filter)
                    .await?;
                let new_next = page
                    .next
                    .is_some()
                    .then_some(offset + page.results.len() as u32);
                let mut buf: VecDeque<ProviderNetwork> = page.results.into_iter().collect();
                match buf.pop_front() {
                    Some(item) => Ok(Some((item, (new_next, buf)))),
                    None => Ok(None),
                }
            },
        ))
    }

    /// Returns a single provider network by ID.
    ///
    /// Maps to `GET /api/circuits/provider-networks/{id}/`
    /// (`operationId`: `circuits_provider_networks_retrieve`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 404 if not found.
    pub async fn provider_network(&self, id: i64) -> crate::Result<ProviderNetwork> {
        let url = format!("{}/api/circuits/provider-networks/{id}/", self.base_url);
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .send_json::<ProviderNetwork>()
            .await
    }

    /// Creates a new provider network.
    ///
    /// Maps to `POST /api/circuits/provider-networks/`
    /// (`operationId`: `circuits_provider_networks_create`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn provider_network_create(
        &self,
        body: &ProviderNetworkRequest,
    ) -> crate::Result<ProviderNetwork> {
        let url = format!("{}/api/circuits/provider-networks/", self.base_url);
        self.http
            .post(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<ProviderNetwork>()
            .await
    }

    /// Replaces a provider network (full update).
    ///
    /// Maps to `PUT /api/circuits/provider-networks/{id}/`
    /// (`operationId`: `circuits_provider_networks_update`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn provider_network_update(
        &self,
        id: i64,
        body: &ProviderNetworkRequest,
    ) -> crate::Result<ProviderNetwork> {
        let url = format!("{}/api/circuits/provider-networks/{id}/", self.base_url);
        self.http
            .put(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<ProviderNetwork>()
            .await
    }

    /// Partially updates a provider network.
    ///
    /// Maps to `PATCH /api/circuits/provider-networks/{id}/`
    /// (`operationId`: `circuits_provider_networks_partial_update`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn provider_network_patch(
        &self,
        id: i64,
        body: &ProviderNetworkPatchRequest,
    ) -> crate::Result<ProviderNetwork> {
        let url = format!("{}/api/circuits/provider-networks/{id}/", self.base_url);
        self.http
            .patch(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<ProviderNetwork>()
            .await
    }

    /// Deletes a provider network.
    ///
    /// Maps to `DELETE /api/circuits/provider-networks/{id}/`
    /// (`operationId`: `circuits_provider_networks_destroy`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 404 if not found.
    pub async fn provider_network_delete(&self, id: i64) -> crate::Result<()> {
        let url = format!("{}/api/circuits/provider-networks/{id}/", self.base_url);
        crate::delete_no_content(&self.http, &url, &self.token).await
    }

    // ── Circuit Types ─────────────────────────────────────────────────────────

    /// Returns a single page of circuit types.
    ///
    /// Maps to `GET /api/circuits/circuit-types/`
    /// (`operationId`: `circuits_circuit_types_list`).
    ///
    /// # Errors
    ///
    /// Returns an error on HTTP or deserialization failure.
    pub async fn circuit_types_list(
        &self,
        limit: u32,
        offset: u32,
        filter: &CircuitTypeFilter,
    ) -> crate::Result<Paginated<CircuitType>> {
        let url = format!("{}/api/circuits/circuit-types/", self.base_url);
        let mut query = filter.as_query();
        query.push(("limit".into(), limit.to_string()));
        query.push(("offset".into(), offset.to_string()));
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .query(&query)
            .send_json::<Paginated<CircuitType>>()
            .await
    }

    /// Streams all circuit types matching `filter`, auto-paginating.
    #[must_use]
    pub fn circuit_types<'a>(
        &'a self,
        filter: &'a CircuitTypeFilter,
    ) -> BoxStream<'a, crate::Result<CircuitType>> {
        Box::pin(try_unfold(
            (Some(0u32), VecDeque::<CircuitType>::new()),
            move |(next_offset, mut buf)| async move {
                if let Some(item) = buf.pop_front() {
                    return Ok(Some((item, (next_offset, buf))));
                }
                let Some(offset) = next_offset else {
                    return Ok(None);
                };
                let page = self.circuit_types_list(PAGE_SIZE, offset, filter).await?;
                let new_next = page
                    .next
                    .is_some()
                    .then_some(offset + page.results.len() as u32);
                let mut buf: VecDeque<CircuitType> = page.results.into_iter().collect();
                match buf.pop_front() {
                    Some(item) => Ok(Some((item, (new_next, buf)))),
                    None => Ok(None),
                }
            },
        ))
    }

    /// Returns a single circuit type by ID.
    ///
    /// Maps to `GET /api/circuits/circuit-types/{id}/`
    /// (`operationId`: `circuits_circuit_types_retrieve`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 404 if not found.
    pub async fn circuit_type(&self, id: i64) -> crate::Result<CircuitType> {
        let url = format!("{}/api/circuits/circuit-types/{id}/", self.base_url);
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .send_json::<CircuitType>()
            .await
    }

    /// Creates a new circuit type.
    ///
    /// Maps to `POST /api/circuits/circuit-types/`
    /// (`operationId`: `circuits_circuit_types_create`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn circuit_type_create(
        &self,
        body: &CircuitTypeRequest,
    ) -> crate::Result<CircuitType> {
        let url = format!("{}/api/circuits/circuit-types/", self.base_url);
        self.http
            .post(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<CircuitType>()
            .await
    }

    /// Replaces a circuit type (full update).
    ///
    /// Maps to `PUT /api/circuits/circuit-types/{id}/`
    /// (`operationId`: `circuits_circuit_types_update`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn circuit_type_update(
        &self,
        id: i64,
        body: &CircuitTypeRequest,
    ) -> crate::Result<CircuitType> {
        let url = format!("{}/api/circuits/circuit-types/{id}/", self.base_url);
        self.http
            .put(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<CircuitType>()
            .await
    }

    /// Partially updates a circuit type.
    ///
    /// Maps to `PATCH /api/circuits/circuit-types/{id}/`
    /// (`operationId`: `circuits_circuit_types_partial_update`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn circuit_type_patch(
        &self,
        id: i64,
        body: &CircuitTypePatchRequest,
    ) -> crate::Result<CircuitType> {
        let url = format!("{}/api/circuits/circuit-types/{id}/", self.base_url);
        self.http
            .patch(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<CircuitType>()
            .await
    }

    /// Deletes a circuit type.
    ///
    /// Maps to `DELETE /api/circuits/circuit-types/{id}/`
    /// (`operationId`: `circuits_circuit_types_destroy`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 404 if not found.
    pub async fn circuit_type_delete(&self, id: i64) -> crate::Result<()> {
        let url = format!("{}/api/circuits/circuit-types/{id}/", self.base_url);
        crate::delete_no_content(&self.http, &url, &self.token).await
    }

    // ── Circuits ──────────────────────────────────────────────────────────────

    /// Returns a single page of circuits.
    ///
    /// Maps to `GET /api/circuits/circuits/`
    /// (`operationId`: `circuits_circuits_list`).
    ///
    /// # Errors
    ///
    /// Returns an error on HTTP or deserialization failure.
    pub async fn circuits_list(
        &self,
        limit: u32,
        offset: u32,
        filter: &CircuitFilter,
    ) -> crate::Result<Paginated<Circuit>> {
        let url = format!("{}/api/circuits/circuits/", self.base_url);
        let mut query = filter.as_query();
        query.push(("limit".into(), limit.to_string()));
        query.push(("offset".into(), offset.to_string()));
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .query(&query)
            .send_json::<Paginated<Circuit>>()
            .await
    }

    /// Streams all circuits matching `filter`, auto-paginating.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> netbox_client::Result<()> {
    /// use futures_util::TryStreamExt as _;
    /// use netbox_client::circuits::CircuitFilter;
    /// let client = netbox_client::NetboxClient::new("https://netbox.example.com", "token")?;
    /// let all: Vec<_> = client.circuits(&CircuitFilter::default()).try_collect().await?;
    /// # Ok(()) }
    /// ```
    #[must_use]
    pub fn circuits<'a>(
        &'a self,
        filter: &'a CircuitFilter,
    ) -> BoxStream<'a, crate::Result<Circuit>> {
        Box::pin(try_unfold(
            (Some(0u32), VecDeque::<Circuit>::new()),
            move |(next_offset, mut buf)| async move {
                if let Some(item) = buf.pop_front() {
                    return Ok(Some((item, (next_offset, buf))));
                }
                let Some(offset) = next_offset else {
                    return Ok(None);
                };
                let page = self.circuits_list(PAGE_SIZE, offset, filter).await?;
                let new_next = page
                    .next
                    .is_some()
                    .then_some(offset + page.results.len() as u32);
                let mut buf: VecDeque<Circuit> = page.results.into_iter().collect();
                match buf.pop_front() {
                    Some(item) => Ok(Some((item, (new_next, buf)))),
                    None => Ok(None),
                }
            },
        ))
    }

    /// Returns a single circuit by ID.
    ///
    /// Maps to `GET /api/circuits/circuits/{id}/`
    /// (`operationId`: `circuits_circuits_retrieve`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 404 if the circuit does not exist.
    pub async fn circuit(&self, id: i64) -> crate::Result<Circuit> {
        let url = format!("{}/api/circuits/circuits/{id}/", self.base_url);
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .send_json::<Circuit>()
            .await
    }

    /// Creates a new circuit.
    ///
    /// Maps to `POST /api/circuits/circuits/`
    /// (`operationId`: `circuits_circuits_create`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn circuit_create(&self, body: &CircuitRequest) -> crate::Result<Circuit> {
        let url = format!("{}/api/circuits/circuits/", self.base_url);
        self.http
            .post(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<Circuit>()
            .await
    }

    /// Replaces a circuit (full update).
    ///
    /// Maps to `PUT /api/circuits/circuits/{id}/`
    /// (`operationId`: `circuits_circuits_update`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn circuit_update(&self, id: i64, body: &CircuitRequest) -> crate::Result<Circuit> {
        let url = format!("{}/api/circuits/circuits/{id}/", self.base_url);
        self.http
            .put(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<Circuit>()
            .await
    }

    /// Partially updates a circuit.
    ///
    /// Maps to `PATCH /api/circuits/circuits/{id}/`
    /// (`operationId`: `circuits_circuits_partial_update`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn circuit_patch(
        &self,
        id: i64,
        body: &CircuitPatchRequest,
    ) -> crate::Result<Circuit> {
        let url = format!("{}/api/circuits/circuits/{id}/", self.base_url);
        self.http
            .patch(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<Circuit>()
            .await
    }

    /// Deletes a circuit.
    ///
    /// Maps to `DELETE /api/circuits/circuits/{id}/`
    /// (`operationId`: `circuits_circuits_destroy`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 404 if the circuit does not exist.
    pub async fn circuit_delete(&self, id: i64) -> crate::Result<()> {
        let url = format!("{}/api/circuits/circuits/{id}/", self.base_url);
        crate::delete_no_content(&self.http, &url, &self.token).await
    }

    // ── Circuit Terminations ──────────────────────────────────────────────────

    /// Returns a single page of circuit terminations.
    ///
    /// Maps to `GET /api/circuits/circuit-terminations/`
    /// (`operationId`: `circuits_circuit_terminations_list`).
    ///
    /// # Errors
    ///
    /// Returns an error on HTTP or deserialization failure.
    pub async fn circuit_terminations_list(
        &self,
        limit: u32,
        offset: u32,
        filter: &CircuitTerminationFilter,
    ) -> crate::Result<Paginated<CircuitTermination>> {
        let url = format!("{}/api/circuits/circuit-terminations/", self.base_url);
        let mut query = filter.as_query();
        query.push(("limit".into(), limit.to_string()));
        query.push(("offset".into(), offset.to_string()));
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .query(&query)
            .send_json::<Paginated<CircuitTermination>>()
            .await
    }

    /// Streams all circuit terminations matching `filter`, auto-paginating.
    #[must_use]
    pub fn circuit_terminations<'a>(
        &'a self,
        filter: &'a CircuitTerminationFilter,
    ) -> BoxStream<'a, crate::Result<CircuitTermination>> {
        Box::pin(try_unfold(
            (Some(0u32), VecDeque::<CircuitTermination>::new()),
            move |(next_offset, mut buf)| async move {
                if let Some(item) = buf.pop_front() {
                    return Ok(Some((item, (next_offset, buf))));
                }
                let Some(offset) = next_offset else {
                    return Ok(None);
                };
                let page = self
                    .circuit_terminations_list(PAGE_SIZE, offset, filter)
                    .await?;
                let new_next = page
                    .next
                    .is_some()
                    .then_some(offset + page.results.len() as u32);
                let mut buf: VecDeque<CircuitTermination> = page.results.into_iter().collect();
                match buf.pop_front() {
                    Some(item) => Ok(Some((item, (new_next, buf)))),
                    None => Ok(None),
                }
            },
        ))
    }

    /// Returns a single circuit termination by ID.
    ///
    /// Maps to `GET /api/circuits/circuit-terminations/{id}/`
    /// (`operationId`: `circuits_circuit_terminations_retrieve`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 404 if not found.
    pub async fn circuit_termination(&self, id: i64) -> crate::Result<CircuitTermination> {
        let url = format!("{}/api/circuits/circuit-terminations/{id}/", self.base_url);
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .send_json::<CircuitTermination>()
            .await
    }

    /// Creates a new circuit termination.
    ///
    /// Maps to `POST /api/circuits/circuit-terminations/`
    /// (`operationId`: `circuits_circuit_terminations_create`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn circuit_termination_create(
        &self,
        body: &CircuitTerminationRequest,
    ) -> crate::Result<CircuitTermination> {
        let url = format!("{}/api/circuits/circuit-terminations/", self.base_url);
        self.http
            .post(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<CircuitTermination>()
            .await
    }

    /// Replaces a circuit termination (full update).
    ///
    /// Maps to `PUT /api/circuits/circuit-terminations/{id}/`
    /// (`operationId`: `circuits_circuit_terminations_update`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn circuit_termination_update(
        &self,
        id: i64,
        body: &CircuitTerminationRequest,
    ) -> crate::Result<CircuitTermination> {
        let url = format!("{}/api/circuits/circuit-terminations/{id}/", self.base_url);
        self.http
            .put(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<CircuitTermination>()
            .await
    }

    /// Partially updates a circuit termination.
    ///
    /// Maps to `PATCH /api/circuits/circuit-terminations/{id}/`
    /// (`operationId`: `circuits_circuit_terminations_partial_update`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn circuit_termination_patch(
        &self,
        id: i64,
        body: &CircuitTerminationPatchRequest,
    ) -> crate::Result<CircuitTermination> {
        let url = format!("{}/api/circuits/circuit-terminations/{id}/", self.base_url);
        self.http
            .patch(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<CircuitTermination>()
            .await
    }

    /// Deletes a circuit termination.
    ///
    /// Maps to `DELETE /api/circuits/circuit-terminations/{id}/`
    /// (`operationId`: `circuits_circuit_terminations_destroy`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 404 if not found.
    pub async fn circuit_termination_delete(&self, id: i64) -> crate::Result<()> {
        let url = format!("{}/api/circuits/circuit-terminations/{id}/", self.base_url);
        crate::delete_no_content(&self.http, &url, &self.token).await
    }

    /// Returns the cable path(s) for a circuit termination.
    ///
    /// Maps to `GET /api/circuits/circuit-terminations/{id}/paths/`
    /// (`operationId`: `circuits_circuit_terminations_paths_retrieve`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 404 if not found.
    pub async fn circuit_termination_paths(&self, id: i64) -> crate::Result<JsonValue> {
        let url = format!(
            "{}/api/circuits/circuit-terminations/{id}/paths/",
            self.base_url
        );
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .send_json::<JsonValue>()
            .await
    }

    // ── Circuit Groups ────────────────────────────────────────────────────────

    /// Returns a single page of circuit groups.
    ///
    /// Maps to `GET /api/circuits/circuit-groups/`
    /// (`operationId`: `circuits_circuit_groups_list`).
    ///
    /// # Errors
    ///
    /// Returns an error on HTTP or deserialization failure.
    pub async fn circuit_groups_list(
        &self,
        limit: u32,
        offset: u32,
        filter: &CircuitGroupFilter,
    ) -> crate::Result<Paginated<CircuitGroup>> {
        let url = format!("{}/api/circuits/circuit-groups/", self.base_url);
        let mut query = filter.as_query();
        query.push(("limit".into(), limit.to_string()));
        query.push(("offset".into(), offset.to_string()));
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .query(&query)
            .send_json::<Paginated<CircuitGroup>>()
            .await
    }

    /// Streams all circuit groups matching `filter`, auto-paginating.
    #[must_use]
    pub fn circuit_groups<'a>(
        &'a self,
        filter: &'a CircuitGroupFilter,
    ) -> BoxStream<'a, crate::Result<CircuitGroup>> {
        Box::pin(try_unfold(
            (Some(0u32), VecDeque::<CircuitGroup>::new()),
            move |(next_offset, mut buf)| async move {
                if let Some(item) = buf.pop_front() {
                    return Ok(Some((item, (next_offset, buf))));
                }
                let Some(offset) = next_offset else {
                    return Ok(None);
                };
                let page = self.circuit_groups_list(PAGE_SIZE, offset, filter).await?;
                let new_next = page
                    .next
                    .is_some()
                    .then_some(offset + page.results.len() as u32);
                let mut buf: VecDeque<CircuitGroup> = page.results.into_iter().collect();
                match buf.pop_front() {
                    Some(item) => Ok(Some((item, (new_next, buf)))),
                    None => Ok(None),
                }
            },
        ))
    }

    /// Returns a single circuit group by ID.
    ///
    /// Maps to `GET /api/circuits/circuit-groups/{id}/`
    /// (`operationId`: `circuits_circuit_groups_retrieve`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 404 if not found.
    pub async fn circuit_group(&self, id: i64) -> crate::Result<CircuitGroup> {
        let url = format!("{}/api/circuits/circuit-groups/{id}/", self.base_url);
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .send_json::<CircuitGroup>()
            .await
    }

    /// Creates a new circuit group.
    ///
    /// Maps to `POST /api/circuits/circuit-groups/`
    /// (`operationId`: `circuits_circuit_groups_create`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn circuit_group_create(
        &self,
        body: &CircuitGroupRequest,
    ) -> crate::Result<CircuitGroup> {
        let url = format!("{}/api/circuits/circuit-groups/", self.base_url);
        self.http
            .post(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<CircuitGroup>()
            .await
    }

    /// Replaces a circuit group (full update).
    ///
    /// Maps to `PUT /api/circuits/circuit-groups/{id}/`
    /// (`operationId`: `circuits_circuit_groups_update`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn circuit_group_update(
        &self,
        id: i64,
        body: &CircuitGroupRequest,
    ) -> crate::Result<CircuitGroup> {
        let url = format!("{}/api/circuits/circuit-groups/{id}/", self.base_url);
        self.http
            .put(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<CircuitGroup>()
            .await
    }

    /// Partially updates a circuit group.
    ///
    /// Maps to `PATCH /api/circuits/circuit-groups/{id}/`
    /// (`operationId`: `circuits_circuit_groups_partial_update`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn circuit_group_patch(
        &self,
        id: i64,
        body: &CircuitGroupPatchRequest,
    ) -> crate::Result<CircuitGroup> {
        let url = format!("{}/api/circuits/circuit-groups/{id}/", self.base_url);
        self.http
            .patch(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<CircuitGroup>()
            .await
    }

    /// Deletes a circuit group.
    ///
    /// Maps to `DELETE /api/circuits/circuit-groups/{id}/`
    /// (`operationId`: `circuits_circuit_groups_destroy`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 404 if not found.
    pub async fn circuit_group_delete(&self, id: i64) -> crate::Result<()> {
        let url = format!("{}/api/circuits/circuit-groups/{id}/", self.base_url);
        crate::delete_no_content(&self.http, &url, &self.token).await
    }

    // ── Circuit Group Assignments ─────────────────────────────────────────────

    /// Returns a single page of circuit group assignments.
    ///
    /// Maps to `GET /api/circuits/circuit-group-assignments/`
    /// (`operationId`: `circuits_circuit_group_assignments_list`).
    ///
    /// # Errors
    ///
    /// Returns an error on HTTP or deserialization failure.
    pub async fn circuit_group_assignments_list(
        &self,
        limit: u32,
        offset: u32,
        filter: &CircuitGroupAssignmentFilter,
    ) -> crate::Result<Paginated<CircuitGroupAssignment>> {
        let url = format!("{}/api/circuits/circuit-group-assignments/", self.base_url);
        let mut query = filter.as_query();
        query.push(("limit".into(), limit.to_string()));
        query.push(("offset".into(), offset.to_string()));
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .query(&query)
            .send_json::<Paginated<CircuitGroupAssignment>>()
            .await
    }

    /// Streams all circuit group assignments matching `filter`, auto-paginating.
    #[must_use]
    pub fn circuit_group_assignments<'a>(
        &'a self,
        filter: &'a CircuitGroupAssignmentFilter,
    ) -> BoxStream<'a, crate::Result<CircuitGroupAssignment>> {
        Box::pin(try_unfold(
            (Some(0u32), VecDeque::<CircuitGroupAssignment>::new()),
            move |(next_offset, mut buf)| async move {
                if let Some(item) = buf.pop_front() {
                    return Ok(Some((item, (next_offset, buf))));
                }
                let Some(offset) = next_offset else {
                    return Ok(None);
                };
                let page = self
                    .circuit_group_assignments_list(PAGE_SIZE, offset, filter)
                    .await?;
                let new_next = page
                    .next
                    .is_some()
                    .then_some(offset + page.results.len() as u32);
                let mut buf: VecDeque<CircuitGroupAssignment> = page.results.into_iter().collect();
                match buf.pop_front() {
                    Some(item) => Ok(Some((item, (new_next, buf)))),
                    None => Ok(None),
                }
            },
        ))
    }

    /// Returns a single circuit group assignment by ID.
    ///
    /// Maps to `GET /api/circuits/circuit-group-assignments/{id}/`
    /// (`operationId`: `circuits_circuit_group_assignments_retrieve`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 404 if not found.
    pub async fn circuit_group_assignment(&self, id: i64) -> crate::Result<CircuitGroupAssignment> {
        let url = format!(
            "{}/api/circuits/circuit-group-assignments/{id}/",
            self.base_url
        );
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .send_json::<CircuitGroupAssignment>()
            .await
    }

    /// Creates a new circuit group assignment.
    ///
    /// Maps to `POST /api/circuits/circuit-group-assignments/`
    /// (`operationId`: `circuits_circuit_group_assignments_create`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn circuit_group_assignment_create(
        &self,
        body: &CircuitGroupAssignmentRequest,
    ) -> crate::Result<CircuitGroupAssignment> {
        let url = format!("{}/api/circuits/circuit-group-assignments/", self.base_url);
        self.http
            .post(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<CircuitGroupAssignment>()
            .await
    }

    /// Replaces a circuit group assignment (full update).
    ///
    /// Maps to `PUT /api/circuits/circuit-group-assignments/{id}/`
    /// (`operationId`: `circuits_circuit_group_assignments_update`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn circuit_group_assignment_update(
        &self,
        id: i64,
        body: &CircuitGroupAssignmentRequest,
    ) -> crate::Result<CircuitGroupAssignment> {
        let url = format!(
            "{}/api/circuits/circuit-group-assignments/{id}/",
            self.base_url
        );
        self.http
            .put(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<CircuitGroupAssignment>()
            .await
    }

    /// Partially updates a circuit group assignment.
    ///
    /// Maps to `PATCH /api/circuits/circuit-group-assignments/{id}/`
    /// (`operationId`: `circuits_circuit_group_assignments_partial_update`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn circuit_group_assignment_patch(
        &self,
        id: i64,
        body: &CircuitGroupAssignmentPatchRequest,
    ) -> crate::Result<CircuitGroupAssignment> {
        let url = format!(
            "{}/api/circuits/circuit-group-assignments/{id}/",
            self.base_url
        );
        self.http
            .patch(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<CircuitGroupAssignment>()
            .await
    }

    /// Deletes a circuit group assignment.
    ///
    /// Maps to `DELETE /api/circuits/circuit-group-assignments/{id}/`
    /// (`operationId`: `circuits_circuit_group_assignments_destroy`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 404 if not found.
    pub async fn circuit_group_assignment_delete(&self, id: i64) -> crate::Result<()> {
        let url = format!(
            "{}/api/circuits/circuit-group-assignments/{id}/",
            self.base_url
        );
        crate::delete_no_content(&self.http, &url, &self.token).await
    }

    // ── Virtual Circuit Types ─────────────────────────────────────────────────

    /// Returns a single page of virtual circuit types.
    ///
    /// Maps to `GET /api/circuits/virtual-circuit-types/`
    /// (`operationId`: `circuits_virtual_circuit_types_list`).
    ///
    /// # Errors
    ///
    /// Returns an error on HTTP or deserialization failure.
    pub async fn virtual_circuit_types_list(
        &self,
        limit: u32,
        offset: u32,
        filter: &VirtualCircuitTypeFilter,
    ) -> crate::Result<Paginated<VirtualCircuitType>> {
        let url = format!("{}/api/circuits/virtual-circuit-types/", self.base_url);
        let mut query = filter.as_query();
        query.push(("limit".into(), limit.to_string()));
        query.push(("offset".into(), offset.to_string()));
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .query(&query)
            .send_json::<Paginated<VirtualCircuitType>>()
            .await
    }

    /// Streams all virtual circuit types matching `filter`, auto-paginating.
    #[must_use]
    pub fn virtual_circuit_types<'a>(
        &'a self,
        filter: &'a VirtualCircuitTypeFilter,
    ) -> BoxStream<'a, crate::Result<VirtualCircuitType>> {
        Box::pin(try_unfold(
            (Some(0u32), VecDeque::<VirtualCircuitType>::new()),
            move |(next_offset, mut buf)| async move {
                if let Some(item) = buf.pop_front() {
                    return Ok(Some((item, (next_offset, buf))));
                }
                let Some(offset) = next_offset else {
                    return Ok(None);
                };
                let page = self
                    .virtual_circuit_types_list(PAGE_SIZE, offset, filter)
                    .await?;
                let new_next = page
                    .next
                    .is_some()
                    .then_some(offset + page.results.len() as u32);
                let mut buf: VecDeque<VirtualCircuitType> = page.results.into_iter().collect();
                match buf.pop_front() {
                    Some(item) => Ok(Some((item, (new_next, buf)))),
                    None => Ok(None),
                }
            },
        ))
    }

    /// Returns a single virtual circuit type by ID.
    ///
    /// Maps to `GET /api/circuits/virtual-circuit-types/{id}/`
    /// (`operationId`: `circuits_virtual_circuit_types_retrieve`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 404 if not found.
    pub async fn virtual_circuit_type(&self, id: i64) -> crate::Result<VirtualCircuitType> {
        let url = format!("{}/api/circuits/virtual-circuit-types/{id}/", self.base_url);
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .send_json::<VirtualCircuitType>()
            .await
    }

    /// Creates a new virtual circuit type.
    ///
    /// Maps to `POST /api/circuits/virtual-circuit-types/`
    /// (`operationId`: `circuits_virtual_circuit_types_create`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn virtual_circuit_type_create(
        &self,
        body: &VirtualCircuitTypeRequest,
    ) -> crate::Result<VirtualCircuitType> {
        let url = format!("{}/api/circuits/virtual-circuit-types/", self.base_url);
        self.http
            .post(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<VirtualCircuitType>()
            .await
    }

    /// Replaces a virtual circuit type (full update).
    ///
    /// Maps to `PUT /api/circuits/virtual-circuit-types/{id}/`
    /// (`operationId`: `circuits_virtual_circuit_types_update`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn virtual_circuit_type_update(
        &self,
        id: i64,
        body: &VirtualCircuitTypeRequest,
    ) -> crate::Result<VirtualCircuitType> {
        let url = format!("{}/api/circuits/virtual-circuit-types/{id}/", self.base_url);
        self.http
            .put(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<VirtualCircuitType>()
            .await
    }

    /// Partially updates a virtual circuit type.
    ///
    /// Maps to `PATCH /api/circuits/virtual-circuit-types/{id}/`
    /// (`operationId`: `circuits_virtual_circuit_types_partial_update`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn virtual_circuit_type_patch(
        &self,
        id: i64,
        body: &VirtualCircuitTypePatchRequest,
    ) -> crate::Result<VirtualCircuitType> {
        let url = format!("{}/api/circuits/virtual-circuit-types/{id}/", self.base_url);
        self.http
            .patch(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<VirtualCircuitType>()
            .await
    }

    /// Deletes a virtual circuit type.
    ///
    /// Maps to `DELETE /api/circuits/virtual-circuit-types/{id}/`
    /// (`operationId`: `circuits_virtual_circuit_types_destroy`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 404 if not found.
    pub async fn virtual_circuit_type_delete(&self, id: i64) -> crate::Result<()> {
        let url = format!("{}/api/circuits/virtual-circuit-types/{id}/", self.base_url);
        crate::delete_no_content(&self.http, &url, &self.token).await
    }

    // ── Virtual Circuits ──────────────────────────────────────────────────────

    /// Returns a single page of virtual circuits.
    ///
    /// Maps to `GET /api/circuits/virtual-circuits/`
    /// (`operationId`: `circuits_virtual_circuits_list`).
    ///
    /// # Errors
    ///
    /// Returns an error on HTTP or deserialization failure.
    pub async fn virtual_circuits_list(
        &self,
        limit: u32,
        offset: u32,
        filter: &VirtualCircuitFilter,
    ) -> crate::Result<Paginated<VirtualCircuit>> {
        let url = format!("{}/api/circuits/virtual-circuits/", self.base_url);
        let mut query = filter.as_query();
        query.push(("limit".into(), limit.to_string()));
        query.push(("offset".into(), offset.to_string()));
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .query(&query)
            .send_json::<Paginated<VirtualCircuit>>()
            .await
    }

    /// Streams all virtual circuits matching `filter`, auto-paginating.
    #[must_use]
    pub fn virtual_circuits<'a>(
        &'a self,
        filter: &'a VirtualCircuitFilter,
    ) -> BoxStream<'a, crate::Result<VirtualCircuit>> {
        Box::pin(try_unfold(
            (Some(0u32), VecDeque::<VirtualCircuit>::new()),
            move |(next_offset, mut buf)| async move {
                if let Some(item) = buf.pop_front() {
                    return Ok(Some((item, (next_offset, buf))));
                }
                let Some(offset) = next_offset else {
                    return Ok(None);
                };
                let page = self
                    .virtual_circuits_list(PAGE_SIZE, offset, filter)
                    .await?;
                let new_next = page
                    .next
                    .is_some()
                    .then_some(offset + page.results.len() as u32);
                let mut buf: VecDeque<VirtualCircuit> = page.results.into_iter().collect();
                match buf.pop_front() {
                    Some(item) => Ok(Some((item, (new_next, buf)))),
                    None => Ok(None),
                }
            },
        ))
    }

    /// Returns a single virtual circuit by ID.
    ///
    /// Maps to `GET /api/circuits/virtual-circuits/{id}/`
    /// (`operationId`: `circuits_virtual_circuits_retrieve`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 404 if not found.
    pub async fn virtual_circuit(&self, id: i64) -> crate::Result<VirtualCircuit> {
        let url = format!("{}/api/circuits/virtual-circuits/{id}/", self.base_url);
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .send_json::<VirtualCircuit>()
            .await
    }

    /// Creates a new virtual circuit.
    ///
    /// Maps to `POST /api/circuits/virtual-circuits/`
    /// (`operationId`: `circuits_virtual_circuits_create`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn virtual_circuit_create(
        &self,
        body: &VirtualCircuitRequest,
    ) -> crate::Result<VirtualCircuit> {
        let url = format!("{}/api/circuits/virtual-circuits/", self.base_url);
        self.http
            .post(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<VirtualCircuit>()
            .await
    }

    /// Replaces a virtual circuit (full update).
    ///
    /// Maps to `PUT /api/circuits/virtual-circuits/{id}/`
    /// (`operationId`: `circuits_virtual_circuits_update`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn virtual_circuit_update(
        &self,
        id: i64,
        body: &VirtualCircuitRequest,
    ) -> crate::Result<VirtualCircuit> {
        let url = format!("{}/api/circuits/virtual-circuits/{id}/", self.base_url);
        self.http
            .put(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<VirtualCircuit>()
            .await
    }

    /// Partially updates a virtual circuit.
    ///
    /// Maps to `PATCH /api/circuits/virtual-circuits/{id}/`
    /// (`operationId`: `circuits_virtual_circuits_partial_update`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn virtual_circuit_patch(
        &self,
        id: i64,
        body: &VirtualCircuitPatchRequest,
    ) -> crate::Result<VirtualCircuit> {
        let url = format!("{}/api/circuits/virtual-circuits/{id}/", self.base_url);
        self.http
            .patch(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<VirtualCircuit>()
            .await
    }

    /// Deletes a virtual circuit.
    ///
    /// Maps to `DELETE /api/circuits/virtual-circuits/{id}/`
    /// (`operationId`: `circuits_virtual_circuits_destroy`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 404 if not found.
    pub async fn virtual_circuit_delete(&self, id: i64) -> crate::Result<()> {
        let url = format!("{}/api/circuits/virtual-circuits/{id}/", self.base_url);
        crate::delete_no_content(&self.http, &url, &self.token).await
    }

    // ── Virtual Circuit Terminations ──────────────────────────────────────────

    /// Returns a single page of virtual circuit terminations.
    ///
    /// Maps to `GET /api/circuits/virtual-circuit-terminations/`
    /// (`operationId`: `circuits_virtual_circuit_terminations_list`).
    ///
    /// # Errors
    ///
    /// Returns an error on HTTP or deserialization failure.
    pub async fn virtual_circuit_terminations_list(
        &self,
        limit: u32,
        offset: u32,
        filter: &VirtualCircuitTerminationFilter,
    ) -> crate::Result<Paginated<VirtualCircuitTermination>> {
        let url = format!(
            "{}/api/circuits/virtual-circuit-terminations/",
            self.base_url
        );
        let mut query = filter.as_query();
        query.push(("limit".into(), limit.to_string()));
        query.push(("offset".into(), offset.to_string()));
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .query(&query)
            .send_json::<Paginated<VirtualCircuitTermination>>()
            .await
    }

    /// Streams all virtual circuit terminations matching `filter`, auto-paginating.
    #[must_use]
    pub fn virtual_circuit_terminations<'a>(
        &'a self,
        filter: &'a VirtualCircuitTerminationFilter,
    ) -> BoxStream<'a, crate::Result<VirtualCircuitTermination>> {
        Box::pin(try_unfold(
            (Some(0u32), VecDeque::<VirtualCircuitTermination>::new()),
            move |(next_offset, mut buf)| async move {
                if let Some(item) = buf.pop_front() {
                    return Ok(Some((item, (next_offset, buf))));
                }
                let Some(offset) = next_offset else {
                    return Ok(None);
                };
                let page = self
                    .virtual_circuit_terminations_list(PAGE_SIZE, offset, filter)
                    .await?;
                let new_next = page
                    .next
                    .is_some()
                    .then_some(offset + page.results.len() as u32);
                let mut buf: VecDeque<VirtualCircuitTermination> =
                    page.results.into_iter().collect();
                match buf.pop_front() {
                    Some(item) => Ok(Some((item, (new_next, buf)))),
                    None => Ok(None),
                }
            },
        ))
    }

    /// Returns a single virtual circuit termination by ID.
    ///
    /// Maps to `GET /api/circuits/virtual-circuit-terminations/{id}/`
    /// (`operationId`: `circuits_virtual_circuit_terminations_retrieve`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 404 if not found.
    pub async fn virtual_circuit_termination(
        &self,
        id: i64,
    ) -> crate::Result<VirtualCircuitTermination> {
        let url = format!(
            "{}/api/circuits/virtual-circuit-terminations/{id}/",
            self.base_url
        );
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .send_json::<VirtualCircuitTermination>()
            .await
    }

    /// Creates a new virtual circuit termination.
    ///
    /// Maps to `POST /api/circuits/virtual-circuit-terminations/`
    /// (`operationId`: `circuits_virtual_circuit_terminations_create`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn virtual_circuit_termination_create(
        &self,
        body: &VirtualCircuitTerminationRequest,
    ) -> crate::Result<VirtualCircuitTermination> {
        let url = format!(
            "{}/api/circuits/virtual-circuit-terminations/",
            self.base_url
        );
        self.http
            .post(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<VirtualCircuitTermination>()
            .await
    }

    /// Replaces a virtual circuit termination (full update).
    ///
    /// Maps to `PUT /api/circuits/virtual-circuit-terminations/{id}/`
    /// (`operationId`: `circuits_virtual_circuit_terminations_update`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn virtual_circuit_termination_update(
        &self,
        id: i64,
        body: &VirtualCircuitTerminationRequest,
    ) -> crate::Result<VirtualCircuitTermination> {
        let url = format!(
            "{}/api/circuits/virtual-circuit-terminations/{id}/",
            self.base_url
        );
        self.http
            .put(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<VirtualCircuitTermination>()
            .await
    }

    /// Partially updates a virtual circuit termination.
    ///
    /// Maps to `PATCH /api/circuits/virtual-circuit-terminations/{id}/`
    /// (`operationId`: `circuits_virtual_circuit_terminations_partial_update`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn virtual_circuit_termination_patch(
        &self,
        id: i64,
        body: &VirtualCircuitTerminationPatchRequest,
    ) -> crate::Result<VirtualCircuitTermination> {
        let url = format!(
            "{}/api/circuits/virtual-circuit-terminations/{id}/",
            self.base_url
        );
        self.http
            .patch(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<VirtualCircuitTermination>()
            .await
    }

    /// Deletes a virtual circuit termination.
    ///
    /// Maps to `DELETE /api/circuits/virtual-circuit-terminations/{id}/`
    /// (`operationId`: `circuits_virtual_circuit_terminations_destroy`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 404 if not found.
    pub async fn virtual_circuit_termination_delete(&self, id: i64) -> crate::Result<()> {
        let url = format!(
            "{}/api/circuits/virtual-circuit-terminations/{id}/",
            self.base_url
        );
        crate::delete_no_content(&self.http, &url, &self.token).await
    }

    /// Returns the cable path(s) for a virtual circuit termination.
    ///
    /// Maps to `GET /api/circuits/virtual-circuit-terminations/{id}/paths/`
    /// (`operationId`: `circuits_virtual_circuit_terminations_paths_retrieve`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 404 if not found.
    pub async fn virtual_circuit_termination_paths(&self, id: i64) -> crate::Result<JsonValue> {
        let url = format!(
            "{}/api/circuits/virtual-circuit-terminations/{id}/paths/",
            self.base_url
        );
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .send_json::<JsonValue>()
            .await
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use futures_util::TryStreamExt as _;
    use wiremock::{
        matchers::{header, method, path, query_param},
        Mock, MockServer, ResponseTemplate,
    };

    use super::*;

    fn provider_json(id: i64) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "url": format!("https://nb.example.com/api/circuits/providers/{id}/"),
            "display_url": format!("https://nb.example.com/circuits/providers/{id}/"),
            "display": format!("Provider {id}"),
            "name": format!("Provider {id}"),
            "slug": format!("provider-{id}"),
            "description": "",
            "accounts": [],
            "comments": "",
            "asns": [],
            "tags": [],
            "custom_fields": {},
            "created": "2024-01-01T00:00:00Z",
            "last_updated": "2024-01-01T00:00:00Z"
        })
    }

    fn circuit_json(id: i64) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "url": format!("https://nb.example.com/api/circuits/circuits/{id}/"),
            "display_url": format!("https://nb.example.com/circuits/circuits/{id}/"),
            "display": format!("CID-{id:04}"),
            "cid": format!("CID-{id:04}"),
            "provider": {
                "id": 1,
                "url": "https://nb.example.com/api/circuits/providers/1/",
                "display": "Provider 1",
                "name": "Provider 1",
                "slug": "provider-1"
            },
            "type": {
                "id": 1,
                "url": "https://nb.example.com/api/circuits/circuit-types/1/",
                "display": "Internet",
                "name": "Internet",
                "slug": "internet"
            },
            "status": { "value": "active", "label": "Active" },
            "description": "",
            "comments": "",
            "tags": [],
            "assignments": [],
            "created": "2024-01-01T00:00:00Z",
            "last_updated": "2024-01-01T00:00:00Z",
            "termination_a": null,
            "termination_z": null
        })
    }

    // ── Provider tests ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn providers_list_returns_page() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/circuits/providers/"))
            .and(header("Authorization", "Token secret"))
            .and(query_param("limit", "50"))
            .and(query_param("offset", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 1,
                "next": null,
                "previous": null,
                "results": [provider_json(1)]
            })))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let page = client
            .providers_list(50, 0, &ProviderFilter::default())
            .await
            .unwrap();
        assert_eq!(page.count, 1);
        assert_eq!(page.results[0].id, 1);
        assert_eq!(page.results[0].name, "Provider 1");
    }

    #[tokio::test]
    async fn providers_stream_walks_two_pages() {
        let server = MockServer::start().await;

        let page1_results: Vec<_> = (1..=50).map(provider_json).collect();
        let page2_results: Vec<_> = (51..=60).map(provider_json).collect();

        Mock::given(method("GET"))
            .and(path("/api/circuits/providers/"))
            .and(query_param("offset", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 60,
                "next": "https://nb.example.com/api/circuits/providers/?limit=50&offset=50",
                "previous": null,
                "results": page1_results
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/api/circuits/providers/"))
            .and(query_param("offset", "50"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 60,
                "next": null,
                "previous": "https://nb.example.com/api/circuits/providers/?limit=50&offset=0",
                "results": page2_results
            })))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let all: Vec<Provider> = client
            .providers(&ProviderFilter::default())
            .try_collect()
            .await
            .unwrap();
        assert_eq!(all.len(), 60);
        assert_eq!(all[0].id, 1);
        assert_eq!(all[59].id, 60);
    }

    #[tokio::test]
    async fn provider_retrieve_returns_provider() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/circuits/providers/42/"))
            .and(header("Authorization", "Token secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(provider_json(42)))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let p = client.provider(42).await.unwrap();
        assert_eq!(p.id, 42);
    }

    #[tokio::test]
    async fn provider_retrieve_returns_404_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/circuits/providers/99/"))
            .respond_with(
                ResponseTemplate::new(404)
                    .set_body_json(serde_json::json!({"detail": "Not found."})),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let err = client.provider(99).await.unwrap_err();
        match err {
            crate::Error::Api { status, body } => {
                assert_eq!(status, reqwest::StatusCode::NOT_FOUND);
                assert_eq!(body.detail.as_deref(), Some("Not found."));
            }
            other => panic!("expected Error::Api, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn provider_create_sends_body_and_returns_provider() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/circuits/providers/"))
            .and(header("Authorization", "Token secret"))
            .and(header("Content-Type", "application/json"))
            .respond_with(ResponseTemplate::new(201).set_body_json(provider_json(10)))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let body = ProviderRequest {
            name: "Acme".into(),
            slug: "acme".into(),
            accounts: vec![],
            description: None,
            owner: None,
            comments: None,
            asns: vec![],
            tags: vec![],
            custom_fields: None,
        };
        let p = client.provider_create(&body).await.unwrap();
        assert_eq!(p.id, 10);
    }

    #[tokio::test]
    async fn provider_delete_returns_ok_on_204() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/api/circuits/providers/5/"))
            .and(header("Authorization", "Token secret"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        client.provider_delete(5).await.unwrap();
    }

    #[tokio::test]
    async fn provider_delete_returns_error_on_404() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/api/circuits/providers/999/"))
            .respond_with(
                ResponseTemplate::new(404)
                    .set_body_json(serde_json::json!({"detail": "Not found."})),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let err = client.provider_delete(999).await.unwrap_err();
        match err {
            crate::Error::Api { status, .. } => {
                assert_eq!(status, reqwest::StatusCode::NOT_FOUND);
            }
            other => panic!("expected Error::Api, got {other:?}"),
        }
    }

    // ── Circuit tests ─────────────────────────────────────────────────────────

    #[tokio::test]
    async fn circuits_list_returns_page() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/circuits/circuits/"))
            .and(header("Authorization", "Token secret"))
            .and(query_param("limit", "50"))
            .and(query_param("offset", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 1,
                "next": null,
                "previous": null,
                "results": [circuit_json(1)]
            })))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let page = client
            .circuits_list(50, 0, &CircuitFilter::default())
            .await
            .unwrap();
        assert_eq!(page.count, 1);
        assert_eq!(page.results[0].cid, "CID-0001");
    }

    #[tokio::test]
    async fn circuits_stream_walks_two_pages() {
        let server = MockServer::start().await;

        let page1: Vec<_> = (1..=50).map(circuit_json).collect();
        let page2: Vec<_> = (51..=55).map(circuit_json).collect();

        Mock::given(method("GET"))
            .and(path("/api/circuits/circuits/"))
            .and(query_param("offset", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 55,
                "next": "https://nb.example.com/api/circuits/circuits/?limit=50&offset=50",
                "previous": null,
                "results": page1
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/api/circuits/circuits/"))
            .and(query_param("offset", "50"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 55,
                "next": null,
                "previous": null,
                "results": page2
            })))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let all: Vec<Circuit> = client
            .circuits(&CircuitFilter::default())
            .try_collect()
            .await
            .unwrap();
        assert_eq!(all.len(), 55);
    }

    #[tokio::test]
    async fn circuit_retrieve_returns_circuit() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/circuits/circuits/7/"))
            .and(header("Authorization", "Token secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(circuit_json(7)))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let c = client.circuit(7).await.unwrap();
        assert_eq!(c.id, 7);
        assert_eq!(c.cid, "CID-0007");
    }

    #[tokio::test]
    async fn circuit_retrieve_returns_404_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/circuits/circuits/999/"))
            .respond_with(
                ResponseTemplate::new(404)
                    .set_body_json(serde_json::json!({"detail": "Not found."})),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let err = client.circuit(999).await.unwrap_err();
        match err {
            crate::Error::Api { status, body } => {
                assert_eq!(status, reqwest::StatusCode::NOT_FOUND);
                assert_eq!(body.detail.as_deref(), Some("Not found."));
            }
            other => panic!("expected Error::Api, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn circuit_create_sends_body_and_returns_circuit() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/circuits/circuits/"))
            .and(header("Authorization", "Token secret"))
            .and(header("Content-Type", "application/json"))
            .respond_with(ResponseTemplate::new(201).set_body_json(circuit_json(20)))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let body = CircuitRequest {
            cid: "CID-0020".into(),
            provider: 1,
            r#type: 1,
            provider_account: None,
            status: Some("active".into()),
            tenant: None,
            install_date: None,
            termination_date: None,
            commit_rate: None,
            description: None,
            distance: None,
            distance_unit: None,
            owner: None,
            comments: None,
            tags: vec![],
            custom_fields: None,
        };
        let c = client.circuit_create(&body).await.unwrap();
        assert_eq!(c.id, 20);
    }

    #[tokio::test]
    async fn circuit_patch_sends_partial_body() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/api/circuits/circuits/3/"))
            .and(header("Authorization", "Token secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(circuit_json(3)))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let patch = CircuitPatchRequest {
            status: Some("planned".into()),
            ..Default::default()
        };
        let c = client.circuit_patch(3, &patch).await.unwrap();
        assert_eq!(c.id, 3);
    }

    #[tokio::test]
    async fn circuit_delete_ok_on_204() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/api/circuits/circuits/8/"))
            .and(header("Authorization", "Token secret"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        client.circuit_delete(8).await.unwrap();
    }

    // ── Circuit type tests ────────────────────────────────────────────────────

    #[tokio::test]
    async fn circuit_types_list_returns_page() {
        let server = MockServer::start().await;
        let ct = serde_json::json!({
            "id": 1, "url": "u", "display_url": "u", "display": "Internet",
            "name": "Internet", "slug": "internet", "color": "", "description": "",
            "comments": "", "tags": [], "created": null, "last_updated": null
        });
        Mock::given(method("GET"))
            .and(path("/api/circuits/circuit-types/"))
            .and(header("Authorization", "Token secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 1, "next": null, "previous": null, "results": [ct]
            })))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let page = client
            .circuit_types_list(50, 0, &CircuitTypeFilter::default())
            .await
            .unwrap();
        assert_eq!(page.results[0].name, "Internet");
    }
}
