pub(crate) mod datadog;
pub(crate) mod id_generator;
pub mod layers;
pub mod middleware;
pub mod reqwest;
pub(crate) mod stdout;

use opentelemetry::Context;
use opentelemetry::trace::{SpanContext, SpanId, TraceContextExt, TraceId};
pub(crate) use opentelemetry_sdk::trace::SdkTracerProvider;

use std::io;
use tracing_opentelemetry::OpenTelemetrySpanExt;
pub use tracing_subscriber::Registry;

/// Handle that shuts down the tracing provider when dropped.
#[must_use]
pub(crate) struct TracingShutdownHandle {
    provider: Option<SdkTracerProvider>,
}

impl TracingShutdownHandle {
    pub fn new(provider: SdkTracerProvider) -> Self {
        Self {
            provider: Some(provider),
        }
    }

    pub fn empty() -> Self {
        Self { provider: None }
    }
}

impl Drop for TracingShutdownHandle {
    fn drop(&mut self) {
        if let Some(provider) = self.provider.take()
            && let Err(e) = provider.shutdown()
        {
            tracing::warn!("Failed to shutdown tracer provider: {e}");
        }
    }
}

pub fn trace_to_headers(headers: &mut http::HeaderMap) {
    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.inject_context(
            &tracing::Span::current().context(),
            &mut opentelemetry_http::HeaderInjector(headers),
        );
    });
}

/// Otel trace id of the currently active span, if any.
///
/// Reads the OpenTelemetry context that `OpenTelemetryLayer` attaches to the
/// thread when a tracing span is entered, so this only reports ids for spans
/// that are currently entered.
pub fn opentelemetry_trace_id() -> Option<u128> {
    let trace_id = current_span_context()?.trace_id();
    Some(u128::from_be_bytes(trace_id.to_bytes()))
}

/// Otel span id of the currently active span, if any.
///
/// Same caveat as [`opentelemetry_trace_id`]: the id is only available while
/// the span is entered, so span end events report nothing.
pub fn opentelemetry_span_id() -> Option<u64> {
    let span_id = current_span_context()?.span_id();
    Some(u64::from_be_bytes(span_id.to_bytes()))
}

/// Span context of the active Otel span, or `None` when no span is active or
/// no Otel layer is installed.
fn current_span_context() -> Option<SpanContext> {
    let context = Context::current();
    let span_context = context.span().span_context().clone();

    if span_context.is_valid() {
        Some(span_context)
    } else {
        None
    }
}

/// Sets the current span's parent to the specified context
pub fn trace_from_ctx(ctx: SpanContext) {
    let parent_ctx = Context::new().with_remote_span_context(ctx);
    let _ = tracing::Span::current().set_parent(parent_ctx);
}

// Extracts the trace id and span id from the current span
pub fn extract_span_ids() -> (TraceId, SpanId) {
    let current_span = tracing::Span::current();
    let current_context = current_span.context();
    let span_ref = current_context.span();

    let span_context = span_ref.span_context();
    let trace_id = span_context.trace_id();
    let span_id = span_context.span_id();

    (trace_id, span_id)
}

pub struct WriteAdapter<'a> {
    fmt_write: &'a mut dyn std::fmt::Write,
}

impl<'a> WriteAdapter<'a> {
    pub fn new(fmt_write: &'a mut dyn std::fmt::Write) -> Self {
        Self { fmt_write }
    }
}

impl io::Write for WriteAdapter<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let s = std::str::from_utf8(buf)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        self.fmt_write.write_str(s).map_err(io::Error::other)?;

        Ok(s.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
