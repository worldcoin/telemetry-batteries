use metrics::MetricsBattery;
use tracing::TracingBattery;

pub mod metrics;
pub mod tracing;

#[derive(Default)]
pub struct TelemetryBatteries<T: TracingBattery, M: MetricsBattery> {
    pub tracing_battery: Option<T>,
    pub metrics_battery: Option<M>,
}

impl<T: TracingBattery, M: MetricsBattery> TelemetryBatteries<T, M> {
    pub fn new() -> Self {
        Self {
            tracing_battery: None,
            metrics_battery: None,
        }
    }

    pub fn tracing(&mut self, battery: T) {
        self.tracing_battery = Some(battery);
    }

    pub fn metrics(&mut self, battery: M) {
        self.metrics_battery = Some(battery);
    }

    pub fn init(self) {
        if let Some(tracing_battery) = &self.tracing_battery {
            tracing_battery.init();
        }

        if let Some(metrics_battery) = &self.metrics_battery {
            metrics_battery.init();
        }
    }
}

impl<T: TracingBattery, M: MetricsBattery> Drop for TelemetryBatteries<T, M> {
    fn drop(&mut self) {
        opentelemetry::global::shutdown_tracer_provider();
    }
}
