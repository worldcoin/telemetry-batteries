use ::tracing::Level;
use telemetry_batteries::{
    metrics::statsd::StatsdBattery, tracing::datadog::DatadogBattery, TelemetryBatteries,
};
use tracing_appender::rolling::Rotation;

pub const SERVICE_NAME: &str = "datadog-example";

pub fn main() -> eyre::Result<()> {
    let mut batteries = TelemetryBatteries::new();

    // Add a new DatadogBattery for tracing/logs
    let datadog_battery = DatadogBattery::new(Level::INFO, SERVICE_NAME, Rotation::DAILY, None);
    batteries.tracing(datadog_battery);

    // Add a new StatsdBattery for metrics
    let statsd_battery = StatsdBattery::new("localhost", 8125, 5000, 1024, None)?;
    batteries.metrics(statsd_battery);

    // Initialize all batteries
    batteries.init()?;

    // Tracing providers are shutdown at the end of the program when TRACING_PROVIDER_GUARD is dropped.
    Ok(())
}
