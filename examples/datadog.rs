use telemetry_batteries::metrics::statsd::StatsdBattery;
use telemetry_batteries::tracing::datadog::DatadogBattery;

pub const SERVICE_NAME: &str = "datadog-example";

pub fn main() -> eyre::Result<()> {
    // Add a new DatadogBattery for tracing/logs
    DatadogBattery::init(None, SERVICE_NAME, None, true);

    // Add a new StatsdBattery for metrics
    StatsdBattery::init("localhost", 8125, 5000, 1024, None)?;

    // Alternatively you can use a prometheus exporter
    // PrometheusBattery::init()?;

    tracing::info!("foo");
    metrics::counter!("foo").increment(1);

    // Tracing providers are shutdown at the end of the program when TRACING_PROVIDER_GUARD is dropped.
    Ok(())
}
