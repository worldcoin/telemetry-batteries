use ::tracing::Level;
use telemetry_batteries::{metrics, tracing, TelemetryBatteries};

pub const SERVICE_NAME: &str = "datadog-example";

pub fn main() -> eyre::Result<()> {
    let mut batteries = TelemetryBatteries::new();

    // Add a new DatadogBattery for tracing/logs
    let datadog_battery = tracing::datadog::DatadogBattery::new(Level::INFO, SERVICE_NAME);
    batteries.add_battery(datadog_battery);

    // Add a new StatsdBattery for metrics
    let statsd_battery = metrics::statsd::StatsdBattery::new("localhost", 8125, 5000, 1024, None)
        .expect("Failed to create StatsdBattery");
    batteries.add_battery(statsd_battery);

    // Initialize all batteries
    batteries.init();

    // Once the batteries variable is dropped out of scope, all tracing providers will be shutdown
    Ok(())
}
