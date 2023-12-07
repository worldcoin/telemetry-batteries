use telemetry_batteries::tracing::{
    batteries::TracingBattery, layers::datadog::DatadogFormatLayer,
};

pub fn main() -> eyre::Result<()> {
    let datadog_format_layer = DatadogFormatLayer::layer(false);
    TracingBattery::init(datadog_format_layer);

    tracing::info!("foo");

    // Tracing providers are shutdown at the end of the program when TRACING_PROVIDER_GUARD is dropped.
    Ok(())
}
