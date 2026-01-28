//! Example of custom tracing layer composition.
//!
//! For advanced use cases where you need fine-grained control over
//! tracing layers, you can compose them manually.
//!
//! Run with:
//! ```bash
//! RUST_LOG=info cargo run --example custom_tracing
//! ```

use telemetry_batteries::tracing::layers::{datadog::datadog_layer, stdout_layer};
use telemetry_batteries::LogFormat;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    // Initialize tracing using layers directly for custom composition
    // datadog_layer returns (layer, provider) - keep the provider alive for proper shutdown
    let (dd_layer, _provider) =
        datadog_layer("datadog-example", "http://localhost:8126", LogFormat::DatadogJson);

    tracing_subscriber::registry()
        .with(stdout_layer())
        .with(dd_layer)
        .init();

    tracing::info!("Hello from custom tracing!");

    Ok(())
}
