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
//! Configuration is loaded from environment variables by default:
//!
//! | Variable | Values | Default |
//! |----------|--------|---------|
//! | `TELEMETRY_SERVICE_NAME` | string | required for datadog |
//! | `TELEMETRY_LOG_FORMAT` | pretty/json/compact | `json` |
//! | `TELEMETRY_TRACING_BACKEND` | stdout/datadog/none | `stdout` |
//! | `TELEMETRY_TRACING_ENDPOINT` | url | `http://localhost:8126` |
//! | `TELEMETRY_TRACING_LOCATION` | true/false | `false` |
//! | `TELEMETRY_EYRE_MODE` | color/json | `color` |
//! | `TELEMETRY_METRICS_BACKEND` | prometheus/statsd/none | `none` |
//! | `TELEMETRY_PROMETHEUS_MODE` | http/push | `http` |
//! | `TELEMETRY_PROMETHEUS_LISTEN` | addr:port | `0.0.0.0:9090` |
//! | `TELEMETRY_PROMETHEUS_ENDPOINT` | url | - |
//! | `TELEMETRY_PROMETHEUS_INTERVAL` | seconds | `10` |
//! | `TELEMETRY_STATSD_HOST` | string | `localhost` |
//! | `TELEMETRY_STATSD_PORT` | u16 | `8125` |
//! | `TELEMETRY_STATSD_PREFIX` | string | - |
//!
//! # Builder Pattern
//!
//! For programmatic configuration, use the builder pattern:
//!
//! ```ignore
//! use telemetry_batteries::{TelemetryConfig, TracingConfig, TracingBackend};
//!
//! let config = TelemetryConfig::builder()
//!     .service_name("my-service".to_owned())
//!     .tracing(TracingConfig::builder()
//!         .backend(TracingBackend::Datadog)
//!         .build())
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
    PrometheusMode, StatsdConfig, TelemetryConfig, TracingBackend, TracingConfig,
};
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
/// - Required configuration is missing (e.g., `service_name` for Datadog)
/// - A requested feature is not compiled in
/// - Backend initialization fails
///
/// # Example
///
/// ```ignore
/// use telemetry_batteries::{TelemetryConfig, TracingConfig, TracingBackend};
///
/// let config = TelemetryConfig::builder()
///     .service_name("my-service".to_owned())
///     .tracing(TracingConfig::builder()
///         .backend(TracingBackend::Datadog)
///         .build())
///     .build();
///
/// let _guard = telemetry_batteries::init_with_config(config)?;
/// ```
pub fn init_with_config(config: TelemetryConfig) -> Result<TelemetryGuard, InitError> {
    // Initialize eyre error reporting first
    eyre::init(&config.eyre)?;

    // Initialize tracing based on backend
    let tracing_handle = match config.tracing.backend {
        TracingBackend::Stdout => {
            Some(tracing::stdout::init(config.tracing.format))
        }
        TracingBackend::Datadog => {
            let service_name = config
                .service_name
                .as_deref()
                .ok_or(InitError::MissingConfig("TELEMETRY_SERVICE_NAME (required for Datadog backend)"))?;

            Some(tracing::datadog::init(
                config.tracing.endpoint.as_deref(),
                service_name,
                config.tracing.location,
            ))
        }
        TracingBackend::None => None,
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
