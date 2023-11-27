use crate::error::BatteryError;
use crate::tracing::layers::datadog::DatadogLayer;
use crate::tracing::layers::StdoutLayer;
use crate::tracing::{opentelemetry_span_id, opentelemetry_trace_id, WriteAdapter};
use chrono::Utc;
use opentelemetry::sdk::trace;
use opentelemetry::sdk::trace::Sampler;
use serde::ser::SerializeMap;
use serde::Serializer;
use tokio::sync::OnceCell;
use tracing::{Event, Level, Subscriber};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_serde::fields::AsMap;
use tracing_serde::AsSerde;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

use super::TracingBattery;

pub const DEFAULT_AGENT_ENDPOINT: &str = "localhost:8126";

static WORKER_GUARD: OnceCell<WorkerGuard> = OnceCell::const_new();

pub struct DatadogBattery {
    pub endpoint: String,
    pub level: Level,
    pub service_name: String,
    pub file_appender: Option<RollingFileAppender>,
}

impl DatadogBattery {
    pub fn new(
        endpoint: Option<&str>,
        level: Level,
        service_name: &str,
        file_appender: Option<RollingFileAppender>,
    ) -> Self {
        Self {
            level,
            service_name: service_name.to_string(),
            endpoint: endpoint.unwrap_or(DEFAULT_AGENT_ENDPOINT).to_string(),
            file_appender,
        }
    }
}

impl DatadogBattery {
    pub fn init(self) -> Result<(), BatteryError> {
        let datadog_layer = DatadogLayer::new(&self.service_name, &self.endpoint, self.level)?;

        if let Some(file_appender) = self.file_appender {
            let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
            WORKER_GUARD.set(guard)?;
            let file_layer = tracing_subscriber::fmt::layer().with_writer(non_blocking);

            let layers = datadog_layer.and_then(file_layer);
            TracingBattery::init(layers);
        } else {
            let std_out_layer = StdoutLayer::new(self.level);
            let layers = datadog_layer.and_then(std_out_layer);

            TracingBattery::init(layers);
        }

        Ok(())
    }
}
