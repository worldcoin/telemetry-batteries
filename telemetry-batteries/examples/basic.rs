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

pub fn main() -> Result<(), telemetry_batteries::InitError> {
    // Initialize telemetry from environment variables
    let _guard = telemetry_batteries::init()?;

    tracing::info!("Hello from telemetry-batteries!");
    tracing::warn!(answer = 42, "The answer is {}", 42);

    Ok(())
}
