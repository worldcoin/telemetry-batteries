pub mod error;
pub mod metrics_batteries;
pub mod tracing_batteries;

use error::BatteryError;
use metrics_batteries::MetricsBattery;
use tracing_batteries::TracingBattery;

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

    pub fn tracing(mut self, battery: T) -> Self {
        self.tracing_battery = Some(battery);
        self
    }

    pub fn metrics(mut self, battery: M) -> Self {
        self.metrics_battery = Some(battery);
        self
    }

    pub fn init(self) -> Result<(), BatteryError> {
        if let Some(tracing_battery) = &self.tracing_battery {
            tracing_battery.init()?;
        }

        if let Some(metrics_battery) = &self.metrics_battery {
            metrics_battery.init()?;
        }

        Ok(())
    }
}
