use std::{net::SocketAddr, time::Duration};

use http::Uri;
use metrics_exporter_prometheus::{BuildError, PrometheusBuilder};

pub struct PrometheusBattery;

#[derive(Clone)]
pub enum PrometheusExporterConfig {
    // Run an HTTP listener on the given `listen_address`.
    HttpListener {
        listen_address: SocketAddr,
    },

    // Run a push gateway task sending to the given `endpoint` after `interval` time has elapsed,
    // infinitely.
    PushGateway {
        endpoint: Uri,
        interval: Duration,
        username: Option<String>,
        password: Option<String>,
    },

    #[allow(dead_code)]
    Unconfigured,
}

impl PrometheusBattery {
    pub fn init(
        exporter_config: Option<PrometheusExporterConfig>,
    ) -> Result<(), BuildError> {
        let mut builder = PrometheusBuilder::new();

        builder = match exporter_config {
            Some(PrometheusExporterConfig::HttpListener { listen_address }) => {
                builder.with_http_listener(listen_address)
            }
            Some(PrometheusExporterConfig::PushGateway {
                endpoint,
                interval,
                username,
                password,
            }) => builder.with_push_gateway(
                endpoint.to_string(),
                interval,
                username,
                password,
            )?,
            _ => builder,
        };

        builder.install()
    }
}
