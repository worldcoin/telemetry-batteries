use std::time::Duration;

use chrono::Utc;
use opentelemetry::trace::TracerProvider;
use opentelemetry_datadog::ApiVersion;
use opentelemetry_sdk::runtime::Tokio;
use opentelemetry_sdk::trace::span_processor_with_async_runtime::BatchSpanProcessor;
use opentelemetry_sdk::trace::{Sampler, SdkTracerProvider};
use serde::Serializer;
use serde::ser::SerializeMap;
use tracing::{Event, Subscriber};
use tracing_serde::AsSerde;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields, FormattedFields};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{Layer, fmt};

use crate::config::LogFormat;
use crate::tracing::id_generator::ReducedIdGenerator;
use crate::tracing::{
    WriteAdapter, opentelemetry_span_id, opentelemetry_trace_id,
};

pub fn datadog_layer<S>(
    service_name: &str,
    endpoint: &str,
    log_format: LogFormat,
) -> (impl Layer<S>, SdkTracerProvider)
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    // Small hack https://github.com/will-bank/datadog-tracing/blob/30cdfba8d00caa04f6ac8e304f76403a5eb97129/src/tracer.rs#L29
    // Until https://github.com/open-telemetry/opentelemetry-rust-contrib/issues/7 is resolved
    // seems to prevent client reuse and avoid the errors in question
    let dd_http_client = reqwest::ClientBuilder::new()
        .pool_idle_timeout(Duration::from_millis(1))
        .build()
        .expect("Could not init datadog http_client");

    // Build the exporter manually so we can use a tokio-based BatchSpanProcessor.
    // The default install_batch() spawns a thread without tokio runtime, which causes
    // reqwest to panic when doing DNS resolution.
    let exporter = opentelemetry_datadog::new_pipeline()
        .with_http_client(dd_http_client)
        .with_agent_endpoint(endpoint)
        .with_service_name(service_name)
        .with_api_version(ApiVersion::Version05)
        .build_exporter()
        .expect("failed to build OpenTelemetry datadog exporter");

    let batch_processor = BatchSpanProcessor::builder(exporter, Tokio).build();

    let provider = SdkTracerProvider::builder()
        .with_span_processor(batch_processor)
        .with_sampler(Sampler::AlwaysOn)
        .with_id_generator(ReducedIdGenerator)
        .build();

    // Set as global tracer provider
    opentelemetry::global::set_tracer_provider(provider.clone());

    // Use a static string for the tracer name since provider.tracer() requires 'static
    let tracer = provider.tracer("telemetry-batteries");
    let otel_layer = tracing_opentelemetry::OpenTelemetryLayer::new(tracer);
    let format_layer = datadog_format_layer(log_format);

    (format_layer.and_then(otel_layer), provider)
}

/// Create a format layer for Datadog.
///
/// If `log_format` is `DatadogJson`, uses the custom Datadog JSON format with trace correlation.
/// Otherwise, uses the standard tracing-subscriber formats.
pub fn datadog_format_layer<S>(
    log_format: LogFormat,
) -> Box<dyn Layer<S> + Send + Sync>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    match log_format {
        LogFormat::DatadogJson => {
            Box::new(fmt::Layer::new().json().event_format(DatadogFormat))
        }
        LogFormat::Pretty => Box::new(
            fmt::Layer::new()
                .with_writer(std::io::stdout)
                .pretty()
                .with_target(false)
                .with_line_number(true)
                .with_file(true),
        ),
        LogFormat::Json => {
            Box::new(fmt::Layer::new().with_writer(std::io::stdout).json())
        }
        LogFormat::Compact => {
            Box::new(fmt::Layer::new().with_writer(std::io::stdout).compact())
        }
    }
}

/// Custom JSON formatter for Datadog that includes trace correlation fields
/// and propagates span fields into the log output.
///
/// Fields from ancestor spans (set via `#[instrument(fields(...))]` or
/// `tracing::info_span!`) are flattened into each log line. Inner spans
/// override outer spans, and event-level fields override all span fields.
///
/// Output format:
/// ```json
/// {"timestamp":"...","level":"INFO","target":"my_app","request_id":"abc","message":"...","dd.trace_id":"123","dd.span_id":"456"}
/// ```
pub struct DatadogFormat;

impl<S, N> FormatEvent<S, N> for DatadogFormat
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> std::fmt::Result
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        let meta = event.metadata();

        let span_id = opentelemetry_span_id(ctx);
        let trace_id = opentelemetry_trace_id(ctx);

        let mut visit = || {
            let mut serializer =
                serde_json::Serializer::new(WriteAdapter::new(&mut writer));
            let mut serializer = serializer.serialize_map(None)?;

            serializer
                .serialize_entry("timestamp", &Utc::now().to_rfc3339())?;
            serializer.serialize_entry("level", &meta.level().as_serde())?;
            serializer.serialize_entry("target", meta.target())?;

            // Propagate fields from ancestor spans (outer-to-inner).
            // Inner span fields override outer; event fields override all.
            if let Some(scope) = ctx.event_scope() {
                for span in scope.from_root() {
                    let extensions = span.extensions();
                    if let Some(fields) = extensions.get::<FormattedFields<N>>() {
                        if !fields.is_empty() {
                            if let Ok(serde_json::Value::Object(fields)) =
                                serde_json::from_str::<serde_json::Value>(fields)
                            {
                                for (key, value) in fields {
                                    serializer.serialize_entry(&key, &value)?;
                                }
                            }
                        }
                    }
                }
            }

            let mut visitor = tracing_serde::SerdeMapVisitor::new(serializer);
            event.record(&mut visitor);
            serializer = visitor.take_serializer()?;

            if let Some(trace_id) = trace_id {
                // The opentelemetry-datadog crate truncates the 128-bit trace-id
                // into a u64 before formatting it.
                let trace_id = format!("{}", trace_id as u64);
                serializer.serialize_entry("dd.trace_id", &trace_id)?;
            }

            if let Some(span_id) = span_id {
                let span_id = format!("{span_id}");
                serializer.serialize_entry("dd.span_id", &span_id)?;
            }

            serializer.end()
        };

        visit().map_err(|_| std::fmt::Error)?;

        writeln!(writer)
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::sync::{Arc, Mutex};

    use tracing_subscriber::fmt::MakeWriter;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::fmt;

    use super::DatadogFormat;

    #[derive(Clone)]
    struct BufWriter(Arc<Mutex<Vec<u8>>>);

    impl BufWriter {
        fn new() -> Self {
            Self(Arc::new(Mutex::new(Vec::new())))
        }

        fn contents(&self) -> String {
            String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
        }
    }

    impl io::Write for BufWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl<'a> MakeWriter<'a> for BufWriter {
        type Writer = BufWriter;
        fn make_writer(&'a self) -> Self::Writer {
            self.clone()
        }
    }

    fn parse_log_line(raw: &str) -> serde_json::Value {
        let line = raw.trim();
        serde_json::from_str(line).expect("log line is not valid JSON")
    }

    #[test]
    fn test_event_without_spans_has_no_span_fields() {
        let buf = BufWriter::new();
        let layer = fmt::Layer::new()
            .json()
            .event_format(DatadogFormat)
            .with_writer(buf.clone());

        let subscriber = tracing_subscriber::registry().with(layer);

        tracing::subscriber::with_default(subscriber, || {
            tracing::info!("bare event");
        });

        let log = parse_log_line(&buf.contents());
        assert_eq!(log["message"], "bare event");
        assert!(log.get("dd.trace_id").is_none());
    }

    #[test]
    fn test_span_fields_propagated_to_event() {
        let buf = BufWriter::new();
        let layer = fmt::Layer::new()
            .json()
            .event_format(DatadogFormat)
            .with_writer(buf.clone());

        let subscriber = tracing_subscriber::registry().with(layer);

        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!("my_span", request_id = "abc-123");
            let _guard = span.enter();
            tracing::info!("inside span");
        });

        let log = parse_log_line(&buf.contents());
        assert_eq!(log["message"], "inside span");
        assert_eq!(log["request_id"], "abc-123");
    }

    #[test]
    fn test_nested_span_fields_inner_overrides_outer() {
        let buf = BufWriter::new();
        let layer = fmt::Layer::new()
            .json()
            .event_format(DatadogFormat)
            .with_writer(buf.clone());

        let subscriber = tracing_subscriber::registry().with(layer);

        tracing::subscriber::with_default(subscriber, || {
            let outer = tracing::info_span!("outer", shared = "from_outer", outer_only = "yes");
            let _outer_guard = outer.enter();
            let inner = tracing::info_span!("inner", shared = "from_inner", inner_only = "yes");
            let _inner_guard = inner.enter();
            tracing::info!("nested event");
        });

        let log = parse_log_line(&buf.contents());
        assert_eq!(log["message"], "nested event");
        assert_eq!(log["shared"], "from_inner");
        assert_eq!(log["outer_only"], "yes");
        assert_eq!(log["inner_only"], "yes");
    }

    #[test]
    fn test_event_fields_override_span_fields() {
        let buf = BufWriter::new();
        let layer = fmt::Layer::new()
            .json()
            .event_format(DatadogFormat)
            .with_writer(buf.clone());

        let subscriber = tracing_subscriber::registry().with(layer);

        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!("my_span", user = "from_span");
            let _guard = span.enter();
            tracing::info!(user = "from_event", "with override");
        });

        let log = parse_log_line(&buf.contents());
        assert_eq!(log["message"], "with override");
        assert_eq!(log["user"], "from_event");
    }
}
