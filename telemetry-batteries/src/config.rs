//! Configuration types for telemetry initialization.

use std::{env, net::SocketAddr, time::Duration};

use bon::Builder;
use eyre::{bail, eyre};

/// Telemetry preset for common configurations.
///
/// Presets configure sensible defaults for logging and span export.
/// Individual settings can be overridden.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TelemetryPreset {
    /// Local development: pretty stdout logs, no span export.
    #[default]
    Local,
    /// Datadog: JSON logs with dd.trace_id/dd.span_id, spans to DD Agent.
    Datadog,
    /// OpenTelemetry: JSON logs, spans to OTLP collector (not yet implemented).
    Otel,
    /// Disable all telemetry output.
    None,
}

impl TelemetryPreset {
    fn from_str(s: &str) -> eyre::Result<Self> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "datadog" => Ok(Self::Datadog),
            "otel" | "otlp" | "opentelemetry" => Ok(Self::Otel),
            "none" => Ok(Self::None),
            _ => bail!(
                "invalid TELEMETRY_PRESET: expected 'local', 'datadog', 'otel', or 'none', got '{s}'"
            ),
        }
    }
}

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
    /// JSON with dd.trace_id/dd.span_id for Datadog log correlation.
    DatadogJson,
}

impl LogFormat {
    fn from_str(s: &str) -> eyre::Result<Self> {
        match s.to_lowercase().as_str() {
            "pretty" => Ok(Self::Pretty),
            "json" => Ok(Self::Json),
            "compact" => Ok(Self::Compact),
            "datadog" | "datadog_json" | "datadogjson" => Ok(Self::DatadogJson),
            _ => bail!(
                "invalid TELEMETRY_LOG_FORMAT: expected 'pretty', 'json', 'compact', or 'datadog_json', got '{s}'"
            ),
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
    fn from_str(s: &str) -> eyre::Result<Self> {
        match s.to_lowercase().as_str() {
            "prometheus" => Ok(Self::Prometheus),
            "statsd" => Ok(Self::Statsd),
            "none" => Ok(Self::None),
            _ => bail!(
                "invalid TELEMETRY_METRICS_BACKEND: expected 'prometheus', 'statsd', or 'none', got '{s}'"
            ),
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
    fn from_str(s: &str) -> eyre::Result<Self> {
        match s.to_lowercase().as_str() {
            "http" => Ok(Self::Http),
            "push" => Ok(Self::Push),
            _ => bail!(
                "invalid TELEMETRY_PROMETHEUS_MODE: expected 'http' or 'push', got '{s}'"
            ),
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
    fn from_str(s: &str) -> eyre::Result<Self> {
        match s.to_lowercase().as_str() {
            "color" => Ok(Self::Color),
            "json" => Ok(Self::Json),
            _ => bail!(
                "invalid TELEMETRY_EYRE_MODE: expected 'color' or 'json', got '{s}'"
            ),
        }
    }
}

/// Prometheus-specific configuration.
#[derive(Debug, Clone, Builder)]
pub struct PrometheusConfig {
    /// Export mode (http listener or push gateway).
    #[builder(default)]
    pub mode: PrometheusMode,

    /// Listen address for HTTP mode.
    #[builder(default = default_prometheus_listen())]
    pub listen: SocketAddr,

    /// Push gateway endpoint.
    pub endpoint: Option<String>,

    /// Push interval in seconds.
    #[builder(default = Duration::from_secs(10))]
    pub interval: Duration,
}

impl Default for PrometheusConfig {
    fn default() -> Self {
        Self {
            mode: PrometheusMode::default(),
            listen: default_prometheus_listen(),
            endpoint: None,
            interval: Duration::from_secs(10),
        }
    }
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
    /// Telemetry preset (sets sensible defaults for logging + span export).
    #[builder(default)]
    pub preset: TelemetryPreset,

    /// Service name (required for datadog/otel presets).
    pub service_name: Option<String>,

    /// Override log format from preset.
    pub log_format: Option<LogFormat>,

    /// Datadog Agent endpoint (for datadog preset).
    /// Defaults to http://localhost:8126.
    pub datadog_endpoint: Option<String>,

    /// OTLP collector endpoint (for otel preset).
    pub otlp_endpoint: Option<String>,

    /// Metrics configuration (independent from preset).
    #[builder(default)]
    pub metrics: MetricsConfig,

    /// Eyre error reporting configuration.
    #[builder(default)]
    pub eyre: EyreConfig,
}

impl TelemetryConfig {
    /// Get the effective log format based on preset and override.
    pub fn effective_log_format(&self) -> LogFormat {
        self.log_format.unwrap_or_else(|| match self.preset {
            TelemetryPreset::Local => LogFormat::Pretty,
            TelemetryPreset::Datadog => LogFormat::DatadogJson,
            TelemetryPreset::Otel => LogFormat::Json,
            TelemetryPreset::None => LogFormat::Json,
        })
    }

    /// Get the log level from environment or default.
    ///
    /// Checks `RUST_LOG` first, then `TELEMETRY_LOG_LEVEL`, defaults to "info".
    pub fn log_level_from_env() -> String {
        std::env::var("RUST_LOG")
            .or_else(|_| std::env::var("TELEMETRY_LOG_LEVEL"))
            .unwrap_or_else(|_| "info".to_owned())
    }
}

impl TelemetryConfig {
    /// Load configuration from environment variables.
    ///
    /// # Environment Variables
    ///
    /// ## Preset configuration
    ///
    /// | Variable | Values | Default |
    /// |----------|--------|---------|
    /// | `TELEMETRY_PRESET` | local/datadog/otel/none | `local` |
    /// | `TELEMETRY_SERVICE_NAME` | string | required for datadog/otel |
    /// | `RUST_LOG` or `TELEMETRY_LOG_LEVEL` | EnvFilter syntax | `info` |
    /// | `TELEMETRY_LOG_FORMAT` | pretty/json/compact/datadog_json | (from preset) |
    /// | `TELEMETRY_DATADOG_ENDPOINT` | url | `http://localhost:8126` |
    /// | `TELEMETRY_OTLP_ENDPOINT` | url | `http://localhost:4317` |
    ///
    /// ## Metrics configuration (independent from presets)
    ///
    /// | Variable | Values | Default |
    /// |----------|--------|---------|
    /// | `TELEMETRY_METRICS_BACKEND` | prometheus/statsd/none | `none` |
    /// | `TELEMETRY_PROMETHEUS_MODE` | http/push | `http` |
    /// | `TELEMETRY_PROMETHEUS_LISTEN` | addr:port | `0.0.0.0:9090` |
    /// | `TELEMETRY_PROMETHEUS_ENDPOINT` | url | - |
    /// | `TELEMETRY_PROMETHEUS_INTERVAL` | seconds | `10` |
    /// | `TELEMETRY_STATSD_HOST` | string | `localhost` |
    /// | `TELEMETRY_STATSD_PORT` | u16 | `8125` |
    /// | `TELEMETRY_STATSD_PREFIX` | string | - |
    pub fn from_env() -> eyre::Result<Self> {
        let service_name = env::var("TELEMETRY_SERVICE_NAME").ok();

        let preset = env::var("TELEMETRY_PRESET")
            .ok()
            .map(|s| TelemetryPreset::from_str(&s))
            .transpose()?
            .unwrap_or_default();

        let log_format = env::var("TELEMETRY_LOG_FORMAT")
            .ok()
            .map(|s| LogFormat::from_str(&s))
            .transpose()?;

        let datadog_endpoint = env::var("TELEMETRY_DATADOG_ENDPOINT").ok();
        let otlp_endpoint = env::var("TELEMETRY_OTLP_ENDPOINT").ok();

        // --- Metrics configuration ---
        let prometheus = PrometheusConfig {
            mode: env::var("TELEMETRY_PROMETHEUS_MODE")
                .ok()
                .map(|s| PrometheusMode::from_str(&s))
                .transpose()?
                .unwrap_or_default(),
            listen: env::var("TELEMETRY_PROMETHEUS_LISTEN")
                .ok()
                .map(|s| {
                    s.parse()
                        .map_err(|_| eyre!("invalid TELEMETRY_PROMETHEUS_LISTEN: {s}"))
                })
                .transpose()?
                .unwrap_or_else(default_prometheus_listen),
            endpoint: env::var("TELEMETRY_PROMETHEUS_ENDPOINT").ok(),
            interval: env::var("TELEMETRY_PROMETHEUS_INTERVAL")
                .ok()
                .map(|s| {
                    s.parse::<u64>()
                        .map(Duration::from_secs)
                        .map_err(|_| {
                            eyre!("invalid TELEMETRY_PROMETHEUS_INTERVAL: expected integer seconds, got '{s}'")
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
                    s.parse()
                        .map_err(|_| eyre!("invalid TELEMETRY_STATSD_PORT: expected u16, got '{s}'"))
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

        // --- Eyre configuration ---
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
            preset,
            service_name,
            log_format,
            datadog_endpoint,
            otlp_endpoint,
            metrics,
            eyre,
        })
    }
}
