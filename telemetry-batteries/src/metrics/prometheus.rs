//! Prometheus metrics initialization.

use metrics_exporter_prometheus::{BuildError, PrometheusBuilder};
use std::{net::SocketAddr, time::Duration};

use crate::config::{PrometheusConfig, PrometheusMode};

/// Initialize Prometheus metrics with the given configuration.
pub(crate) fn init(config: &PrometheusConfig) -> Result<(), BuildError> {
    let mut builder = PrometheusBuilder::new();

    builder = match config.mode {
        PrometheusMode::Http => builder.with_http_listener(config.listen),
        PrometheusMode::Push => {
            if let Some(ref endpoint) = config.endpoint {
                builder.with_push_gateway(
                    endpoint,
                    config.interval,
                    None::<String>,
                    None::<String>,
                    false, // use_http_post_method - use PUT by default per prometheus spec
                )?
            } else {
                // If no endpoint is provided for push mode, fall back to http
                builder.with_http_listener(config.listen)
            }
        }
    };

    builder.install()
}

/// Legacy exporter config enum (kept for reference during migration).
#[allow(dead_code)]
pub(crate) enum PrometheusExporterConfig {
    HttpListener { listen_address: SocketAddr },
    PushGateway {
        endpoint: String,
        interval: Duration,
        username: Option<String>,
        password: Option<String>,
    },
    Unconfigured,
}
