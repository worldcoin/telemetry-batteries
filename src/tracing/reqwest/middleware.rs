//! Trace-context propagation for `reqwest-middleware` clients.

use crate::tracing::trace_to_headers;

/// Injects the current span's trace context into every outgoing request.
///
/// This middleware only propagates context; it does not create a client span.
///
/// # Example
///
/// ```no_run
/// use telemetry_batteries::tracing::reqwest::middleware::TraceContextMiddleware;
///
/// let client = reqwest_middleware::ClientBuilder::new(reqwest::Client::new())
///     .with(TraceContextMiddleware::new())
///     .build();
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct TraceContextMiddleware;

impl TraceContextMiddleware {
    /// Create trace-context propagation middleware.
    pub const fn new() -> Self {
        Self
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
impl reqwest_middleware::Middleware for TraceContextMiddleware {
    async fn handle(
        &self,
        mut request: ::reqwest::Request,
        extensions: &mut http::Extensions,
        next: reqwest_middleware::Next<'_>,
    ) -> reqwest_middleware::Result<::reqwest::Response> {
        trace_to_headers(request.headers_mut());
        next.run(request, extensions).await
    }
}
