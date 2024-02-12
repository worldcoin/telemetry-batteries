use metrics_exporter_prometheus::{BuildError, PrometheusBuilder};

pub struct PrometheusBattery;

impl PrometheusBattery {
    pub fn init() -> Result<(), BuildError> {
        PrometheusBuilder::new().install()
    }
}
