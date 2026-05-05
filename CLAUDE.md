# netbox-client

A Rust client library for the Netbox REST API.

## Project Overview

- **API spec**: `schema.json` (OpenAPI 3.0.3 — NetBox REST API 4.5.1 (4.5))
- **Scope**: Only a subset of endpoints from the spec will be implemented; add methods as needed rather than generating the full spec
- **HTTP client**: `reqwest` 0.13 with async/await throughout. The crate is runtime-agnostic — it does **not** depend on `tokio` directly; consumers pick the runtime
- **Base URL**: the spec's `servers` entry is empty. The consumer supplies the NetBox base URL (e.g. `https://netbox.example.com`); the client appends `/api/...` paths
- **Inspiration**: the overall shape of this crate (JSON-backend shim, `RequestBuilderExt`, `ClientBuilder`, auto-paginating `BoxStream` helpers) follows [`omada-client`](https://github.com/ogital-net/omada-client). Mirror its patterns when in doubt.

## Development Commands

```sh
cargo build
cargo test
cargo clippy --all-targets -- -W clippy::pedantic
```

Run `cargo clippy --all-targets -- -W clippy::pedantic` after each change. Clippy warnings are not gospel — suppress them with `#[allow(...)]` when they conflict with readability or the Rust API Guidelines. Prefer idiomatic, readable code over mechanical lint compliance.

## Build Checklist

Use this when adding a new endpoint or feature.

1. **Locate the operation in `schema.json`** by `operationId` and confirm path, method, query params, request body schema, and response schema.
2. **Define request/response models** in `src/models.rs` (only the fields needed by the chosen endpoints). Re-export from the crate root.
   - `#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]`
   - Match JSON field casing exactly (NetBox uses `snake_case` natively — **no** `rename_all`).
   - `Option<T>` for fields the spec marks `nullable: true` or omits from `required`.
   - `///` doc comments lifted from the spec's `description`.
3. **Add a method to `NetboxClient`** in the appropriate submodule under `src/` (group by spec tag, e.g. `dcim`, `ipam`, `circuits`).
   - Async, returns `Result<T, crate::Error>`.
   - Build requests with `reqwest::Client` and send them via the internal `RequestBuilderExt::body_json` / `send_json` helpers — never call `.json()` / `.json::<T>()` directly. The helpers route through the active JSON backend and add DEBUG logging.
   - The `Authorization: Token <token>` header is attached centrally; method bodies should not re-add it.
   - Use `.query(&[...])` / a typed filter builder for query params; never hand-format query strings.
   - For paginated list endpoints, **always provide both**: a `*_page` method returning `Paginated<T>` for callers that want manual control, and a streaming method returning `BoxStream<'_, Result<T>>` that walks `next` automatically. See the Pagination section.
4. **Handle errors**: map non-2xx into `Error::Api { status, body }`; deserialize NetBox's error body when present (`{"detail": "..."}` or field-keyed validation errors).
5. **Write a `wiremock` test** in a `#[cfg(test)] mod tests` block in the same file. Assert method, path, query params, `Authorization` header, and (for writes) the JSON body against the spec. Cover both success and an error response.
6. **Run** `cargo build`, `cargo test`, `cargo clippy --all-targets -- -W clippy::pedantic`. Fix or `#[allow]` with justification.
7. **Update docs**: a crate-level `//!` example for each new top-level capability; method-level `///` examples for non-obvious calls.

## Architecture

- All public API methods live on a central `NetboxClient` struct in `src/lib.rs`, with implementations grouped by spec tag in submodules under `src/` (e.g. `src/dcim.rs`, `src/ipam.rs`).
- The client holds the base URL, an authenticated `reqwest::Client`, and the API token.
- Public request/response types are defined in `src/models.rs` and re-exported at the crate root via `pub use models::*`.
- Internal-only types (auth headers, pagination cursors) stay in `src/lib.rs` or the submodule that uses them.
- A `ClientBuilder` exposes advanced HTTP configuration (TLS verification, custom timeouts, proxies). `NetboxClient::new` is sugar over `NetboxClient::builder().build(...)`.
- A private `mod json` shim re-exports `from_slice` / `to_vec` / `Error` / `Value` from whichever JSON backend feature is enabled. All crate code routes JSON encode/decode through this shim — never `use serde_json::*` directly outside `#[cfg(test)]`.
- A private `RequestBuilderExt` trait adds `.body_json(&body)?` and `.send_json::<T>().await?` to `reqwest::RequestBuilder`. These call through the JSON shim and emit DEBUG logs (request line + headers + body, response status + headers + body) via the `log` crate when `log::Level::Debug` is enabled. Hot path is identical to plain `reqwest` when DEBUG is off.
- Public re-exports `JsonValue` and `JsonError` expose the active backend's `Value` / `Error` types so callers can interoperate with raw JSON without picking a backend themselves.

## Authentication

NetBox uses a single API-key scheme (per `components.securitySchemes.tokenAuth`):

- Header: `Authorization: Token <token>` (v1) — supported everywhere.
- Header: `Authorization: Bearer <key>.<token>` (v2) — newer split-key form.

The `cookieAuth` (`sessionid` cookie) scheme exists for browser sessions and is **not** implemented by this crate.

Construct the client with the token once; do not require callers to pass it on every method:

```rust
let client = NetboxClient::new("https://netbox.example.com", "abc123...")?;
```

## Pagination

List endpoints use `limit` + `offset` query params and return:

```json
{ "count": 123, "next": "https://.../?offset=100&limit=50", "previous": null, "results": [ ... ] }
```

Model the envelope as:

```rust
pub struct Paginated<T> {
    pub count: u64,
    pub next: Option<String>,
    pub previous: Option<String>,
    pub results: Vec<T>,
}
```

**Always expose two methods per list endpoint:**

1. `xs_page(&self, limit: u32, offset: u32, /* filters */) -> Result<Paginated<X>>` — single page, full control.
2. `xs(&self, /* filters */) -> BoxStream<'_, Result<X>>` — auto-paginates, lazily fetching the next page on demand.

The streaming helper uses `futures_util::stream::try_unfold` over `(next_offset: Option<u32>, buf: VecDeque<X>)`. Drain `buf` first, then issue the next request. Stop when `next` is `None` (or, equivalently, when `offset + results.len() >= count`). Match the omada-client `sites()` / `devices()` shape; lift it directly when adding new list endpoints.

A crate-private `const PAGE_SIZE: u32 = 50;` (NetBox's default `limit`) is used by the streaming helpers. Callers that need a different page size go through `xs_page` directly. The stream ends after the first error (i.e. `try_unfold`, not `unfold`).

```rust
// Usage
use futures_util::TryStreamExt as _;
let sites: Vec<Site> = client.sites(&SiteFilter::default()).try_collect().await?;
```

## Filtering

NetBox query params use Django-style lookup suffixes. Support the ones actually needed by callers; do not try to cover every combination. Common suffixes:

| Suffix | Meaning |
|---|---|
| (none) | exact match |
| `__n` | negated (not equal) |
| `__ic` / `__nic` | case-insensitive contains / not-contains |
| `__isw` / `__iew` | case-insensitive starts-/ends-with |
| `__ie` / `__nie` | case-insensitive equals / not-equals |
| `__empty` | is empty / null |
| `__gt` `__gte` `__lt` `__lte` | range comparisons (numeric / date) |

Many filters accept arrays (`?tag=a&tag=b`) and most list endpoints also accept the `id` filter for bulk-by-ID lookup. Many endpoints also accept a `brief=true` query param that returns reduced nested objects — use it when only `id`/`url`/`display` are needed.

## Method Naming

Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/checklist.html):

| Pattern | Convention |
|---|---|
| Getters | `field_name()` — no `get_` prefix (C-GETTER) |
| Conversions to owned | `to_foo()` |
| Conversions borrowing | `as_foo()` |
| Consuming conversions | `into_foo()` |
| Casing | `snake_case` for functions/methods, `CamelCase` for types (C-CASE) |
| Constructors | Associated `new` / `with_*` methods, no bare functions (C-CTOR) |
| Word order | Object–verb–qualifier, e.g. `sites_list`, `device_reboot` (C-WORD-ORDER) |

Map OAS `operationId` snake_case names directly, dropping the redundant `_retrieve` / `_list` suffixes when the return type already disambiguates:

| OAS operationId | Rust method name |
|---|---|
| `dcim_sites_list` | `sites_list()` -> `Paginated<Site>` |
| `dcim_sites_retrieve` | `site(id)` -> `Site` |
| `dcim_sites_create` | `site_create(body)` |
| `dcim_sites_update` | `site_update(id, body)` (PUT) |
| `dcim_sites_partial_update` | `site_patch(id, body)` (PATCH) |
| `dcim_sites_destroy` | `site_delete(id)` |

## Dependencies

This is a library crate — only specify the minimal features actually needed. Do **not** use catch-all feature flags like `tokio/full`. The crate is runtime-agnostic; consumers bring their own async runtime.

All deps currently listed in `Cargo.toml` are intentional, even where not yet used in `src/`:

- `futures-core` / `futures-util` — required by the `BoxStream` auto-pagination helpers.
- `log` — used by `RequestBuilderExt::send_json` for DEBUG-level request/response tracing. No-op when consumers don't install a logger.
- `dotenvy` / `env_logger` (dev) — used by integration tests / examples that hit a real NetBox instance with credentials in `.env`.

```toml
[dependencies]
reqwest = { version = "0.13", features = ["query"] }
serde = { version = "1", features = ["derive"] }
serde_json = { version = "1", optional = true }
sonic-rs = { version = "0", optional = true }
simd-json = { version = "0", optional = true }
thiserror = "2"
futures-core = "0.3"
futures-util = { version = "0.3", default-features = false, features = ["alloc"] }
log = "0"

[features]
default = ["serde_json"]
serde_json = ["dep:serde_json"]
sonic-rs   = ["dep:sonic-rs"]   # mutually exclusive with serde_json / simd-json
simd-json  = ["dep:simd-json"]  # mutually exclusive with serde_json / sonic-rs

[dev-dependencies]
serde_json = "1"
wiremock = "0.6"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
dotenvy = "0.15"
env_logger = "0.11"
```

## JSON Backends

Three mutually-exclusive feature flags select the JSON backend at compile time: `serde_json` (default), `sonic-rs`, `simd-json`. `src/lib.rs` enforces this with `compile_error!`:

```rust
#[cfg(all(feature = "serde_json", feature = "sonic-rs"))]
compile_error!("features `serde_json` and `sonic-rs` are mutually exclusive; enable only one");
#[cfg(all(feature = "serde_json", feature = "simd-json"))]
compile_error!("features `serde_json` and `simd-json` are mutually exclusive; enable only one");
#[cfg(all(feature = "sonic-rs", feature = "simd-json"))]
compile_error!("features `sonic-rs` and `simd-json` are mutually exclusive; enable only one");
#[cfg(not(any(feature = "serde_json", feature = "sonic-rs", feature = "simd-json")))]
compile_error!("at least one of the `serde_json`, `sonic-rs`, or `simd-json` features must be enabled");
```

A private `mod json` shim normalizes the three backends to the same surface (`from_slice`, `to_vec`, `Error`, `Value`). All crate code uses the shim:

```rust
#[cfg(feature = "serde_json")]
mod json { pub use serde_json::{from_slice, to_vec, Error, Value}; }

#[cfg(feature = "sonic-rs")]
mod json { pub use sonic_rs::{from_slice, to_vec, Error, Value}; }

#[cfg(feature = "simd-json")]
mod json {
    pub use simd_json::{to_vec, Error, OwnedValue as Value};
    pub fn from_slice<T>(input: &[u8]) -> Result<T, Error>
    where T: for<'de> serde::Deserialize<'de> {
        let mut bytes = input.to_vec();
        simd_json::serde::from_slice(&mut bytes)
    }
}

pub use json::Value as JsonValue;
pub use json::Error as JsonError;
```

**Never** write `use serde_json::...` (or any backend's symbol) in `src/` outside `#[cfg(test)]`. Always go through `json::...` or the public `JsonValue` / `JsonError` re-exports.

## Error Handling

- Define a crate-level `Error` enum using `thiserror`.
- Variants should at minimum cover:
  - `Http(#[from] reqwest::Error)` — transport / decoding failures
  - `Api { status: reqwest::StatusCode, body: ApiError }` — non-2xx with NetBox error body (`{"detail": "..."}` or `{"<field>": ["msg", ...]}`)
  - `InvalidUrl(...)` — base-URL join failures
  - `Json(#[from] crate::json::Error)` — backend-agnostic JSON encode/decode failures from the shim
- Return `Result<T, crate::Error>` from all public async methods.
- Never `unwrap()`/`expect()` in library code; propagate with `?`.

## Internal HTTP Helper

All request/response JSON goes through a private `RequestBuilderExt` trait on `reqwest::RequestBuilder` (mirrors omada-client):

```rust
trait RequestBuilderExt {
    /// Serialize `body` via the active JSON backend and attach as the request body
    /// with `Content-Type: application/json`.
    fn body_json<B: serde::Serialize>(self, body: &B) -> crate::Result<Self> where Self: Sized;

    /// Send the request, collect the response body bytes, and deserialize via the
    /// active JSON backend. Logs the full request/response at DEBUG when enabled.
    async fn send_json<T>(self) -> crate::Result<T> where T: for<'de> serde::Deserialize<'de>;
}
```

`send_json` is the single chokepoint for non-2xx → `Error::Api` mapping and for DEBUG logging via the `log` crate. Method bodies become uniform:

```rust
self.http
    .get(url)
    .header("Authorization", format!("Token {}", self.token))
    .query(&filter.as_query())
    .send_json::<Paginated<Site>>()
    .await
```

## Testing

- Use [`wiremock`](https://crates.io/crates/wiremock) for all HTTP-level tests. Spin up a `MockServer`, mount `Mock` handlers that assert the exact method, path, query params, headers (`Authorization: Token ...`), and JSON body prescribed by the spec, then assert on response values.
- Tests live in a `#[cfg(test)]` module inside the relevant source file.
- Cover both happy path and at least one error path (e.g. 404 with `{"detail":"Not found."}`, 400 with field-keyed validation body) per endpoint.
- Pagination helpers must have a test that walks at least two pages.

## Code Style

- `#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]` on all request/response structs.
- **Do not** apply `#[serde(rename_all = ...)]` — NetBox JSON is already `snake_case`.
- Use `Option<T>` for fields the spec marks `nullable: true` or omits from `required`.
- Most NetBox objects share a common envelope: `id: i64`, `url: String`, `display: String`, `created: Option<json_ts::JsonTimestamp>`, `last_updated: Option<json_ts::JsonTimestamp>`. Consider a shared trait or struct embed if many models repeat these.
- **Timestamp fields** (`created`, `last_updated`) use `Option<json_ts::JsonTimestamp>` — the `json-ts` crate (git dep with `serde` feature) parses/formats RFC3339/ISO8601 strings automatically via serde. **Date-only fields** (e.g. `install_date`, `termination_date`, format `YYYY-MM-DD`) remain `Option<String>` because `json-ts` only handles full RFC3339 timestamps with a `Z` suffix.
- Avoid `unwrap()`/`expect()` in library code; propagate errors with `?`.
- Every public field on a public type must have a `///` doc comment sourced from the spec's `description`. Lightly edit for Rust doc conventions (backtick literals, wrap at ~100 chars) but preserve the original meaning.
