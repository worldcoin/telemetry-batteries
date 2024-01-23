use telemetry_batteries::tracing::layers::{
    datadog::datadog_layer, stdout_layer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn main() -> eyre::Result<()> {
    // Initialize tracing using layers
    let datadog_layer =
        datadog_layer("datadog-example", "http://localhost:8126", true);

    let stdout_layer = stdout_layer();

    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(datadog_layer)
        .init();

    Ok(())
}
