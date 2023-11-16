use metrics_exporter_statsd::{StatsdBuilder, StatsdError, StatsdRecorder};

use super::MetricsBattery;

pub struct StatsdBattery<'a> {
    pub host: &'a str,
    pub port: u16,
    pub queue_size: usize,
    pub buffer_size: usize,
    pub prefix: Option<&'a str>,
}

impl<'a> StatsdBattery<'a> {
    pub fn new(
        host: &'a str,
        port: u16,
        queue_size: usize,
        buffer_size: usize,
        prefix: Option<&'a str>,
    ) -> Result<Self, StatsdError> {
        Ok(StatsdBattery {
            host,
            port,
            queue_size,
            buffer_size,
            prefix,
        })
    }
}

impl<'a> MetricsBattery for StatsdBattery<'a> {
    fn init(&self) {
        let recorder = StatsdBuilder::from(self.host, self.port)
            .with_queue_size(self.queue_size)
            .with_buffer_size(self.buffer_size)
            .build(self.prefix)
            .expect("TODO: handle this error gracefully");

        metrics::set_boxed_recorder(Box::new(recorder))
            .expect("TODO: Handle this error gracefully");
    }
}
