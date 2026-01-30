use std::time::Duration;

use chrono::Utc;
use opentelemetry::trace::TracerProvider;
use opentelemetry_datadog::ApiVersion;
use opentelemetry_sdk::trace::{Config, Sampler, SdkTracerProvider};
use serde::Serializer;
use serde::ser::SerializeMap;
use tracing::{Event, Subscriber};
use tracing_serde::AsSerde;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields};
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
    let mut tracer_config = Config::default();
    tracer_config.sampler = Box::new(Sampler::AlwaysOn);
    tracer_config.id_generator = Box::new(ReducedIdGenerator);

    // Small hack https://github.com/will-bank/datadog-tracing/blob/30cdfba8d00caa04f6ac8e304f76403a5eb97129/src/tracer.rs#L29
    // Until https://github.com/open-telemetry/opentelemetry-rust-contrib/issues/7 is resolved
    // seems to prevent client reuse and avoid the errors in question
    let dd_http_client = reqwest::ClientBuilder::new()
        .pool_idle_timeout(Duration::from_millis(1))
        .build()
        .expect("Could not init datadog http_client");

    let provider = opentelemetry_datadog::new_pipeline()
        .with_http_client(dd_http_client)
        .with_agent_endpoint(endpoint)
        .with_trace_config(tracer_config)
        .with_service_name(service_name)
        .with_api_version(ApiVersion::Version05)
        .install_batch()
        .expect("failed to install OpenTelemetry datadog tracer, perhaps check which async runtime is being used");

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

/// Custom JSON formatter for Datadog that includes trace correlation fields.
///
/// Output format:
/// ```json
/// {"timestamp":"...","level":"INFO","target":"my_app","message":"...","dd.trace_id":"123","dd.span_id":"456"}
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
