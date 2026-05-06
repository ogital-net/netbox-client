//! Methods for the `extras` tag of the NetBox REST API.
//!
//! # Example
//!
//! ```no_run
//! # async fn example() -> netbox_client::Result<()> {
//! use futures_util::TryStreamExt as _;
//! use netbox_client::extras::CustomFieldFilter;
//!
//! let client = netbox_client::NetboxClient::new("https://netbox.example.com", "abc123")?;
//!
//! // Stream all custom fields assigned to dcim.site
//! let filter = CustomFieldFilter {
//!     object_type: Some("dcim.site".into()),
//!     ..Default::default()
//! };
//! let fields: Vec<_> = client.custom_fields(&filter).try_collect().await?;
//! println!("Found {} custom fields on dcim.site", fields.len());
//! # Ok(()) }
//! ```

// Page sizes from NetBox are always far smaller than 2^32 items.
#![allow(clippy::cast_possible_truncation)]

use std::collections::VecDeque;

use futures_core::stream::BoxStream;
use futures_util::stream::try_unfold;

use crate::RequestBuilderExt as _;
use crate::{CustomField, CustomFieldPatchRequest, CustomFieldRequest, NetboxClient, Paginated};

const PAGE_SIZE: u32 = 50;

// ── Filter types ─────────────────────────────────────────────────────────────

/// Filters for the [`NetboxClient::custom_fields_list`] / [`NetboxClient::custom_fields`] endpoints.
#[derive(Debug, Clone, Default)]
pub struct CustomFieldFilter {
    /// Free-text search.
    pub q: Option<String>,
    /// Limit results to these IDs.
    pub id: Vec<i64>,
    /// Filter by field name (exact).
    pub name: Vec<String>,
    /// Filter by content type (e.g. `dcim.site`).
    pub object_type: Option<String>,
    /// Filter by field type (e.g. `text`, `integer`, `select`).
    pub r#type: Vec<String>,
}

impl CustomFieldFilter {
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
        if let Some(ot) = &self.object_type {
            p.push(("object_type".into(), ot.clone()));
        }
        for v in &self.r#type {
            p.push(("type".into(), v.clone()));
        }
        p
    }
}

// ── NetboxClient implementation ───────────────────────────────────────────────

impl NetboxClient {
    // ── Custom fields ─────────────────────────────────────────────────────────

    /// Returns a single page of custom fields.
    ///
    /// Maps to `GET /api/extras/custom-fields/`
    /// (`operationId`: `extras_custom_fields_list`).
    ///
    /// # Errors
    ///
    /// Returns an error on HTTP or deserialization failure.
    pub async fn custom_fields_list(
        &self,
        limit: u32,
        offset: u32,
        filter: &CustomFieldFilter,
    ) -> crate::Result<Paginated<CustomField>> {
        let url = format!("{}/api/extras/custom-fields/", self.base_url);
        let mut query = filter.as_query();
        query.push(("limit".into(), limit.to_string()));
        query.push(("offset".into(), offset.to_string()));
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .query(&query)
            .send_json::<Paginated<CustomField>>()
            .await
    }

    /// Streams all custom fields matching `filter`, auto-paginating.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> netbox_client::Result<()> {
    /// use futures_util::TryStreamExt as _;
    /// use netbox_client::extras::CustomFieldFilter;
    /// let client = netbox_client::NetboxClient::new("https://netbox.example.com", "token")?;
    /// let all: Vec<_> = client.custom_fields(&CustomFieldFilter::default()).try_collect().await?;
    /// # Ok(()) }
    /// ```
    #[must_use]
    pub fn custom_fields<'a>(
        &'a self,
        filter: &'a CustomFieldFilter,
    ) -> BoxStream<'a, crate::Result<CustomField>> {
        Box::pin(try_unfold(
            (Some(0u32), VecDeque::<CustomField>::new()),
            move |(next_offset, mut buf)| async move {
                if let Some(item) = buf.pop_front() {
                    return Ok(Some((item, (next_offset, buf))));
                }
                let Some(offset) = next_offset else {
                    return Ok(None);
                };
                let page = self.custom_fields_list(PAGE_SIZE, offset, filter).await?;
                let new_next = page
                    .next
                    .is_some()
                    .then_some(offset + page.results.len() as u32);
                let mut buf: VecDeque<CustomField> = page.results.into_iter().collect();
                match buf.pop_front() {
                    Some(item) => Ok(Some((item, (new_next, buf)))),
                    None => Ok(None),
                }
            },
        ))
    }

    /// Returns a single custom field by ID.
    ///
    /// Maps to `GET /api/extras/custom-fields/{id}/`
    /// (`operationId`: `extras_custom_fields_retrieve`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 404 if the custom field does not exist.
    pub async fn custom_field(&self, id: i64) -> crate::Result<CustomField> {
        let url = format!("{}/api/extras/custom-fields/{id}/", self.base_url);
        self.http
            .get(&url)
            .header("Authorization", format!("Token {}", self.token))
            .send_json::<CustomField>()
            .await
    }

    /// Creates a new custom field.
    ///
    /// Maps to `POST /api/extras/custom-fields/`
    /// (`operationId`: `extras_custom_fields_create`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn custom_field_create(
        &self,
        body: &CustomFieldRequest,
    ) -> crate::Result<CustomField> {
        let url = format!("{}/api/extras/custom-fields/", self.base_url);
        self.http
            .post(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<CustomField>()
            .await
    }

    /// Replaces a custom field (full update).
    ///
    /// Maps to `PUT /api/extras/custom-fields/{id}/`
    /// (`operationId`: `extras_custom_fields_update`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn custom_field_update(
        &self,
        id: i64,
        body: &CustomFieldRequest,
    ) -> crate::Result<CustomField> {
        let url = format!("{}/api/extras/custom-fields/{id}/", self.base_url);
        self.http
            .put(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<CustomField>()
            .await
    }

    /// Partially updates a custom field.
    ///
    /// Maps to `PATCH /api/extras/custom-fields/{id}/`
    /// (`operationId`: `extras_custom_fields_partial_update`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 400 on validation failure.
    pub async fn custom_field_patch(
        &self,
        id: i64,
        body: &CustomFieldPatchRequest,
    ) -> crate::Result<CustomField> {
        let url = format!("{}/api/extras/custom-fields/{id}/", self.base_url);
        self.http
            .patch(&url)
            .header("Authorization", format!("Token {}", self.token))
            .body_json(body)?
            .send_json::<CustomField>()
            .await
    }

    /// Deletes a custom field.
    ///
    /// Maps to `DELETE /api/extras/custom-fields/{id}/`
    /// (`operationId`: `extras_custom_fields_destroy`).
    ///
    /// # Errors
    ///
    /// Returns `Error::Api` with status 404 if the custom field does not exist.
    pub async fn custom_field_delete(&self, id: i64) -> crate::Result<()> {
        let url = format!("{}/api/extras/custom-fields/{id}/", self.base_url);
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
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use futures_util::TryStreamExt as _;
    use wiremock::matchers::{header, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::{CustomField, CustomFieldPatchRequest, CustomFieldRequest, NetboxClient};

    use super::CustomFieldFilter;

    fn custom_field_json(id: i64) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "url": format!("https://nb.example.com/api/extras/custom-fields/{id}/"),
            "display_url": format!("https://nb.example.com/extras/custom-fields/{id}/"),
            "display": format!("cf_field_{id}"),
            "object_types": ["dcim.site"],
            "type": {"value": "text", "label": "Text"},
            "related_object_type": null,
            "data_type": "str",
            "name": format!("cf_field_{id}"),
            "label": format!("Custom Field {id}"),
            "group_name": "",
            "description": "",
            "required": false,
            "unique": false,
            "search_weight": 1000,
            "filter_logic": {"value": "loose", "label": "Loose"},
            "ui_visible": {"value": "always", "label": "Always"},
            "ui_editable": {"value": "yes", "label": "Yes"},
            "is_cloneable": false,
            "default": null,
            "related_object_filter": null,
            "weight": 100,
            "validation_minimum": null,
            "validation_maximum": null,
            "validation_regex": "",
            "choice_set": null,
            "owner": null,
            "comments": "",
            "created": null,
            "last_updated": null
        })
    }

    // ── List / stream ─────────────────────────────────────────────────────────

    #[tokio::test]
    async fn custom_fields_list_returns_page() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/extras/custom-fields/"))
            .and(header("Authorization", "Token secret"))
            .and(query_param("limit", "50"))
            .and(query_param("offset", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 1,
                "next": null,
                "previous": null,
                "results": [custom_field_json(1)]
            })))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let page = client
            .custom_fields_list(50, 0, &CustomFieldFilter::default())
            .await
            .unwrap();
        assert_eq!(page.count, 1);
        assert_eq!(page.results[0].name, "cf_field_1");
    }

    #[tokio::test]
    async fn custom_fields_list_applies_filter() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/extras/custom-fields/"))
            .and(query_param("object_type", "dcim.site"))
            .and(query_param("type", "text"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 1,
                "next": null,
                "previous": null,
                "results": [custom_field_json(2)]
            })))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let filter = CustomFieldFilter {
            object_type: Some("dcim.site".into()),
            r#type: vec!["text".into()],
            ..Default::default()
        };
        let page = client.custom_fields_list(50, 0, &filter).await.unwrap();
        assert_eq!(page.results[0].id, 2);
    }

    #[tokio::test]
    async fn custom_fields_stream_walks_two_pages() {
        let server = MockServer::start().await;

        let page1: Vec<_> = (1..=50).map(custom_field_json).collect();
        let page2: Vec<_> = (51..=55).map(custom_field_json).collect();

        Mock::given(method("GET"))
            .and(path("/api/extras/custom-fields/"))
            .and(query_param("offset", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "count": 55,
                "next": "https://nb.example.com/api/extras/custom-fields/?limit=50&offset=50",
                "previous": null,
                "results": page1
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/api/extras/custom-fields/"))
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
        let all: Vec<CustomField> = client
            .custom_fields(&CustomFieldFilter::default())
            .try_collect()
            .await
            .unwrap();
        assert_eq!(all.len(), 55);
    }

    // ── Retrieve ──────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn custom_field_retrieve_returns_field() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/extras/custom-fields/7/"))
            .and(header("Authorization", "Token secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(custom_field_json(7)))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let cf = client.custom_field(7).await.unwrap();
        assert_eq!(cf.id, 7);
        assert_eq!(cf.name, "cf_field_7");
    }

    #[tokio::test]
    async fn custom_field_retrieve_returns_404_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/extras/custom-fields/999/"))
            .respond_with(
                ResponseTemplate::new(404)
                    .set_body_json(serde_json::json!({"detail": "Not found."})),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let err = client.custom_field(999).await.unwrap_err();
        match err {
            crate::Error::Api { status, body } => {
                assert_eq!(status, reqwest::StatusCode::NOT_FOUND);
                assert_eq!(body.detail.as_deref(), Some("Not found."));
            }
            other => panic!("expected Error::Api, got {other:?}"),
        }
    }

    // ── Create ────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn custom_field_create_sends_body_and_returns_field() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/extras/custom-fields/"))
            .and(header("Authorization", "Token secret"))
            .and(header("Content-Type", "application/json"))
            .respond_with(ResponseTemplate::new(201).set_body_json(custom_field_json(10)))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let body = CustomFieldRequest {
            object_types: vec!["dcim.site".into()],
            r#type: "text".into(),
            name: "cf_field_10".into(),
            ..Default::default()
        };
        let cf = client.custom_field_create(&body).await.unwrap();
        assert_eq!(cf.id, 10);
        assert_eq!(cf.name, "cf_field_10");
    }

    #[tokio::test]
    async fn custom_field_create_returns_400_on_validation_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/extras/custom-fields/"))
            .respond_with(
                ResponseTemplate::new(400)
                    .set_body_json(serde_json::json!({"detail": "Invalid field type."})),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let body = CustomFieldRequest {
            object_types: vec!["dcim.site".into()],
            r#type: "invalid_type".into(),
            name: "bad_field".into(),
            ..Default::default()
        };
        let err = client.custom_field_create(&body).await.unwrap_err();
        match err {
            crate::Error::Api { status, .. } => {
                assert_eq!(status, reqwest::StatusCode::BAD_REQUEST);
            }
            other => panic!("expected Error::Api, got {other:?}"),
        }
    }

    // ── Update (PUT) ──────────────────────────────────────────────────────────

    #[tokio::test]
    async fn custom_field_update_sends_put_and_returns_field() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/api/extras/custom-fields/3/"))
            .and(header("Authorization", "Token secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(custom_field_json(3)))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let body = CustomFieldRequest {
            object_types: vec!["dcim.site".into()],
            r#type: "text".into(),
            name: "cf_field_3".into(),
            description: Some("updated description".into()),
            ..Default::default()
        };
        let cf = client.custom_field_update(3, &body).await.unwrap();
        assert_eq!(cf.id, 3);
    }

    // ── Patch (PATCH) ─────────────────────────────────────────────────────────

    #[tokio::test]
    async fn custom_field_patch_sends_partial_body() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/api/extras/custom-fields/5/"))
            .and(header("Authorization", "Token secret"))
            .respond_with(ResponseTemplate::new(200).set_body_json(custom_field_json(5)))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let patch = CustomFieldPatchRequest {
            description: Some("new description".into()),
            ..Default::default()
        };
        let cf = client.custom_field_patch(5, &patch).await.unwrap();
        assert_eq!(cf.id, 5);
    }

    // ── Delete ────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn custom_field_delete_ok_on_204() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/api/extras/custom-fields/8/"))
            .and(header("Authorization", "Token secret"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        client.custom_field_delete(8).await.unwrap();
    }

    #[tokio::test]
    async fn custom_field_delete_returns_error_on_404() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/api/extras/custom-fields/999/"))
            .respond_with(
                ResponseTemplate::new(404)
                    .set_body_json(serde_json::json!({"detail": "Not found."})),
            )
            .mount(&server)
            .await;

        let client = NetboxClient::new(server.uri(), "secret").unwrap();
        let err = client.custom_field_delete(999).await.unwrap_err();
        match err {
            crate::Error::Api { status, .. } => {
                assert_eq!(status, reqwest::StatusCode::NOT_FOUND);
            }
            other => panic!("expected Error::Api, got {other:?}"),
        }
    }
}
