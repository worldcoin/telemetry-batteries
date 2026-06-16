//! Tower middleware for distributed trace propagation.
//!
//! Provides a [`TraceLayer`] that automatically extracts trace context from
//! incoming request headers and injects it into outgoing response headers.
//!
//! This module also exposes a set of small helpers that match the conventions
//! used by `will-bank/datadog-tracing`:
//!
//! - [`make_span_from_request`] — build a richly-tagged server span from an
//!   `http::Request`, populating the standard OpenTelemetry `http.*`,
//!   `url.*`, `network.*`, `server.*`, `user_agent.*` and Datadog
//!   compatibility fields up-front.
//! - [`update_span_from_response`] — record the response status code and
//!   set `otel.status_code = ERROR` on 5xx responses.
//! - [`update_span_from_error`] — record `otel.status_code = ERROR` and the
//!   error's message under `exception.message`.
//! - [`update_span_from_response_or_error`] — convenience dispatcher used
//!   when wrapping a fallible handler.

use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use http::{Request, Response};
use tower::{Layer, Service};
use tracing::field::Empty;
use tracing::{Instrument, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_opentelemetry_instrumentation_sdk::TRACING_TARGET;
use tracing_opentelemetry_instrumentation_sdk::http::{
    http_flavor, http_host, url_scheme, user_agent,
};

/// Function type for creating custom spans.
pub type MakeSpan = fn(&http::Request<()>) -> Span;

/// Build a richly-tagged server span for an HTTP request.
///
/// The set of fields mirrors the conventions used by
/// `will-bank/datadog-tracing` and includes the standard OpenTelemetry
/// `http.*`/`url.*`/`network.*`/`server.*`/`user_agent.*` attributes
/// alongside Datadog-specific aliases (`http.status_code`, `span.type`).
///
/// Fields that are only known once the response is produced (status code,
/// route, client address, trace/request ids, exception message) are declared
/// as [`tracing::field::Empty`] so they can be populated later via
/// [`update_span_from_response`] / [`update_span_from_error`].
pub fn make_span_from_request<B>(req: &http::Request<B>) -> Span {
    let http_method = req.method();
    tracing::trace_span!(
        target: TRACING_TARGET,
        "HTTP request",
        http.request.method = %http_method,
        http.route = Empty,
        network.protocol.version = %http_flavor(req.version()),
        server.address = http_host(req),
        http.client.address = Empty,
        user_agent.original = user_agent(req),
        http.response.status_code = Empty,
        // Datadog alias kept alongside the OTel-native field.
        http.status_code = Empty,
        url.path = req.uri().path(),
        url.query = req.uri().query(),
        url.scheme = url_scheme(req.uri()),
        otel.name = %http_method,
        otel.kind = ?opentelemetry::trace::SpanKind::Server,
        otel.status_code = Empty,
        trace_id = Empty,
        request_id = Empty,
        exception.message = Empty,
        // Datadog-specific, non-standard OTel.
        "span.type" = "web",
    )
}

/// Record the response status code on the span and mark 5xx responses as
/// errored for OpenTelemetry / Datadog.
pub fn update_span_from_response<B>(span: &Span, response: &http::Response<B>) {
    let status = response.status();
    span.record("http.response.status_code", status.as_u16());
    span.record("http.status_code", status.as_u16());
    if status.is_server_error() {
        span.record("otel.status_code", "ERROR");
    }
}

/// Record an error on the span: sets `otel.status_code = ERROR` and copies
/// the error's `Display` representation into `exception.message`. If the
/// error chains a source it is recorded as well.
pub fn update_span_from_error<E: Error>(span: &Span, error: &E) {
    span.record("otel.status_code", "ERROR");
    span.record("exception.message", error.to_string());
    if let Some(source) = error.source() {
        span.record("exception.message", source.to_string());
    }
}

/// Dispatch helper that updates the span from either a successful response
/// or an error.
pub fn update_span_from_response_or_error<B, E: Error>(
    span: &Span,
    result: &Result<http::Response<B>, E>,
) {
    match result {
        Ok(response) => update_span_from_response(span, response),
        Err(error) => update_span_from_error(span, error),
    }
}

fn default_make_span(request: &http::Request<()>) -> Span {
    make_span_from_request(request)
}

/// Tower layer that propagates distributed trace context.
///
/// When applied to a service, this layer will:
/// 1. Create a request span (customizable via [`with_make_span`](Self::with_make_span))
/// 2. Extract trace context from incoming request headers (e.g., `traceparent`)
/// 3. Run the inner service within the span
/// 4. Inject trace context into outgoing response headers
///
/// # Example
///
/// ```ignore
/// use axum::Router;
/// use telemetry_batteries::tracing::middleware::TraceLayer;
///
/// let app = Router::new()
///     .route("/", get(handler))
///     .layer(TraceLayer::new());
/// ```
///
/// # Custom Span
///
/// ```ignore
/// use telemetry_batteries::tracing::middleware::TraceLayer;
/// use tracing::info_span;
///
/// let layer = TraceLayer::new().with_make_span(|req| {
///     info_span!(
///         "http_request",
///         method = %req.method(),
///         path = %req.uri().path(),
///     )
/// });
/// ```
#[derive(Debug, Clone, Copy)]
pub struct TraceLayer {
    make_span: MakeSpan,
}

impl Default for TraceLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl TraceLayer {
    /// Create a new `TraceLayer` with default settings.
    pub fn new() -> Self {
        Self {
            make_span: default_make_span,
        }
    }

    /// Set a custom function for creating the request span.
    ///
    /// The function receives a reference to the request (with an empty body type)
    /// and should return a `Span`.
    pub fn with_make_span(mut self, make_span: MakeSpan) -> Self {
        self.make_span = make_span;
        self
    }
}

impl<S> Layer<S> for TraceLayer {
    type Service = TraceService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TraceService {
            inner,
            make_span: self.make_span,
        }
    }
}

/// Tower service that propagates distributed trace context.
#[derive(Debug, Clone)]
pub struct TraceService<S> {
    inner: S,
    make_span: MakeSpan,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for TraceService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>
        + Clone
        + Send
        + 'static,
    S::Future: Send,
    ReqBody: Send + 'static,
    ResBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<
        Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<ReqBody>) -> Self::Future {
        // Clone to satisfy borrow checker for the async block
        let inner = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, inner);

        // Build a body-erased view of the request for span creation. We
        // copy the headers across so helpers like `http_host` and
        // `user_agent` can populate the span correctly.
        let mut span_request_builder = http::Request::builder()
            .method(request.method().clone())
            .uri(request.uri().clone())
            .version(request.version());
        if let Some(headers) = span_request_builder.headers_mut() {
            *headers = request.headers().clone();
        }
        let span_request = span_request_builder
            .body(())
            .expect("request builder with () body cannot fail");

        let span = (self.make_span)(&span_request);
        let _ = span.set_parent(
            opentelemetry::global::get_text_map_propagator(|propagator| {
                propagator.extract(&opentelemetry_http::HeaderExtractor(
                    request.headers(),
                ))
            }),
        );

        let span_for_response = span.clone();
        Box::pin(
            async move {
                let result = inner.call(request).await;
                if let Ok(ref response) = result {
                    update_span_from_response(&span_for_response, response);
                }
                result
            }
            .instrument(span),
        )
    }
}
