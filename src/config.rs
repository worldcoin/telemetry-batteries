//! Configuration types for telemetry initialization.

use std::{
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};

use bon::Builder;
use eyre::eyre;

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
    fn from_otel_exporter(s: &str) -> eyre::Result<Self> {
        match s.to_lowercase().as_str() {
            "prometheus" => Ok(Self::Prometheus),
            "none" => Ok(Self::None),
            _ => Err(eyre!(
                "unsupported OTEL_METRICS_EXPORTER: expected 'prometheus' or 'none', got '{s}'"
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
    /// Service name. Setting it enables the Datadog integration.
    pub service_name: Option<String>,

    /// Override the default log format.
    pub log_format: Option<LogFormat>,

    /// Datadog Agent endpoint.
    /// Defaults to http://localhost:8126.
    pub datadog_endpoint: Option<String>,

    /// Whether distributed spans should be exported to Datadog.
    ///
    /// This does not disable log output. Use `RUST_LOG=off` to disable logs.
    #[builder(default = true)]
    pub tracing_enabled: bool,

    /// Metrics configuration.
    #[builder(default)]
    pub metrics: MetricsConfig,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            service_name: None,
            log_format: None,
            datadog_endpoint: None,
            tracing_enabled: true,
            metrics: MetricsConfig::default(),
        }
    }
}

impl TelemetryConfig {
    /// Whether the Datadog integration is enabled.
    pub fn datadog_enabled(&self) -> bool {
        self.service_name
            .as_deref()
            .is_some_and(|service_name| !service_name.trim().is_empty())
    }

    /// Get the effective log format based on the enabled backend and override.
    pub fn effective_log_format(&self) -> LogFormat {
        self.log_format.unwrap_or(if self.datadog_enabled() {
            LogFormat::DatadogJson
        } else {
            LogFormat::Pretty
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
    /// | `DD_SERVICE` | string | enables Datadog when set |
    /// | `DD_TRACE_AGENT_URL` | url | derived from `DD_AGENT_HOST` |
    /// | `DD_AGENT_HOST` | hostname/IP | `localhost` |
    /// | `DD_TRACE_AGENT_PORT` | port | `8126` |
    /// | `DD_TRACE_ENABLED` | true/false | `true` |
    /// | `OTEL_TRACES_EXPORTER` | none | - |
    /// | `RUST_LOG` | EnvFilter syntax | `info` |
    /// | `LOG_FORMAT` | pretty/json/compact/datadog_json | (from backend) |
    ///
    /// ## Metrics configuration
    ///
    /// | Variable | Values | Default |
    /// |----------|--------|---------|
    /// | `OTEL_METRICS_EXPORTER` | prometheus/none | `none` |
    /// | `OTEL_EXPORTER_PROMETHEUS_HOST` | hostname/IP | `localhost` |
    /// | `OTEL_EXPORTER_PROMETHEUS_PORT` | port | `9464` |
    /// | `DD_DOGSTATSD_URL` | udp URL | - |
    /// | `DD_DOGSTATSD_PORT` | port | - |
    pub fn from_env() -> eyre::Result<Self> {
        Self::from_env_with(|name| env::var(name).ok())
    }

    fn from_env_with(
        get: impl Fn(&str) -> Option<String>,
    ) -> eyre::Result<Self> {
        let service_name = get("DD_SERVICE")
            .filter(|service_name| !service_name.trim().is_empty());
        let tracing_enabled = match get("DD_TRACE_ENABLED") {
            Some(value) => parse_bool("DD_TRACE_ENABLED", &value)?,
            None => match get("OTEL_TRACES_EXPORTER") {
                Some(value) if value.eq_ignore_ascii_case("none") => false,
                Some(value) => {
                    return Err(eyre!(
                        "unsupported OTEL_TRACES_EXPORTER: only 'none' is supported, got '{value}'"
                    ));
                }
                None => true,
            },
        };

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
        // --- Metrics configuration ---
        let metrics_backend = get("OTEL_METRICS_EXPORTER")
            .map(|value| MetricsBackend::from_otel_exporter(&value))
            .transpose()?;

        let dogstatsd_url = get("DD_DOGSTATSD_URL");
        let dogstatsd_port = get("DD_DOGSTATSD_PORT");
        let dogstatsd_configured =
            dogstatsd_url.is_some() || dogstatsd_port.is_some();

        let backend = match metrics_backend {
            Some(MetricsBackend::Prometheus) if dogstatsd_configured => {
                return Err(eyre!(
                    "conflicting metrics exporters: OTEL_METRICS_EXPORTER=prometheus and DogStatsD configuration are both set"
                ));
            }
            Some(backend) => backend,
            None if dogstatsd_configured => MetricsBackend::Statsd,
            None => MetricsBackend::None,
        };

        let prometheus = if backend == MetricsBackend::Prometheus {
            let host = get("OTEL_EXPORTER_PROMETHEUS_HOST")
                .unwrap_or_else(|| "localhost".to_owned());
            let port = get("OTEL_EXPORTER_PROMETHEUS_PORT")
                .map(|value| {
                    parse_port("OTEL_EXPORTER_PROMETHEUS_PORT", &value)
                })
                .transpose()?
                .unwrap_or(9464);
            PrometheusConfig {
                mode: PrometheusMode::Http,
                listen: parse_prometheus_listen(&host, port)?,
                endpoint: None,
                interval: Duration::from_secs(10),
            }
        } else {
            PrometheusConfig::default()
        };

        let statsd = if backend == MetricsBackend::Statsd {
            let (host, port) = match dogstatsd_url {
                Some(url) => parse_dogstatsd_url(&url)?,
                None => {
                    let host = get("DD_AGENT_HOST")
                        .unwrap_or_else(|| "localhost".to_owned());
                    let port = dogstatsd_port
                        .map(|value| parse_port("DD_DOGSTATSD_PORT", &value))
                        .transpose()?
                        .unwrap_or(8125);
                    (host, port)
                }
            };
            StatsdConfig {
                host,
                port,
                prefix: None,
                queue_size: 5000,
                buffer_size: 1024,
            }
        } else {
            StatsdConfig::default()
        };

        let metrics = MetricsConfig {
            backend,
            prometheus,
            statsd,
        };

        Ok(Self {
            service_name,
            log_format,
            datadog_endpoint,
            tracing_enabled,
            metrics,
        })
    }
}

fn parse_port(name: &str, value: &str) -> eyre::Result<u16> {
    value
        .parse()
        .map_err(|_| eyre!("invalid {name}: expected u16, got '{value}'"))
}

fn parse_prometheus_listen(host: &str, port: u16) -> eyre::Result<SocketAddr> {
    let ip = if host.eq_ignore_ascii_case("localhost") {
        IpAddr::V4(Ipv4Addr::LOCALHOST)
    } else {
        host.parse().map_err(|_| {
            eyre!(
                "invalid OTEL_EXPORTER_PROMETHEUS_HOST: expected an IP address or 'localhost', got '{host}'"
            )
        })?
    };

    Ok(SocketAddr::new(ip, port))
}

fn parse_dogstatsd_url(value: &str) -> eyre::Result<(String, u16)> {
    let authority = value.strip_prefix("udp://").ok_or_else(|| {
        eyre!(
            "unsupported DD_DOGSTATSD_URL: expected udp://host[:port], got '{value}'"
        )
    })?;
    let authority = authority.parse::<http::uri::Authority>().map_err(|_| {
        eyre!("invalid DD_DOGSTATSD_URL: expected udp://host[:port], got '{value}'")
    })?;

    Ok((
        authority.host().to_owned(),
        authority.port_u16().unwrap_or(8125),
    ))
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

        assert!(!config.datadog_enabled());
        assert_eq!(config.effective_log_format(), LogFormat::Pretty);
        assert!(config.tracing_enabled);
        assert_eq!(config.metrics.backend, MetricsBackend::None);
    }

    #[test]
    fn enables_datadog_when_a_service_name_is_present() {
        let config = config(&[("DD_SERVICE", "accounts-api")]).unwrap();

        assert!(config.datadog_enabled());
        assert_eq!(config.service_name.as_deref(), Some("accounts-api"));
        assert_eq!(config.effective_log_format(), LogFormat::DatadogJson);
        assert!(config.tracing_enabled);
    }

    #[test]
    fn empty_service_names_do_not_enable_datadog() {
        for entries in [vec![("DD_SERVICE", "")], vec![("DD_SERVICE", "  ")]] {
            let config = config(&entries).unwrap();
            assert!(!config.datadog_enabled());
            assert_eq!(config.service_name, None);
        }
    }

    #[test]
    fn datadog_native_flag_can_disable_distributed_tracing() {
        let config = config(&[
            ("DD_SERVICE", "accounts-api"),
            ("DD_TRACE_ENABLED", "false"),
        ])
        .unwrap();

        assert!(config.datadog_enabled());
        assert!(!config.tracing_enabled);
        assert_eq!(config.effective_log_format(), LogFormat::DatadogJson);
    }

    #[test]
    fn opentelemetry_flag_can_disable_distributed_tracing() {
        let config = config(&[
            ("DD_SERVICE", "accounts-api"),
            ("OTEL_TRACES_EXPORTER", "none"),
        ])
        .unwrap();

        assert!(!config.tracing_enabled);
    }

    #[test]
    fn datadog_trace_flag_takes_precedence_over_opentelemetry() {
        let config = config(&[
            ("DD_SERVICE", "accounts-api"),
            ("DD_TRACE_ENABLED", "true"),
            ("OTEL_TRACES_EXPORTER", "none"),
        ])
        .unwrap();

        assert!(config.tracing_enabled);
    }

    #[test]
    fn rejects_invalid_datadog_boolean_values() {
        let error = config(&[("DD_TRACE_ENABLED", "yes")]).unwrap_err();
        assert_eq!(
            error.to_string(),
            "invalid DD_TRACE_ENABLED: expected 'true' or 'false', got 'yes'"
        );
    }

    #[test]
    fn does_not_read_the_legacy_telemetry_namespace() {
        let config = config(&[
            ("TELEMETRY_SERVICE_NAME", "legacy-service"),
            ("TELEMETRY_LOG_FORMAT", "json"),
        ])
        .unwrap();

        assert!(!config.datadog_enabled());
        assert_eq!(config.service_name, None);
        assert_eq!(config.log_format, None);
    }

    #[test]
    fn configures_prometheus_from_opentelemetry_variables() {
        let config = config(&[
            ("LOG_FORMAT", "compact"),
            ("DD_TRACE_AGENT_URL", "http://agent:8126"),
            ("OTEL_METRICS_EXPORTER", "prometheus"),
            ("OTEL_EXPORTER_PROMETHEUS_HOST", "0.0.0.0"),
            ("OTEL_EXPORTER_PROMETHEUS_PORT", "8080"),
        ])
        .unwrap();

        assert_eq!(config.log_format, Some(LogFormat::Compact));
        assert_eq!(
            config.datadog_endpoint.as_deref(),
            Some("http://agent:8126")
        );
        assert_eq!(config.metrics.backend, MetricsBackend::Prometheus);
        assert_eq!(
            config.metrics.prometheus.listen,
            "0.0.0.0:8080".parse().unwrap()
        );
    }

    #[test]
    fn uses_opentelemetry_prometheus_defaults() {
        let config =
            config(&[("OTEL_METRICS_EXPORTER", "prometheus")]).unwrap();

        assert_eq!(config.metrics.backend, MetricsBackend::Prometheus);
        assert_eq!(
            config.metrics.prometheus.listen,
            "127.0.0.1:9464".parse().unwrap()
        );
    }

    #[test]
    fn rejects_unsupported_opentelemetry_exporters() {
        let error = config(&[("OTEL_METRICS_EXPORTER", "otlp")]).unwrap_err();

        assert_eq!(
            error.to_string(),
            "unsupported OTEL_METRICS_EXPORTER: expected 'prometheus' or 'none', got 'otlp'"
        );
    }

    #[test]
    fn configures_statsd_from_datadog_url() {
        let config =
            config(&[("DD_DOGSTATSD_URL", "udp://metrics:18125")]).unwrap();

        assert_eq!(config.metrics.backend, MetricsBackend::Statsd);
        assert_eq!(config.metrics.statsd.host, "metrics");
        assert_eq!(config.metrics.statsd.port, 18125);
    }

    #[test]
    fn configures_statsd_from_datadog_host_and_port() {
        let config = config(&[
            ("DD_AGENT_HOST", "10.20.30.40"),
            ("DD_DOGSTATSD_PORT", "18125"),
        ])
        .unwrap();

        assert_eq!(config.metrics.backend, MetricsBackend::Statsd);
        assert_eq!(config.metrics.statsd.host, "10.20.30.40");
        assert_eq!(config.metrics.statsd.port, 18125);
    }

    #[test]
    fn explicit_none_disables_dogstatsd_discovery() {
        let config = config(&[
            ("OTEL_METRICS_EXPORTER", "none"),
            ("DD_DOGSTATSD_PORT", "8125"),
        ])
        .unwrap();

        assert_eq!(config.metrics.backend, MetricsBackend::None);
    }

    #[test]
    fn rejects_conflicting_metrics_exporters() {
        let error = config(&[
            ("OTEL_METRICS_EXPORTER", "prometheus"),
            ("DD_DOGSTATSD_PORT", "8125"),
        ])
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "conflicting metrics exporters: OTEL_METRICS_EXPORTER=prometheus and DogStatsD configuration are both set"
        );
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
