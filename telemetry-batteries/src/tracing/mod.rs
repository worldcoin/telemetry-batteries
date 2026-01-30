pub(crate) mod datadog;
pub(crate) mod id_generator;
pub mod layers;
pub mod middleware;
pub(crate) mod stdout;

use opentelemetry::Context;
use opentelemetry::trace::{SpanContext, SpanId, TraceContextExt, TraceId};
pub(crate) use opentelemetry_sdk::trace::SdkTracerProvider;

use std::path::PathBuf;
use std::{fs, io};
use tracing::Subscriber;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_opentelemetry::OtelData;
pub use tracing_subscriber::Registry;
use tracing_subscriber::fmt::{FmtContext, FormatFields};
use tracing_subscriber::registry::{LookupSpan, SpanRef};

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

pub fn trace_from_headers(headers: &http::HeaderMap) {
    let _ = tracing::Span::current().set_parent(
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
    let trace_id = data.trace_id()?;
    Some(u128::from_be_bytes(trace_id.to_bytes()))
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
    let span_id = data.span_id()?;
    Some(u64::from_be_bytes(span_id.to_bytes()))
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

fn span_from_ctx<'a, S, N>(
    ctx: &'a FmtContext<'a, S, N>,
) -> Option<SpanRef<'a, S>>
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    ctx.lookup_current().or_else(|| ctx.parent_span())
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
