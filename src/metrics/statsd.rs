//! StatsD metrics initialization.
//!
//! Two things matter when sending to a node-local Datadog agent over UDP:
//!
//! - DogStatsD cannot tell which container a packet came from, so metrics only carry the node's
//!   tags unless the client sends its own (`service`, `env`, `version`, and
//!   `dd.internal.entity_id` for the agent to attach pod tags).
//! - The UDP sink buffers until full. At low traffic that holds samples for minutes, so they land
//!   in the wrong time bucket, and drops them at shutdown. A background thread flushes it on an
//!   interval, and [`StatsdHandle`] flushes once more when dropped.

use std::{
    io,
    net::UdpSocket,
    sync::{
        Arc,
        mpsc::{self, RecvTimeoutError},
    },
    thread::JoinHandle,
};

use cadence::{
    BufferedUdpMetricSink, MetricSink, QueuingMetricSink, SinkStats,
};
use eyre::WrapErr as _;
use metrics_exporter_statsd::StatsdBuilder;

use crate::config::StatsdConfig;

/// Initialize StatsD metrics with the given configuration.
///
/// The returned handle must be kept alive (it lives in [`crate::TelemetryGuard`]): dropping it
/// stops the flush thread and sends whatever is still buffered.
pub(crate) fn init(config: &StatsdConfig) -> eyre::Result<StatsdHandle> {
    let socket = UdpSocket::bind("0.0.0.0:0")
        .wrap_err("binding the StatsD client socket")?;
    socket
        .set_nonblocking(true)
        .wrap_err("making the StatsD client socket non-blocking")?;
    let udp = BufferedUdpMetricSink::with_capacity(
        (config.host.as_str(), config.port),
        socket,
        config.buffer_size,
    )
    .wrap_err_with(|| {
        format!("resolving the StatsD host {}:{}", config.host, config.port)
    })?;
    let sink =
        Arc::new(QueuingMetricSink::with_capacity(udp, config.queue_size));

    let mut builder = StatsdBuilder::from(&config.host, config.port)
        .with_sink(SharedSink(Arc::clone(&sink)));
    for (key, value) in &config.default_tags {
        builder = builder.with_default_tag(key, value);
    }
    let recorder = builder.build(config.prefix.as_deref())?;
    metrics::set_global_recorder(recorder)?;

    let flusher = if config.flush_interval.is_zero() {
        None
    } else {
        let (stop, stopped) = mpsc::channel::<()>();
        let interval = config.flush_interval;
        let sink = Arc::clone(&sink);
        let thread = std::thread::Builder::new()
            .name("statsd-flush".to_owned())
            .spawn(move || {
                let mut failing = false;
                // Runs until the handle drops its sender.
                while let Err(RecvTimeoutError::Timeout) =
                    stopped.recv_timeout(interval)
                {
                    failing = flush(&sink, failing);
                }
            })
            .wrap_err("spawning the StatsD flush thread")?;
        Some((stop, thread))
    };

    Ok(StatsdHandle { sink, flusher })
}

/// Flushes the sink. Warns on the first failure after a success, so a broken socket logs one
/// line rather than one per interval. Returns whether this flush failed.
fn flush(sink: &QueuingMetricSink, was_failing: bool) -> bool {
    match sink.flush() {
        Ok(()) => false,
        Err(error) => {
            if !was_failing {
                tracing::warn!(%error, "flushing StatsD metrics failed");
            }
            true
        }
    }
}

/// Keeps the StatsD flush thread running; flushes once more on drop.
pub(crate) struct StatsdHandle {
    sink: Arc<QueuingMetricSink>,
    flusher: Option<(mpsc::Sender<()>, JoinHandle<()>)>,
}

impl Drop for StatsdHandle {
    fn drop(&mut self) {
        if let Some((stop, thread)) = self.flusher.take() {
            drop(stop);
            if thread.join().is_err() {
                tracing::warn!("the StatsD flush thread panicked");
            }
        }
        flush(&self.sink, false);
    }
}

/// Lets the recorder own a sink that [`StatsdHandle`] can still flush.
struct SharedSink(Arc<QueuingMetricSink>);

impl MetricSink for SharedSink {
    fn emit(&self, metric: &str) -> io::Result<usize> {
        self.0.emit(metric)
    }

    fn flush(&self) -> io::Result<()> {
        self.0.flush()
    }

    fn stats(&self) -> SinkStats {
        self.0.stats()
    }
}
