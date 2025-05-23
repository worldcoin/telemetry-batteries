pub mod datadog;
pub mod id_generator;
pub mod layers;
pub mod stdout;

use opentelemetry::trace::{SpanContext, SpanId, TraceContextExt, TraceId};
use opentelemetry::Context;

use std::path::PathBuf;
use std::{fs, io};
use tracing::Subscriber;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_opentelemetry::OtelData;
use tracing_subscriber::fmt::{FmtContext, FormatFields};
use tracing_subscriber::registry::{LookupSpan, SpanRef};
pub use tracing_subscriber::Registry;

/// `TracingShutdownHandle` ensures the global tracing provider
/// is gracefully shut down when the handle is dropped, preventing loss
/// of any remaining traces not yet exported.
#[must_use]
pub struct TracingShutdownHandle;

impl Drop for TracingShutdownHandle {
    fn drop(&mut self) {
        tracing::warn!("Shutting down tracing provider");
        opentelemetry::global::shutdown_tracer_provider();
    }
}

pub fn trace_from_headers(headers: &http::HeaderMap) {
    tracing::Span::current().set_parent(
        opentelemetry::global::get_text_map_propagator(|propagator| {
            propagator.extract(&opentelemetry_http::HeaderExtractor(headers))
        }),
    );
}

pub fn trace_to_headers(headers: &mut http::HeaderMap) {
    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.inject_context(
            &tracing::Span::current().context(),
            &mut opentelemetry_http::HeaderInjector(headers),
        );
    });
}

/// Finds Otel trace id by going up the span stack until we find a span
/// with a trace id.
pub fn opentelemetry_trace_id<S, N>(ctx: &FmtContext<'_, S, N>) -> Option<u128>
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    let span_ref = span_from_ctx(ctx)?;

    let extensions = span_ref.extensions();

    let data = extensions.get::<OtelData>()?;
    let parent_trace_id = data.parent_cx.span().span_context().trace_id();
    let parent_trace_id_u128 = u128::from_be_bytes(parent_trace_id.to_bytes());

    // So parent trace id will usually be zero UNLESS we extract a trace id from
    // headers in which case it'll be the trace id from headers. And for some
    // reason this logic is not handled with Option
    //
    // So in case the parent trace id is zero, we should use the builder trace id.
    if parent_trace_id_u128 == 0 {
        let builder_id = data.builder.trace_id?;

        Some(u128::from_be_bytes(builder_id.to_bytes()))
    } else {
        Some(parent_trace_id_u128)
    }
}

/// Finds Otel span id
///
/// BUG: The otel object is not available for span end events. This is
/// because the Otel layer is higher in the stack and removes the
/// extension before we get here.
///
/// Fallbacks on tracing span id
pub fn opentelemetry_span_id<S, N>(ctx: &FmtContext<'_, S, N>) -> Option<u64>
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    let span_ref = span_from_ctx(ctx)?;

    let extensions = span_ref.extensions();

    let data = extensions.get::<OtelData>()?;
    let parent_span_id = data.parent_cx.span().span_context().span_id();
    let parent_span_id_u64 = u64::from_be_bytes(parent_span_id.to_bytes());

    // Same logic as for trace ids
    if parent_span_id_u64 == 0 {
        let builder_id = data.builder.span_id?;

        Some(u64::from_be_bytes(builder_id.to_bytes()))
    } else {
        Some(parent_span_id_u64)
    }
}

/// Sets the current span's parent to the specified context
pub fn trace_from_ctx(ctx: SpanContext) {
    let parent_ctx = Context::new().with_remote_span_context(ctx);
    tracing::Span::current().set_parent(parent_ctx);
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

fn span_from_ctx<'a, S, N>(
    ctx: &'a FmtContext<'a, S, N>,
) -> Option<SpanRef<'a, S>>
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    let span = ctx.lookup_current().or_else(|| ctx.parent_span());

    span
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

/// Platform agnostic function to get the path to the log directory. If the directory does not
/// exist, it will be created.
///
/// # Returns
/// * `Ok(PathBuf)` containing the path to the `.logs` directory in the user's home directory.
/// * `Err(io::Error)` if the home directory cannot be determined, or the `.logs` directory
///   cannot be created.
///
/// # Errors
/// This function will return an `Err` if the home directory cannot be found or the `.logs`
/// directory cannot be created. It does not guarantee that the `.logs` directory is writable.
pub fn get_log_directory() -> Result<PathBuf, io::Error> {
    let home_dir = dirs::home_dir().ok_or(io::ErrorKind::NotFound)?;
    let log_dir = home_dir.join(".logs");

    // Create the `.logs` directory if it does not exist
    if !log_dir.exists() {
        fs::create_dir_all(&log_dir)?;
    }

    Ok(log_dir)
}
