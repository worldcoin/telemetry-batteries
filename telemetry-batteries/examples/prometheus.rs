//! Example using the unified init API with Prometheus metrics.
//!
//! Run with environment variables:
//! ```bash
//! TELEMETRY_PRESET=local \
//! TELEMETRY_METRICS_BACKEND=prometheus \
//! TELEMETRY_PROMETHEUS_LISTEN=0.0.0.0:9998 \
//! cargo run --example prometheus
//! ```
//!
//! Or with programmatic configuration (shown below).

use telemetry_batteries::{
    MetricsBackend, MetricsConfig, PrometheusConfig, PrometheusMode, TelemetryConfig,
    TelemetryPreset,
};

pub fn main() -> Result<(), telemetry_batteries::InitError> {
    // Configure telemetry programmatically using presets
    let config = TelemetryConfig::builder()
        .preset(TelemetryPreset::Local) // pretty logs, no span export
        .metrics(
            MetricsConfig::builder()
                .backend(MetricsBackend::Prometheus)
                .prometheus(
                    PrometheusConfig::builder()
                        .mode(PrometheusMode::Http)
                        .listen("0.0.0.0:9998".parse().unwrap())
                        .build(),
                )
                .build(),
        )
        .build();

    let _guard = telemetry_batteries::init_with_config(config)?;

    metrics::counter!("foo").increment(1);

    // Keep the server running to allow scraping
    println!("Prometheus metrics available at http://0.0.0.0:9998/metrics");
    println!("Press Ctrl+C to exit");
    std::thread::park();

    Ok(())
}
