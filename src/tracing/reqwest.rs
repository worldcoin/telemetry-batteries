//! Trace-context propagation for outgoing `reqwest` requests.
//!
//! Use [`RequestBuilderExt`] to instrument individual requests. Enable the
//! `reqwest-middleware` feature and use
//! [`middleware::TraceContextMiddleware`] to instrument every request sent by
//! a `reqwest_middleware::ClientWithMiddleware`.

#[cfg(feature = "reqwest-middleware")]
pub mod middleware;

use http::HeaderMap;

use super::trace_to_headers;

/// Adds the current span's trace context to a `reqwest` request.
pub trait RequestBuilderExt: Sized {
    /// Inject the current span's trace context into this request's headers.
    ///
    /// Any existing propagation headers are replaced with values from the
    /// current context.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use telemetry_batteries::tracing::reqwest::RequestBuilderExt;
    ///
    /// # async fn send() -> Result<(), reqwest::Error> {
    /// let response = reqwest::Client::new()
    ///     .get("https://example.com")
    ///     .inject_trace_context()
    ///     .send()
    ///     .await?;
    /// # drop(response);
    /// # Ok(())
    /// # }
    /// ```
    fn inject_trace_context(self) -> Self;
}

impl RequestBuilderExt for ::reqwest::RequestBuilder {
    fn inject_trace_context(self) -> Self {
        let mut headers = HeaderMap::new();
        trace_to_headers(&mut headers);
        self.headers(headers)
    }
}
