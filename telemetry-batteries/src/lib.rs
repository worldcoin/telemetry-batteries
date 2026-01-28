//! Batteries included telemetry for Rust applications.
//!
//! This crate provides a unified initialization API for tracing, metrics, and error reporting.
//!
//! # Quick Start
//!
//! ```ignore
//! fn main() -> Result<(), telemetry_batteries::InitError> {
//!     let _guard = telemetry_batteries::init()?;
//!
//!     tracing::info!("Hello, telemetry!");
//!
//!     Ok(())
//! }
//! ```
//!
//! # Configuration
//!
//! Configuration is loaded from environment variables using **presets**:
//!
//! ## Presets
//!
//! | Preset | Log Format | Log Output | Span Export | Use Case |
//! |--------|------------|------------|-------------|----------|
//! | `local` | pretty | stdout | none | Local development |
//! | `datadog` | datadog_json | stdout | Datadog Agent | Production with Datadog |
//! | `otel` | json | stdout | OTLP | Production with OTel collector |
//! | `none` | - | none | none | Disable telemetry |
//!
//! ## Environment Variables
//!
//! | Variable | Values | Default |
//! |----------|--------|---------|
//! | `TELEMETRY_PRESET` | local/datadog/otel/none | `local` |
//! | `TELEMETRY_SERVICE_NAME` | string | required for datadog/otel |
//! | `RUST_LOG` or `TELEMETRY_LOG_LEVEL` | EnvFilter syntax | `info` |
//! | `TELEMETRY_LOG_FORMAT` | pretty/json/compact/datadog_json | (from preset) |
//! | `TELEMETRY_DATADOG_ENDPOINT` | url | `http://localhost:8126` |
//! | `TELEMETRY_OTLP_ENDPOINT` | url | `http://localhost:4317` |
//! | `TELEMETRY_METRICS_BACKEND` | prometheus/statsd/none | `none` |
//! | `TELEMETRY_PROMETHEUS_MODE` | http/push | `http` |
//! | `TELEMETRY_PROMETHEUS_LISTEN` | addr:port | `0.0.0.0:9090` |
//! | `TELEMETRY_PROMETHEUS_ENDPOINT` | url | - |
//! | `TELEMETRY_PROMETHEUS_INTERVAL` | seconds | `10` |
//! | `TELEMETRY_STATSD_HOST` | string | `localhost` |
//! | `TELEMETRY_STATSD_PORT` | u16 | `8125` |
//! | `TELEMETRY_STATSD_PREFIX` | string | - |
//!
//! # Examples
//!
//! ```bash
//! # Local development - pretty logs, no tracing
//! TELEMETRY_PRESET=local cargo run
//!
//! # Datadog production
//! TELEMETRY_PRESET=datadog TELEMETRY_SERVICE_NAME=my-service cargo run
//!
//! # Datadog but with pretty logs for debugging
//! TELEMETRY_PRESET=datadog TELEMETRY_SERVICE_NAME=my-service TELEMETRY_LOG_FORMAT=pretty cargo run
//! ```
//!
//! # Builder Pattern
//!
//! For programmatic configuration, use the builder pattern:
//!
//! ```ignore
//! use telemetry_batteries::{TelemetryConfig, TelemetryPreset};
//!
//! let config = TelemetryConfig::builder()
//!     .preset(TelemetryPreset::Datadog)
//!     .service_name("my-service".to_owned())
//!     .build();
//!
//! let _guard = telemetry_batteries::init_with_config(config)?;
//! ```

pub mod config;
pub mod error;
pub mod eyre;
mod guard;
#[cfg(any(feature = "metrics-prometheus", feature = "metrics-statsd"))]
mod metrics;
pub mod tracing;

pub use config::{
    EyreConfig, EyreMode, LogFormat, MetricsBackend, MetricsConfig, PrometheusConfig,
    PrometheusMode, StatsdConfig, TelemetryConfig, TelemetryPreset,
};

// Re-export deprecated types for backward compatibility
#[allow(deprecated)]
pub use config::{TracingBackend, TracingConfig};
pub use error::InitError;
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
/// fn main() -> Result<(), telemetry_batteries::InitError> {
///     let _guard = telemetry_batteries::init()?;
///
///     tracing::info!("Hello, telemetry!");
///
///     Ok(())
/// }
/// ```
pub fn init() -> Result<TelemetryGuard, InitError> {
    let config = TelemetryConfig::from_env()?;
    init_with_config(config)
}

/// Initialize telemetry with the given configuration.
///
/// Use this when you need programmatic control over the configuration
/// instead of loading from environment variables.
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
///
/// # Example
///
/// ```ignore
/// use telemetry_batteries::{TelemetryConfig, TelemetryPreset};
///
/// let config = TelemetryConfig::builder()
///     .preset(TelemetryPreset::Datadog)
///     .service_name("my-service".to_owned())
///     .build();
///
/// let _guard = telemetry_batteries::init_with_config(config)?;
/// ```
pub fn init_with_config(config: TelemetryConfig) -> Result<TelemetryGuard, InitError> {
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
                .ok_or(InitError::MissingConfig("TELEMETRY_SERVICE_NAME (required for Datadog preset)"))?;

            Some(tracing::datadog::init(
                config.datadog_endpoint.as_deref(),
                service_name,
                log_format,
                &log_level,
            ))
        }
        TelemetryPreset::Otel => {
            return Err(InitError::FeatureNotCompiled(
                "otel preset is not yet implemented",
            ));
        }
        TelemetryPreset::None => None,
    };

    // Initialize metrics based on backend
    init_metrics(&config.metrics)?;

    Ok(TelemetryGuard::new(tracing_handle))
}

fn init_metrics(config: &MetricsConfig) -> Result<(), InitError> {
    match config.backend {
        MetricsBackend::Prometheus => {
            #[cfg(feature = "metrics-prometheus")]
            {
                metrics::prometheus::init(&config.prometheus)?;
            }
            #[cfg(not(feature = "metrics-prometheus"))]
            {
                return Err(InitError::FeatureNotCompiled("metrics-prometheus"));
            }
        }
        MetricsBackend::Statsd => {
            #[cfg(feature = "metrics-statsd")]
            {
                metrics::statsd::init(&config.statsd)?;
            }
            #[cfg(not(feature = "metrics-statsd"))]
            {
                return Err(InitError::FeatureNotCompiled("metrics-statsd"));
            }
        }
        MetricsBackend::None => {
            // No metrics initialization
        }
    }

    Ok(())
}
