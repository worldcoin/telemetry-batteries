use ::tracing::Level;
use telemetry_batteries::{
    metrics::{statsd::StatsdBattery, MetricsBattery},
    tracing::batteries::datadog::DatadogBattery,
};

pub const SERVICE_NAME: &str = "datadog-example";

pub fn main() -> eyre::Result<()> {
    // Add a new DatadogBattery for tracing/logs
    let datadog_battery = DatadogBattery::new(None, Level::INFO, SERVICE_NAME, None);
    datadog_battery.init()?;

    // Add a new StatsdBattery for metrics
    let statsd_battery = StatsdBattery::new("localhost", 8125, 5000, 1024, None)?;
    statsd_battery.init()?;

    tracing::info!("foo");
    metrics::increment_counter!("bar");

    // Tracing providers are shutdown at the end of the program when TRACING_PROVIDER_GUARD is dropped.
    Ok(())
}
