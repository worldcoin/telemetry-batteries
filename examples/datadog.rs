use ::tracing::Level;
use telemetry_batteries::{
    metrics_batteries::statsd::StatsdBattery, tracing_batteries::datadog::DatadogBattery,
    TelemetryBatteries,
};
use tracing_appender::rolling::Rotation;

pub const SERVICE_NAME: &str = "datadog-example";

pub fn main() -> eyre::Result<()> {
    let batteries = TelemetryBatteries::new();

    // Add a new DatadogBattery for tracing/logs
    let datadog_battery = DatadogBattery::new(Level::INFO, SERVICE_NAME, Rotation::DAILY, None);

    // Add a new StatsdBattery for metrics
    let statsd_battery = StatsdBattery::new("localhost", 8125, 5000, 1024, None)?;

    // Add the batteries and initialize
    batteries
        .tracing(datadog_battery)
        .metrics(statsd_battery)
        .init()?;

    tracing::info!("foo");
    metrics::increment_counter!("bar");

    // Tracing providers are shutdown at the end of the program when TRACING_PROVIDER_GUARD is dropped.
    Ok(())
}
