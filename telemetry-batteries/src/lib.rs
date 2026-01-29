#![doc = include_str!("../../README.md")]

pub mod config;
pub mod eyre;
mod guard;
#[cfg(any(feature = "metrics-prometheus", feature = "metrics-statsd"))]
mod metrics;
pub mod tracing;

pub use config::{
    EyreConfig, EyreMode, LogFormat, MetricsBackend, MetricsConfig, PrometheusConfig,
    PrometheusMode, StatsdConfig, TelemetryConfig, TelemetryPreset,
};
pub use guard::TelemetryGuard;

/// Reexports of crates that appear in the public API.
///
/// Using these directly instead of adding them yourself to Cargo.toml will help avoid
/// errors where types have the same name but actually are distinct types from different
/// crate versions.
pub mod reexports {
    #[cfg(any(feature = "metrics-prometheus", feature = "metrics-statsd"))]
    pub use ::metrics;
    pub use ::opentelemetry;
}

/// Initialize telemetry from environment variables.
///
/// This is the main entry point for most applications. Configuration is loaded
/// from `TELEMETRY_*` environment variables.
///
/// Returns a [`TelemetryGuard`] that must be kept alive for the duration of
/// the application. When dropped, it gracefully shuts down the tracing provider.
///
/// # Errors
///
/// Returns an error if:
/// - Required configuration is missing (e.g., `TELEMETRY_SERVICE_NAME` for Datadog)
/// - Configuration values are invalid
/// - A requested feature is not compiled in
/// - Backend initialization fails
///
/// # Example
///
/// ```ignore
/// fn main() -> eyre::Result<()> {
///     let _guard = telemetry_batteries::init()?;
///
///     tracing::info!("Hello, telemetry!");
///
///     Ok(())
/// }
/// ```
pub fn init() -> ::eyre::Result<TelemetryGuard> {
    let config = TelemetryConfig::from_env()?;
    init_with_config(config)
}

/// Initialize telemetry with the given configuration.
///
/// Use this when you need programmatic control over the configuration.
/// For most use cases, prefer [`init()`] which loads from environment variables.
///
/// Returns a [`TelemetryGuard`] that must be kept alive for the duration of
/// the application. When dropped, it gracefully shuts down the tracing provider.
///
/// # Errors
///
/// Returns an error if:
/// - Required configuration is missing (e.g., `service_name` for Datadog/Otel)
/// - A requested feature is not compiled in
/// - Backend initialization fails
pub fn init_with_config(config: TelemetryConfig) -> ::eyre::Result<TelemetryGuard> {
    use ::eyre::bail;

    // Initialize eyre error reporting first
    eyre::init(&config.eyre)?;

    let log_format = config.effective_log_format();
    let log_level = TelemetryConfig::log_level_from_env();

    // Initialize tracing based on preset
    let tracing_handle = match config.preset {
        TelemetryPreset::Local => {
            Some(tracing::stdout::init(log_format, &log_level))
        }
        TelemetryPreset::Datadog => {
            let service_name = config
                .service_name
                .as_deref()
                .ok_or_else(|| ::eyre::eyre!("TELEMETRY_SERVICE_NAME is required for Datadog preset"))?;

            Some(tracing::datadog::init(
                config.datadog_endpoint.as_deref(),
                service_name,
                log_format,
                &log_level,
            ))
        }
        TelemetryPreset::Otel => {
            bail!("otel preset is not yet implemented");
        }
        TelemetryPreset::None => None,
    };

    // Initialize metrics based on backend
    init_metrics(&config.metrics)?;

    Ok(TelemetryGuard::new(tracing_handle))
}

fn init_metrics(config: &MetricsConfig) -> ::eyre::Result<()> {
    match config.backend {
        MetricsBackend::Prometheus => {
            #[cfg(feature = "metrics-prometheus")]
            {
                metrics::prometheus::init(&config.prometheus)?;
            }
            #[cfg(not(feature = "metrics-prometheus"))]
            {
                ::eyre::bail!("metrics-prometheus feature not compiled in");
            }
        }
        MetricsBackend::Statsd => {
            #[cfg(feature = "metrics-statsd")]
            {
                metrics::statsd::init(&config.statsd)?;
            }
            #[cfg(not(feature = "metrics-statsd"))]
            {
                ::eyre::bail!("metrics-statsd feature not compiled in");
            }
        }
        MetricsBackend::None => {
            // No metrics initialization
        }
    }

    Ok(())
}
