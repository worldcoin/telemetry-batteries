use std::time::Duration;

use chrono::Utc;
use opentelemetry_datadog::ApiVersion;
use opentelemetry_sdk::trace::{Config, Sampler};
use serde::ser::SerializeMap;
use serde::Serializer;
use tracing::{Event, Subscriber};
use tracing_serde::AsSerde;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{fmt, Layer};

use crate::tracing::id_generator::ReducedIdGenerator;
use crate::tracing::{
    opentelemetry_span_id, opentelemetry_trace_id, WriteAdapter,
};

pub fn datadog_layer<S>(
    service_name: &str,
    endpoint: &str,
    location: bool,
) -> impl Layer<S>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    let tracer_config = Config::default()
        .with_id_generator(ReducedIdGenerator)
        .with_sampler(Sampler::AlwaysOn);

    // Small hack https://github.com/will-bank/datadog-tracing/blob/30cdfba8d00caa04f6ac8e304f76403a5eb97129/src/tracer.rs#L29
    // Until https://github.com/open-telemetry/opentelemetry-rust-contrib/issues/7 is resolved
    // seems to prevent client reuse and avoid the errors in question
    let dd_http_client = reqwest::ClientBuilder::new()
        .pool_idle_timeout(Duration::from_millis(1))
        .build()
        .expect("Could not init datadog http_client");

    let tracer = opentelemetry_datadog::new_pipeline()
        .with_http_client(dd_http_client)
        .with_agent_endpoint(endpoint)
        .with_trace_config(tracer_config)
        .with_service_name(service_name)
        .with_api_version(ApiVersion::Version05)
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .expect("failed to install OpenTelemetry datadog tracer, perhaps check which async runtime is being used");

    let otel_layer = tracing_opentelemetry::OpenTelemetryLayer::new(tracer);
    let dd_format_layer = datadog_format_layer(location);

    dd_format_layer.and_then(otel_layer)
}

pub fn datadog_format_layer<S>(location: bool) -> impl Layer<S>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fmt::Layer::new()
        .json()
        .event_format(DatadogFormat { location })
}

pub struct DatadogFormat {
    location: bool,
}

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

            if self.location {
                serializer.serialize_entry("line", &meta.line())?;
                serializer.serialize_entry("file", &meta.file())?;
                serializer
                    .serialize_entry("module_path", &meta.module_path())?;
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
