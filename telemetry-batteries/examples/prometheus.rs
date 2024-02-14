use telemetry_batteries::metrics::prometheus::{
    PrometheusBattery, PrometheusExporterConfig,
};

pub fn main() -> eyre::Result<()> {
    // Configure http listener for Prometheus scrape endpoint
    let prometheus_exporter_config = PrometheusExporterConfig::HttpListener {
        listen_address: "http://0.0.0.0:9998"
            .parse::<std::net::SocketAddr>()?,
    };

    // Initialize the Prometheus metrics exporter
    PrometheusBattery::init(Some(prometheus_exporter_config))?;

    metrics::counter!("foo").increment(1);

    Ok(())
}
