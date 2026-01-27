//! Configuration types for telemetry initialization.

use std::{env, net::SocketAddr, time::Duration};

use bon::Builder;

use crate::error::InitError;

/// Log output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogFormat {
    /// Pretty-printed human-readable output.
    Pretty,
    /// JSON-formatted output (default).
    #[default]
    Json,
    /// Compact single-line output.
    Compact,
}

impl LogFormat {
    fn from_str(s: &str) -> Result<Self, InitError> {
        match s.to_lowercase().as_str() {
            "pretty" => Ok(Self::Pretty),
            "json" => Ok(Self::Json),
            "compact" => Ok(Self::Compact),
            _ => Err(InitError::InvalidConfig {
                field: "TELEMETRY_LOG_FORMAT",
                message: format!("expected 'pretty', 'json', or 'compact', got '{s}'"),
            }),
        }
    }
}

/// Tracing/logging backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TracingBackend {
    /// Output traces to stdout (default).
    #[default]
    Stdout,
    /// Send traces to Datadog Agent.
    Datadog,
    /// Disable tracing.
    None,
}

impl TracingBackend {
    fn from_str(s: &str) -> Result<Self, InitError> {
        match s.to_lowercase().as_str() {
            "stdout" => Ok(Self::Stdout),
            "datadog" => Ok(Self::Datadog),
            "none" => Ok(Self::None),
            _ => Err(InitError::InvalidConfig {
                field: "TELEMETRY_TRACING_BACKEND",
                message: format!("expected 'stdout', 'datadog', or 'none', got '{s}'"),
            }),
        }
    }
}

/// Metrics backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MetricsBackend {
    /// Prometheus metrics exporter.
    Prometheus,
    /// StatsD metrics exporter.
    Statsd,
    /// Disable metrics (default).
    #[default]
    None,
}

impl MetricsBackend {
    fn from_str(s: &str) -> Result<Self, InitError> {
        match s.to_lowercase().as_str() {
            "prometheus" => Ok(Self::Prometheus),
            "statsd" => Ok(Self::Statsd),
            "none" => Ok(Self::None),
            _ => Err(InitError::InvalidConfig {
                field: "TELEMETRY_METRICS_BACKEND",
                message: format!("expected 'prometheus', 'statsd', or 'none', got '{s}'"),
            }),
        }
    }
}

/// Prometheus export mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PrometheusMode {
    /// Run HTTP listener for scraping (default).
    #[default]
    Http,
    /// Push metrics to push gateway.
    Push,
}

impl PrometheusMode {
    fn from_str(s: &str) -> Result<Self, InitError> {
        match s.to_lowercase().as_str() {
            "http" => Ok(Self::Http),
            "push" => Ok(Self::Push),
            _ => Err(InitError::InvalidConfig {
                field: "TELEMETRY_PROMETHEUS_MODE",
                message: format!("expected 'http' or 'push', got '{s}'"),
            }),
        }
    }
}

/// Eyre error reporting mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EyreMode {
    /// Colored multi-line output (default).
    #[default]
    Color,
    /// JSON output.
    Json,
}

impl EyreMode {
    fn from_str(s: &str) -> Result<Self, InitError> {
        match s.to_lowercase().as_str() {
            "color" => Ok(Self::Color),
            "json" => Ok(Self::Json),
            _ => Err(InitError::InvalidConfig {
                field: "TELEMETRY_EYRE_MODE",
                message: format!("expected 'color' or 'json', got '{s}'"),
            }),
        }
    }
}

/// Tracing configuration.
#[derive(Debug, Clone, Builder, Default)]
pub struct TracingConfig {
    /// Tracing backend to use.
    #[builder(default)]
    pub backend: TracingBackend,

    /// Endpoint for the tracing backend (e.g., Datadog Agent URL).
    #[builder(default)]
    pub endpoint: Option<String>,

    /// Log output format.
    #[builder(default)]
    pub format: LogFormat,

    /// Include location information (file, line, module path) in traces.
    #[builder(default)]
    pub location: bool,
}

/// Prometheus-specific configuration.
#[derive(Debug, Clone, Builder, Default)]
pub struct PrometheusConfig {
    /// Export mode (http listener or push gateway).
    #[builder(default)]
    pub mode: PrometheusMode,

    /// Listen address for HTTP mode.
    #[builder(default = default_prometheus_listen())]
    pub listen: SocketAddr,

    /// Push gateway endpoint.
    #[builder(default)]
    pub endpoint: Option<String>,

    /// Push interval in seconds.
    #[builder(default = Duration::from_secs(10))]
    pub interval: Duration,
}

fn default_prometheus_listen() -> SocketAddr {
    "0.0.0.0:9090".parse().unwrap()
}

/// StatsD-specific configuration.
#[derive(Debug, Clone, Builder)]
pub struct StatsdConfig {
    /// StatsD server host.
    #[builder(default = "localhost".to_owned())]
    pub host: String,

    /// StatsD server port.
    #[builder(default = 8125)]
    pub port: u16,

    /// Metric name prefix.
    #[builder(default)]
    pub prefix: Option<String>,

    /// Queue size for the exporter.
    #[builder(default = 5000)]
    pub queue_size: usize,

    /// Buffer size for the exporter.
    #[builder(default = 1024)]
    pub buffer_size: usize,
}

impl Default for StatsdConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_owned(),
            port: 8125,
            prefix: None,
            queue_size: 5000,
            buffer_size: 1024,
        }
    }
}

/// Metrics configuration.
#[derive(Debug, Clone, Builder, Default)]
pub struct MetricsConfig {
    /// Metrics backend to use.
    #[builder(default)]
    pub backend: MetricsBackend,

    /// Prometheus-specific configuration.
    #[builder(default)]
    pub prometheus: PrometheusConfig,

    /// StatsD-specific configuration.
    #[builder(default)]
    pub statsd: StatsdConfig,
}

/// Eyre error reporting configuration.
#[derive(Debug, Clone, Copy, Builder, Default)]
pub struct EyreConfig {
    /// Error reporting mode.
    #[builder(default)]
    pub mode: EyreMode,

    /// Enable backtrace capture by default.
    #[builder(default = true)]
    pub with_default_backtrace: bool,

    /// Enable spantrace capture by default.
    #[builder(default = true)]
    pub with_default_spantrace: bool,
}

/// Main telemetry configuration.
#[derive(Debug, Clone, Builder, Default)]
pub struct TelemetryConfig {
    /// Service name (required for Datadog).
    #[builder(default)]
    pub service_name: Option<String>,

    /// Tracing/logging configuration.
    #[builder(default)]
    pub tracing: TracingConfig,

    /// Metrics configuration.
    #[builder(default)]
    pub metrics: MetricsConfig,

    /// Eyre error reporting configuration.
    #[builder(default)]
    pub eyre: EyreConfig,
}

impl TelemetryConfig {
    /// Load configuration from environment variables.
    ///
    /// # Environment Variables
    ///
    /// | Variable | Values | Default |
    /// |----------|--------|---------|
    /// | `TELEMETRY_SERVICE_NAME` | string | required for datadog |
    /// | `TELEMETRY_LOG_LEVEL` | trace/debug/info/warn/error | `info` (respects `RUST_LOG` first) |
    /// | `TELEMETRY_LOG_FORMAT` | pretty/json/compact | `json` |
    /// | `TELEMETRY_TRACING_BACKEND` | stdout/datadog/none | `stdout` |
    /// | `TELEMETRY_TRACING_ENDPOINT` | url | `http://localhost:8126` |
    /// | `TELEMETRY_TRACING_LOCATION` | true/false | `false` |
    /// | `TELEMETRY_EYRE_MODE` | color/json | `color` |
    /// | `TELEMETRY_METRICS_BACKEND` | prometheus/statsd/none | `none` |
    /// | `TELEMETRY_PROMETHEUS_MODE` | http/push | `http` |
    /// | `TELEMETRY_PROMETHEUS_LISTEN` | addr:port | `0.0.0.0:9090` |
    /// | `TELEMETRY_PROMETHEUS_ENDPOINT` | url | - |
    /// | `TELEMETRY_PROMETHEUS_INTERVAL` | seconds | `10` |
    /// | `TELEMETRY_STATSD_HOST` | string | `localhost` |
    /// | `TELEMETRY_STATSD_PORT` | u16 | `8125` |
    /// | `TELEMETRY_STATSD_PREFIX` | string | - |
    pub fn from_env() -> Result<Self, InitError> {
        let service_name = env::var("TELEMETRY_SERVICE_NAME").ok();

        let tracing = TracingConfig {
            backend: env::var("TELEMETRY_TRACING_BACKEND")
                .ok()
                .map(|s| TracingBackend::from_str(&s))
                .transpose()?
                .unwrap_or_default(),
            endpoint: env::var("TELEMETRY_TRACING_ENDPOINT").ok(),
            format: env::var("TELEMETRY_LOG_FORMAT")
                .ok()
                .map(|s| LogFormat::from_str(&s))
                .transpose()?
                .unwrap_or_default(),
            location: env::var("TELEMETRY_TRACING_LOCATION")
                .ok()
                .map(|s| parse_bool(&s, "TELEMETRY_TRACING_LOCATION"))
                .transpose()?
                .unwrap_or(false),
        };

        let prometheus = PrometheusConfig {
            mode: env::var("TELEMETRY_PROMETHEUS_MODE")
                .ok()
                .map(|s| PrometheusMode::from_str(&s))
                .transpose()?
                .unwrap_or_default(),
            listen: env::var("TELEMETRY_PROMETHEUS_LISTEN")
                .ok()
                .map(|s| {
                    s.parse().map_err(|_| InitError::InvalidConfig {
                        field: "TELEMETRY_PROMETHEUS_LISTEN",
                        message: format!("invalid socket address: {s}"),
                    })
                })
                .transpose()?
                .unwrap_or_else(default_prometheus_listen),
            endpoint: env::var("TELEMETRY_PROMETHEUS_ENDPOINT").ok(),
            interval: env::var("TELEMETRY_PROMETHEUS_INTERVAL")
                .ok()
                .map(|s| {
                    s.parse::<u64>()
                        .map(Duration::from_secs)
                        .map_err(|_| InitError::InvalidConfig {
                            field: "TELEMETRY_PROMETHEUS_INTERVAL",
                            message: format!("expected integer seconds, got '{s}'"),
                        })
                })
                .transpose()?
                .unwrap_or(Duration::from_secs(10)),
        };

        let statsd = StatsdConfig {
            host: env::var("TELEMETRY_STATSD_HOST").unwrap_or_else(|_| "localhost".to_owned()),
            port: env::var("TELEMETRY_STATSD_PORT")
                .ok()
                .map(|s| {
                    s.parse().map_err(|_| InitError::InvalidConfig {
                        field: "TELEMETRY_STATSD_PORT",
                        message: format!("expected u16 port number, got '{s}'"),
                    })
                })
                .transpose()?
                .unwrap_or(8125),
            prefix: env::var("TELEMETRY_STATSD_PREFIX").ok(),
            queue_size: 5000,
            buffer_size: 1024,
        };

        let metrics = MetricsConfig {
            backend: env::var("TELEMETRY_METRICS_BACKEND")
                .ok()
                .map(|s| MetricsBackend::from_str(&s))
                .transpose()?
                .unwrap_or_default(),
            prometheus,
            statsd,
        };

        let eyre = EyreConfig {
            mode: env::var("TELEMETRY_EYRE_MODE")
                .ok()
                .map(|s| EyreMode::from_str(&s))
                .transpose()?
                .unwrap_or_default(),
            with_default_backtrace: true,
            with_default_spantrace: true,
        };

        Ok(Self {
            service_name,
            tracing,
            metrics,
            eyre,
        })
    }
}

fn parse_bool(s: &str, field: &'static str) -> Result<bool, InitError> {
    match s.to_lowercase().as_str() {
        "true" | "1" | "yes" => Ok(true),
        "false" | "0" | "no" => Ok(false),
        _ => Err(InitError::InvalidConfig {
            field,
            message: format!("expected 'true' or 'false', got '{s}'"),
        }),
    }
}
