use super::MetricsBattery;
use crate::error::BatteryError;
use metrics_exporter_statsd::{StatsdBuilder, StatsdError};

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
    fn init(&self) -> Result<(), BatteryError> {
        let recorder = StatsdBuilder::from(self.host, self.port)
            .with_queue_size(self.queue_size)
            .with_buffer_size(self.buffer_size)
            .build(self.prefix)?;

        metrics::set_boxed_recorder(Box::new(recorder))?;

        Ok(())
    }
}
