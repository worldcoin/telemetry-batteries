use telemetry_batteries::tracing::{
    batteries::TracingBattery, layers::datadog::DatadogFormatLayer,
};

pub fn main() -> eyre::Result<()> {
    let datadog_format_layer = DatadogFormatLayer::layer();
    TracingBattery::init(datadog_format_layer);

    tracing::info!("foo");

    Ok(())
}
