//! Tower middleware for distributed trace propagation.
//!
//! Provides a [`TraceLayer`] that automatically extracts trace context from
//! incoming request headers and injects it into outgoing response headers.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use http::{Request, Response};
use tower::{Layer, Service};
use tracing::{info_span, Instrument};

use super::{trace_from_headers, trace_to_headers};

/// Tower layer that propagates distributed trace context.
///
/// When applied to a service, this layer will:
/// 1. Create a request span
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
///     .layer(TraceLayer);
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct TraceLayer;

impl<S> Layer<S> for TraceLayer {
    type Service = TraceService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TraceService { inner }
    }
}

/// Tower service that propagates distributed trace context.
#[derive(Debug, Clone)]
pub struct TraceService<S> {
    inner: S,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for TraceService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send,
    ReqBody: Send + 'static,
    ResBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<ReqBody>) -> Self::Future {
        let inner = self.inner.clone();
        let inner = std::mem::replace(&mut self.inner, inner);

        Box::pin(async move {
            let method = request.method().clone();
            let uri = request.uri().clone();
            let path = uri.path().to_string();
            let query = uri.query().map(ToString::to_string);

            let span = info_span!(
                "request",
                http.method = %method,
                http.path = %path,
                http.query = ?query,
            );

            async move {
                // Extract trace context from incoming headers and attach to current span
                trace_from_headers(request.headers());

                let mut inner = inner;
                let mut response = inner.call(request).await?;

                // Inject trace context into response headers
                trace_to_headers(response.headers_mut());

                Ok(response)
            }
            .instrument(span)
            .await
        })
    }
}
