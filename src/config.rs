//! Configuration types for telemetry initialization.

use std::{env, net::SocketAddr, time::Duration};

use bon::Builder;
use eyre::eyre;

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
            _ => Err(eyre!(
                "invalid LOG_FORMAT: expected 'pretty', 'json', 'compact', or 'datadog_json', got '{s}'"
            )),
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
            _ => Err(eyre!(
                "invalid METRICS_BACKEND: expected 'prometheus', 'statsd', or 'none', got '{s}'"
            )),
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
            _ => Err(eyre!(
                "invalid PROMETHEUS_MODE: expected 'http' or 'push', got '{s}'"
            )),
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

/// Main telemetry configuration.
#[derive(Debug, Clone, Builder)]
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

    /// Whether distributed spans should be exported by the selected preset.
    ///
    /// This does not disable log output. Use `RUST_LOG=off` to disable logs.
    #[builder(default = true)]
    pub tracing_enabled: bool,

    /// Metrics configuration (independent from preset).
    #[builder(default)]
    pub metrics: MetricsConfig,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            preset: TelemetryPreset::default(),
            service_name: None,
            log_format: None,
            datadog_endpoint: None,
            otlp_endpoint: None,
            tracing_enabled: true,
            metrics: MetricsConfig::default(),
        }
    }
}

impl TelemetryConfig {
    /// Get the effective log format based on preset and override.
    pub fn effective_log_format(&self) -> LogFormat {
        self.log_format.unwrap_or(match self.preset {
            TelemetryPreset::Local => LogFormat::Pretty,
            TelemetryPreset::Datadog => LogFormat::DatadogJson,
            TelemetryPreset::Otel => LogFormat::Json,
            TelemetryPreset::None => LogFormat::Json,
        })
    }

    /// Get the log level from environment or default.
    ///
    /// Checks `RUST_LOG` and defaults to "info".
    pub fn log_level_from_env() -> String {
        std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_owned())
    }
}

impl TelemetryConfig {
    /// Load configuration from environment variables.
    ///
    /// # Environment Variables
    ///
    /// ## Logging and distributed tracing
    ///
    /// | Variable | Values | Default |
    /// |----------|--------|---------|
    /// | `DD_ENABLED` | true/false | `false` |
    /// | `DD_SERVICE` | string | required when `DD_ENABLED=true` |
    /// | `DD_TRACE_AGENT_URL` | url | derived from `DD_AGENT_HOST` |
    /// | `DD_AGENT_HOST` | hostname/IP | `localhost` |
    /// | `DD_TRACE_AGENT_PORT` | port | `8126` |
    /// | `TRACING_ENABLED` | true/false | `true` |
    /// | `RUST_LOG` | EnvFilter syntax | `info` |
    /// | `LOG_FORMAT` | pretty/json/compact/datadog_json | (from backend) |
    ///
    /// ## Metrics configuration (independent from presets)
    ///
    /// | Variable | Values | Default |
    /// |----------|--------|---------|
    /// | `METRICS_BACKEND` | prometheus/statsd/none | `none` |
    /// | `PROMETHEUS_MODE` | http/push | `http` |
    /// | `PROMETHEUS_LISTEN` | addr:port | `0.0.0.0:9090` |
    /// | `PROMETHEUS_ENDPOINT` | url | - |
    /// | `PROMETHEUS_INTERVAL` | seconds | `10` |
    /// | `STATSD_HOST` | string | `localhost` |
    /// | `STATSD_PORT` | u16 | `8125` |
    /// | `STATSD_PREFIX` | string | - |
    pub fn from_env() -> eyre::Result<Self> {
        Self::from_env_with(|name| env::var(name).ok())
    }

    fn from_env_with(
        get: impl Fn(&str) -> Option<String>,
    ) -> eyre::Result<Self> {
        let datadog_enabled = get("DD_ENABLED")
            .map(|value| parse_bool("DD_ENABLED", &value))
            .transpose()?
            .unwrap_or(false);
        let service_name = get("DD_SERVICE");

        if datadog_enabled
            && service_name
                .as_deref()
                .is_none_or(|name| name.trim().is_empty())
        {
            return Err(eyre!("DD_SERVICE is required when DD_ENABLED=true"));
        }

        let preset = if datadog_enabled {
            TelemetryPreset::Datadog
        } else {
            TelemetryPreset::Local
        };

        let tracing_enabled = get("TRACING_ENABLED")
            .map(|value| parse_bool("TRACING_ENABLED", &value))
            .transpose()?
            .unwrap_or(true);

        let log_format = get("LOG_FORMAT")
            .map(|s| LogFormat::from_str(&s))
            .transpose()?;

        let datadog_endpoint = match get("DD_TRACE_AGENT_URL") {
            Some(url) => Some(url),
            None => match get("DD_AGENT_HOST") {
                Some(host) => {
                    let port = get("DD_TRACE_AGENT_PORT")
                        .map(|value| {
                            value.parse::<u16>().map_err(|_| {
                                eyre!(
                                    "invalid DD_TRACE_AGENT_PORT: expected u16, got '{value}'"
                                )
                            })
                        })
                        .transpose()?
                        .unwrap_or(8126);
                    Some(format!("http://{host}:{port}"))
                }
                None => None,
            },
        };
        let otlp_endpoint = get("OTEL_EXPORTER_OTLP_ENDPOINT");

        // --- Metrics configuration ---
        let prometheus = PrometheusConfig {
            mode: get("PROMETHEUS_MODE")
                .map(|s| PrometheusMode::from_str(&s))
                .transpose()?
                .unwrap_or_default(),
            listen: get("PROMETHEUS_LISTEN")
                .map(|s| {
                    s.parse()
                        .map_err(|_| eyre!("invalid PROMETHEUS_LISTEN: {s}"))
                })
                .transpose()?
                .unwrap_or_else(default_prometheus_listen),
            endpoint: get("PROMETHEUS_ENDPOINT"),
            interval: get("PROMETHEUS_INTERVAL")
                .map(|s| {
                    s.parse::<u64>()
                        .map(Duration::from_secs)
                        .map_err(|_| {
                            eyre!("invalid PROMETHEUS_INTERVAL: expected integer seconds, got '{s}'")
                        })
                })
                .transpose()?
                .unwrap_or(Duration::from_secs(10)),
        };

        let statsd = StatsdConfig {
            host: get("STATSD_HOST").unwrap_or_else(|| "localhost".to_owned()),
            port: get("STATSD_PORT")
                .map(|s| {
                    s.parse().map_err(|_| {
                        eyre!("invalid STATSD_PORT: expected u16, got '{s}'")
                    })
                })
                .transpose()?
                .unwrap_or(8125),
            prefix: get("STATSD_PREFIX"),
            queue_size: 5000,
            buffer_size: 1024,
        };

        let metrics = MetricsConfig {
            backend: get("METRICS_BACKEND")
                .map(|s| MetricsBackend::from_str(&s))
                .transpose()?
                .unwrap_or_default(),
            prometheus,
            statsd,
        };

        Ok(Self {
            preset,
            service_name,
            log_format,
            datadog_endpoint,
            otlp_endpoint,
            tracing_enabled,
            metrics,
        })
    }
}

fn parse_bool(name: &str, value: &str) -> eyre::Result<bool> {
    match value.to_ascii_lowercase().as_str() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(eyre!(
            "invalid {name}: expected 'true' or 'false', got '{value}'"
        )),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn config(entries: &[(&str, &str)]) -> eyre::Result<TelemetryConfig> {
        let env = entries
            .iter()
            .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
            .collect::<HashMap<_, _>>();

        TelemetryConfig::from_env_with(|name| env.get(name).cloned())
    }

    #[test]
    fn defaults_to_local_logging_without_span_export_backend() {
        let config = config(&[]).unwrap();

        assert_eq!(config.preset, TelemetryPreset::Local);
        assert_eq!(config.effective_log_format(), LogFormat::Pretty);
        assert!(config.tracing_enabled);
        assert_eq!(config.metrics.backend, MetricsBackend::None);
    }

    #[test]
    fn enables_datadog_from_two_environment_variables() {
        let config =
            config(&[("DD_ENABLED", "true"), ("DD_SERVICE", "accounts-api")])
                .unwrap();

        assert_eq!(config.preset, TelemetryPreset::Datadog);
        assert_eq!(config.service_name.as_deref(), Some("accounts-api"));
        assert_eq!(config.effective_log_format(), LogFormat::DatadogJson);
        assert!(config.tracing_enabled);
    }

    #[test]
    fn datadog_requires_a_non_empty_service_name() {
        for entries in [
            vec![("DD_ENABLED", "true")],
            vec![("DD_ENABLED", "true"), ("DD_SERVICE", "")],
            vec![("DD_ENABLED", "true"), ("DD_SERVICE", "  ")],
        ] {
            let error = config(&entries).unwrap_err();
            assert_eq!(
                error.to_string(),
                "DD_SERVICE is required when DD_ENABLED=true"
            );
        }
    }

    #[test]
    fn distributed_tracing_can_be_disabled_without_disabling_logs() {
        let config = config(&[
            ("DD_ENABLED", "true"),
            ("DD_SERVICE", "accounts-api"),
            ("TRACING_ENABLED", "false"),
        ])
        .unwrap();

        assert_eq!(config.preset, TelemetryPreset::Datadog);
        assert!(!config.tracing_enabled);
        assert_eq!(config.effective_log_format(), LogFormat::DatadogJson);
    }

    #[test]
    fn rejects_invalid_boolean_values() {
        let error = config(&[("DD_ENABLED", "yes")]).unwrap_err();
        assert_eq!(
            error.to_string(),
            "invalid DD_ENABLED: expected 'true' or 'false', got 'yes'"
        );
    }

    #[test]
    fn does_not_read_the_legacy_telemetry_namespace() {
        let config = config(&[
            ("TELEMETRY_PRESET", "datadog"),
            ("TELEMETRY_SERVICE_NAME", "legacy-service"),
        ])
        .unwrap();

        assert_eq!(config.preset, TelemetryPreset::Local);
        assert_eq!(config.service_name, None);
    }

    #[test]
    fn reads_unscoped_backend_configuration() {
        let config = config(&[
            ("LOG_FORMAT", "compact"),
            ("DD_TRACE_AGENT_URL", "http://agent:8126"),
            ("METRICS_BACKEND", "statsd"),
            ("STATSD_HOST", "metrics"),
            ("STATSD_PORT", "18125"),
        ])
        .unwrap();

        assert_eq!(config.log_format, Some(LogFormat::Compact));
        assert_eq!(
            config.datadog_endpoint.as_deref(),
            Some("http://agent:8126")
        );
        assert_eq!(config.metrics.backend, MetricsBackend::Statsd);
        assert_eq!(config.metrics.statsd.host, "metrics");
        assert_eq!(config.metrics.statsd.port, 18125);
    }

    #[test]
    fn discovers_the_datadog_agent_from_standard_environment_variables() {
        let config = config(&[
            ("DD_AGENT_HOST", "10.20.30.40"),
            ("DD_TRACE_AGENT_PORT", "18126"),
        ])
        .unwrap();

        assert_eq!(
            config.datadog_endpoint.as_deref(),
            Some("http://10.20.30.40:18126")
        );
    }
}
