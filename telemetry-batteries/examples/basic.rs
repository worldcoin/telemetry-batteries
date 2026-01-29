//! Basic example using the unified init API with environment variables.
//!
//! This is the simplest way to initialize telemetry - all configuration
//! comes from environment variables.
//!
//! Run with default settings (local preset - pretty logs):
//! ```bash
//! cargo run --example basic
//! ```
//!
//! Run with Datadog:
//! ```bash
//! TELEMETRY_PRESET=datadog \
//! TELEMETRY_SERVICE_NAME=my-service \
//! cargo run --example basic
//! ```
//!
//! Run with Datadog but pretty logs for debugging:
//! ```bash
//! TELEMETRY_PRESET=datadog \
//! TELEMETRY_SERVICE_NAME=my-service \
//! TELEMETRY_LOG_FORMAT=pretty \
//! cargo run --example basic
//! ```
//!
//! Note: Datadog preset requires a Tokio runtime.

use std::time::Duration;

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    // Initialize telemetry from environment variables
    let _guard = telemetry_batteries::init()?;

    tracing::info!("Hello from telemetry-batteries!");
    tracing::warn!(answer = 42, "The answer is {}", 42);

    tracing::info!("Press Ctrl+C to exit...");

    tokio::select! {
        _ = inner() => {}
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Shutting down");
        }
    }

    Ok(())
}

#[tracing::instrument]
async fn inner() {
    loop {
        contained_span().await;

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

#[tracing::instrument]
async fn contained_span() {
    tracing::info!("Inside a contained span");
}
