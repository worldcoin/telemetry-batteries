use chrono::Utc;
use opentelemetry::sdk::trace;
use opentelemetry::sdk::trace::Sampler;
use opentelemetry_datadog::ApiVersion;
use serde::ser::SerializeMap;
use serde::Serializer;
use tracing::{Event, Level, Subscriber};
use tracing_serde::AsSerde;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{fmt, EnvFilter, Layer};

use crate::error::BatteryError;
use crate::tracing::{opentelemetry_span_id, opentelemetry_trace_id, WriteAdapter};

pub struct DatadogLayer;

impl DatadogLayer {
    pub fn layer<S>(
        service_name: &str,
        endpoint: &str,
        level: Level,
    ) -> Result<impl Layer<S>, BatteryError>
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        let tracer_config = trace::config().with_sampler(Sampler::AlwaysOn);

        let tracer = opentelemetry_datadog::new_pipeline()
            .with_agent_endpoint(endpoint)
            .with_trace_config(tracer_config)
            .with_service_name(service_name)
            .with_api_version(ApiVersion::Version05)
            .install_batch(opentelemetry::runtime::Tokio)?;

        let otel_layer = tracing_opentelemetry::OpenTelemetryLayer::new(tracer);
        let filter = EnvFilter::from_default_env().add_directive(level.into());
        let dd_format_layer = DatadogFormatLayer::layer();

        Ok(filter.and_then(dd_format_layer).and_then(otel_layer))
    }
}

pub struct DatadogFormatLayer;

impl DatadogFormatLayer {
    pub fn layer<S>() -> impl Layer<S>
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        fmt::Layer::new().json().event_format(DatadogFormat)
    }
}

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
            let mut serializer = serde_json::Serializer::new(WriteAdapter::new(&mut writer));
            let mut serializer = serializer.serialize_map(None)?;

            serializer.serialize_entry("timestamp", &Utc::now().to_rfc3339())?;
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
                let span_id = format!("{}", span_id);
                serializer.serialize_entry("dd.span_id", &span_id)?;
            }

            serializer.end()
        };

        visit().map_err(|_| std::fmt::Error)?;

        writeln!(writer)
    }
}
