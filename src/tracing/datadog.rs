use super::{get_log_directory, TracingBattery};
use crate::error::BatteryError;
use crate::tracing::{opentelemetry_span_id, opentelemetry_trace_id, WriteAdapter};
use chrono::Utc;
use opentelemetry::sdk::trace;
use opentelemetry::sdk::trace::Sampler;
use serde::ser::SerializeMap;
use serde::Serializer;
use tokio::sync::OnceCell;
use tracing::{Event, Level, Subscriber};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::Rotation;
use tracing_serde::fields::AsMap;
use tracing_serde::AsSerde;
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::util::SubscriberInitExt;

pub const DEFAULT_AGENT_ENDPOINT: &str = "localhost:8126";

static WORKER_GUARD: OnceCell<WorkerGuard> = OnceCell::const_new();

pub struct DatadogBattery {
    pub endpoint: String,
    pub level: Level,
    pub service_name: String,
    pub rotation: Rotation,
}

impl DatadogBattery {
    pub fn new(
        level: Level,
        service_name: &str,
        rotation: Rotation,
        endpoint: Option<&str>,
    ) -> Self {
        Self {
            level,
            service_name: service_name.to_string(),
            rotation,
            endpoint: endpoint.unwrap_or(DEFAULT_AGENT_ENDPOINT).to_string(),
        }
    }
}

impl TracingBattery for DatadogBattery {
    fn init(&self) -> Result<(), BatteryError> {
        let service_name = self.service_name.as_str();

        let tracer_config = trace::config().with_sampler(Sampler::AlwaysOn);

        let tracer = opentelemetry_datadog::new_pipeline()
            .with_agent_endpoint(self.endpoint.clone())
            .with_trace_config(tracer_config)
            .with_service_name(service_name)
            .with_api_version(opentelemetry_datadog::ApiVersion::Version05)
            .install_batch(opentelemetry::runtime::Tokio)?;

        let otel_layer = tracing_opentelemetry::OpenTelemetryLayer::new(tracer);

        let filter = tracing_subscriber::filter::EnvFilter::from_default_env()
            .add_directive(self.level.into());

        let fmt_layer = fmt::layer().with_target(false).with_level(true);

        //TODO: conditionally initialize logs
        let file_appender = tracing_appender::rolling::RollingFileAppender::new(
            self.rotation.clone(),
            get_log_directory()?,
            format!("{service_name}.log"),
        );

        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        WORKER_GUARD.set(guard)?;

        let dd_layer = fmt::Layer::new()
            .json()
            .event_format(DataDogFormat)
            .with_writer(non_blocking);

        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .with(dd_layer)
            .with(otel_layer)
            .init();

        Ok(())
    }
}

pub struct DataDogFormat;

impl<S, N> FormatEvent<S, N> for DataDogFormat
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
            serializer.serialize_entry("fields", &event.field_map())?;
            serializer.serialize_entry("target", meta.target())?;

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
