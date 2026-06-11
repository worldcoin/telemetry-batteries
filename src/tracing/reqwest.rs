//! Traced reqwest HTTP client.
//!
//! Wraps a [`reqwest::Client`] with [`reqwest_middleware`] and
//! [`reqwest_tracing::TracingMiddleware`] so that every outgoing request
//! automatically gets a tracing span (with URL, method, status, etc.).
//! When a `tracing-opentelemetry` layer is active these spans are exported
//! as OTel spans and the trace context is propagated to downstream services.
//!
//! # Quick start
//!
//! ```ignore
//! use telemetry_batteries::tracing::reqwest::traced_client;
//!
//! let client = traced_client(reqwest::Client::new());
//! let resp = client.get("https://example.com").send().await?;
//! ```

pub use reqwest_middleware::ClientWithMiddleware;

use reqwest_middleware::ClientBuilder;
use reqwest_tracing::{SpanBackendWithUrl, TracingMiddleware};

/// Wrap an existing [`reqwest::Client`] with tracing middleware.
///
/// Returns a [`ClientWithMiddleware`] that instruments every request with a
/// tracing span containing the HTTP method, URL, and response status.
pub fn traced_client(client: reqwest::Client) -> ClientWithMiddleware {
    ClientBuilder::new(client)
        .with(TracingMiddleware::<SpanBackendWithUrl>::new())
        .build()
}
