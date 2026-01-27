//! Example using the unified init API with Datadog backend.
//!
//! Run with:
//! ```bash
//! TELEMETRY_SERVICE_NAME=datadog-example \
//! TELEMETRY_TRACING_BACKEND=datadog \
//! TELEMETRY_TRACING_LOCATION=true \
//! cargo run --example datadog
//! ```

use telemetry_batteries::{
    MetricsBackend, MetricsConfig, StatsdConfig, TelemetryConfig, TracingBackend, TracingConfig,
};

pub fn main() -> Result<(), telemetry_batteries::InitError> {
    // Configure telemetry programmatically
    let config = TelemetryConfig::builder()
        .service_name("datadog-example".to_owned())
        .tracing(
            TracingConfig::builder()
                .backend(TracingBackend::Datadog)
                .location(true)
                .build(),
        )
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

    tracing::info!("foo");
    metrics::counter!("foo").increment(1);

    Ok(())
}
