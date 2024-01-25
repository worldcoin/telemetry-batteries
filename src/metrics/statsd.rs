use metrics_exporter_statsd::{StatsdBuilder, StatsdError};

pub struct StatsdBattery;

impl StatsdBattery {
    pub fn init(
        host: &str,
        port: u16,
        queue_size: usize,
        buffer_size: usize,
        prefix: Option<&str>,
    ) -> Result<(), StatsdError> {
        let recorder = StatsdBuilder::from(host, port)
            .with_queue_size(queue_size)
            .with_buffer_size(buffer_size)
            .build(prefix)?;

        metrics::set_global_recorder(recorder)?;

        Ok(())
    }
}
