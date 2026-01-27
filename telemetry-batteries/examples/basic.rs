//! Basic example using the unified init API with environment variables.
//!
//! This is the simplest way to initialize telemetry - all configuration
//! comes from environment variables.
//!
//! Run with default settings (stdout tracing):
//! ```bash
//! cargo run --example basic
//! ```
//!
//! Run with Datadog:
//! ```bash
//! TELEMETRY_SERVICE_NAME=my-service \
//! TELEMETRY_TRACING_BACKEND=datadog \
//! cargo run --example basic
//! ```
//!
//! Note: Datadog backend requires a Tokio runtime.

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    // Initialize telemetry from environment variables
    let _guard = telemetry_batteries::init()?;

    tracing::info!("Hello from telemetry-batteries!");
    tracing::warn!(answer = 42, "The answer is {}", 42);

    inner()?;

    Ok(())
}

#[tracing::instrument]
fn inner() -> eyre::Result<()> {
    tracing::info!("Inside an instrumented function!");

    inner_inner()?;

    Ok(())
}

fn inner_inner() -> eyre::Result<()> {
    eyre::bail!("Deep fail");
}
