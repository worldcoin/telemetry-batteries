use crate::error::BatteryError;

pub mod statsd;

pub trait MetricsBattery {
    fn init(&self) -> Result<(), BatteryError>;
}
