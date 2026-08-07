//! Basic example using the unified init API with environment variables.
//!
//! This is the simplest way to initialize telemetry - all configuration
//! comes from environment variables.
//!
//! Run with default settings (pretty local logs):
//! ```bash
//! cargo run --example basic
//! ```
//!
//! Run with Datadog:
//! ```bash
//! DD_ENABLED=true \
//! DD_SERVICE=my-service \
//! cargo run --example basic
//! ```
//!
//! Run with Datadog but pretty logs for debugging:
//! ```bash
//! DD_ENABLED=true \
//! DD_SERVICE=my-service \
//! LOG_FORMAT=pretty \
//! cargo run --example basic
//! ```
//!
//! Note: Datadog tracing requires a Tokio runtime.

use std::time::Duration;

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    // Initialize telemetry from environment variables
    let _guard = telemetry_batteries::init()?;

    tracing::info!("Hello from telemetry-batteries!");

    loop {
        metrics::counter!("example.count").increment(1);
        contained_span().await;

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

#[tracing::instrument]
async fn contained_span() {
    tracing::info!("Inside a contained span");
}
