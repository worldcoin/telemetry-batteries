//! Example using the unified init API with Datadog preset.
//!
//! Run with environment variables:
//! ```bash
//! TELEMETRY_PRESET=datadog \
//! TELEMETRY_SERVICE_NAME=datadog-example \
//! cargo run --example datadog
//! ```
//!
//! Or with programmatic configuration (shown below).

use telemetry_batteries::{
    MetricsBackend, MetricsConfig, StatsdConfig, TelemetryConfig, TelemetryPreset,
};

pub fn main() -> Result<(), telemetry_batteries::InitError> {
    // Configure telemetry programmatically using presets
    let config = TelemetryConfig::builder()
        .preset(TelemetryPreset::Datadog)
        .service_name("datadog-example".to_owned())
        // Optional: override log format (default for Datadog is DatadogJson)
        // .log_format(LogFormat::Pretty)
        .metrics(
            MetricsConfig::builder()
                .backend(MetricsBackend::Statsd)
                .statsd(
                    StatsdConfig::builder()
                        .host("localhost".to_owned())
                        .port(8125)
                        .build(),
                )
                .build(),
        )
        .build();

    let _guard = telemetry_batteries::init_with_config(config)?;

    tracing::info!("Hello from Datadog example!");
    metrics::counter!("foo").increment(1);

    Ok(())
}
