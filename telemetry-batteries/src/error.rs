//! Error types for telemetry initialization.

use thiserror::Error;

/// Error type for telemetry initialization failures.
#[derive(Debug, Error)]
pub enum InitError {
    /// A required configuration value was not provided.
    #[error("missing required config: {0}")]
    MissingConfig(&'static str),

    /// A configuration value was invalid.
    #[error("invalid config for {field}: {message}")]
    InvalidConfig {
        field: &'static str,
        message: String,
    },

    /// A feature was requested but not compiled in.
    #[error("feature '{0}' was requested but not compiled in")]
    FeatureNotCompiled(&'static str),

    /// Failed to initialize eyre error reporting.
    #[error("failed to initialize eyre: {0}")]
    Eyre(#[from] eyre::InstallError),

    /// Failed to initialize Prometheus metrics.
    #[cfg(feature = "metrics-prometheus")]
    #[error("failed to initialize prometheus: {0}")]
    Prometheus(#[from] metrics_exporter_prometheus::BuildError),

    /// Failed to initialize StatsD metrics.
    #[cfg(feature = "metrics-statsd")]
    #[error("failed to initialize statsd: {0}")]
    Statsd(#[from] metrics_exporter_statsd::StatsdError),

    /// Failed to set global metrics recorder.
    #[error("failed to set global metrics recorder: {0}")]
    MetricsRecorder(#[from] metrics::SetRecorderError),
}
