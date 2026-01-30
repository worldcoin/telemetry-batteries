//! StatsD metrics initialization.

use metrics_exporter_statsd::{StatsdBuilder, StatsdError};

use crate::config::StatsdConfig;

/// Initialize StatsD metrics with the given configuration.
pub(crate) fn init(config: &StatsdConfig) -> Result<(), StatsdError> {
    let recorder = StatsdBuilder::from(&config.host, config.port)
        .with_queue_size(config.queue_size)
        .with_buffer_size(config.buffer_size)
        .build(config.prefix.as_deref())?;

    metrics::set_global_recorder(recorder)?;

    Ok(())
}
