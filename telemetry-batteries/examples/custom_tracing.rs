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
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn main() -> eyre::Result<()> {
    // Initialize tracing using layers directly for custom composition
    let datadog_layer = datadog_layer("datadog-example", "http://localhost:8126", true);

    tracing_subscriber::registry()
        .with(stdout_layer())
        .with(datadog_layer)
        .init();

    tracing::info!("Hello from custom tracing!");

    Ok(())
}
