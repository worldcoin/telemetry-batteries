//! Tower middleware for distributed trace propagation.
//!
//! Provides a [`TraceLayer`] that automatically extracts trace context from
//! incoming request headers and injects it into outgoing response headers.

use std::task::{Context, Poll};

use http::{Request, Response};
use tower::{Layer, Service};

use super::{trace_from_headers, trace_to_headers};

/// Tower layer that propagates distributed trace context.
///
/// When applied to a service, this layer will:
/// 1. Extract trace context from incoming request headers (e.g., `traceparent`)
/// 2. Inject trace context into outgoing response headers
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
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = TraceFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<ReqBody>) -> Self::Future {
        // Extract trace context from incoming request headers
        trace_from_headers(request.headers());

        TraceFuture {
            inner: self.inner.call(request),
        }
    }
}

/// Future that injects trace context into the response.
#[pin_project::pin_project]
pub struct TraceFuture<F> {
    #[pin]
    inner: F,
}

impl<F, ResBody, E> std::future::Future for TraceFuture<F>
where
    F: std::future::Future<Output = Result<Response<ResBody>, E>>,
{
    type Output = Result<Response<ResBody>, E>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        match this.inner.poll(cx) {
            Poll::Ready(Ok(mut response)) => {
                // Inject trace context into response headers
                trace_to_headers(response.headers_mut());
                Poll::Ready(Ok(response))
            }
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
}
