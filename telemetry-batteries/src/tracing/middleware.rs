//! Tower middleware for distributed trace propagation.
//!
//! Provides a [`TraceLayer`] that automatically extracts trace context from
//! incoming request headers and injects it into outgoing response headers.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use http::{Request, Response};
use tower::{Layer, Service};
use tracing::{info_span, Instrument, Span};

use super::{trace_from_headers, trace_to_headers};

/// Function type for creating custom spans.
pub type MakeSpan = fn(&http::Request<()>) -> Span;

fn default_make_span(request: &http::Request<()>) -> Span {
    info_span!(
        "request",
        http.method = %request.method(),
        http.path = %request.uri().path(),
        http.query = ?request.uri().query(),
    )
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
        let inner = std::mem::replace(&mut self.inner, inner);

        // Create a temporary request view for span creation (avoids body type issues)
        let span_request = http::Request::builder()
            .method(request.method().clone())
            .uri(request.uri().clone())
            .body(())
            .expect("request builder with () body cannot fail");

        let span = (self.make_span)(&span_request);

        Box::pin(
            async move {
                // Extract trace context from incoming headers and attach to current span
                trace_from_headers(request.headers());

                let mut inner = inner;
                let mut response = inner.call(request).await?;

                // Inject trace context into response headers
                trace_to_headers(response.headers_mut());

                Ok(response)
            }
            .instrument(span),
        )
    }
}
