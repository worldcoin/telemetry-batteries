use tokio::sync::OnceCell;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::RollingFileAppender;
use tracing_subscriber::{EnvFilter, Layer};

use super::TracingBattery;
use crate::error::BatteryError;
use crate::tracing::layers::datadog::DatadogLayer;

pub const DEFAULT_AGENT_ENDPOINT: &str = "http://localhost:8126";

static WORKER_GUARD: OnceCell<WorkerGuard> = OnceCell::const_new();

pub struct DatadogBattery {
    pub endpoint: String,
    pub env_filter: EnvFilter,
    pub service_name: String,
    pub file_appender: Option<RollingFileAppender>,
    pub location: bool,
}

impl DatadogBattery {
    pub fn new(
        endpoint: Option<&str>,
        service_name: &str,
        file_appender: Option<RollingFileAppender>,
    ) -> Self {
        Self {
            env_filter: EnvFilter::from_default_env(),
            service_name: service_name.to_string(),
            endpoint: endpoint.unwrap_or(DEFAULT_AGENT_ENDPOINT).to_string(),
            file_appender,
            location: false,
        }
    }

    pub fn with_location(mut self) -> Self {
        self.location = true;
        self
    }
}

impl DatadogBattery {
    pub fn init(self) -> Result<(), BatteryError> {
        let mut datadog_layer =
            DatadogLayer::new(&self.service_name, &self.endpoint);

        datadog_layer.location = self.location;
        datadog_layer.env_filter = self.env_filter;

        let datadog_layer = datadog_layer.into_layer()?;

        if let Some(file_appender) = self.file_appender {
            let (non_blocking, guard) =
                tracing_appender::non_blocking(file_appender);
            WORKER_GUARD.set(guard)?;
            let file_layer =
                tracing_subscriber::fmt::layer().with_writer(non_blocking);

            let layers = datadog_layer.and_then(file_layer);
            TracingBattery::init(Some(layers));
        } else {
            TracingBattery::init(Some(datadog_layer));
        }

        Ok(())
    }
}
