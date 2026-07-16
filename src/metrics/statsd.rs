//! StatsD metrics initialization.

use std::net::UdpSocket;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use cadence::{BufferedUdpMetricSink, MetricSink, QueuingMetricSink};
use metrics_exporter_statsd::{StatsdBuilder, StatsdError};

use crate::config::StatsdConfig;

/// Handle for the background flush thread.
///
/// Signals the thread to stop on drop and waits for it to finish.
pub(crate) struct StatsdFlushHandle {
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}

impl Drop for StatsdFlushHandle {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}

/// Initialize StatsD metrics with the given configuration.
///
/// When [`StatsdConfig::flush_interval`] is `Some(d)`, a background thread is
/// spawned that calls `flush()` on the underlying [`BufferedUdpMetricSink`]
/// every `d`. This prevents silent metric loss at low traffic where the
/// buffer never fills naturally.
pub(crate) fn init(
    config: &StatsdConfig,
) -> Result<Option<StatsdFlushHandle>, StatsdError> {
    // Build the sink manually so we can keep a clone for periodic flushing.
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_nonblocking(true)?;

    let host = (config.host.as_str(), config.port);
    let udp_sink =
        BufferedUdpMetricSink::with_capacity(host, socket, config.buffer_size)?;

    let queuing_sink =
        QueuingMetricSink::with_capacity(udp_sink, config.queue_size);

    // Clone the sink *before* passing ownership to the builder. The clone
    // shares the same underlying `Arc<BufferedUdpMetricSink>`, so calling
    // `flush()` on it flushes the real buffer.
    let flush_sink = queuing_sink.clone();

    let recorder = StatsdBuilder::from(&config.host, config.port)
        .with_sink(queuing_sink)
        .build(config.prefix.as_deref())?;

    metrics::set_global_recorder(recorder)?;

    // Spawn a background flush thread if configured.
    let handle = config.flush_interval.map(|interval| {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_clone = Arc::clone(&stop);

        let thread = thread::Builder::new()
            .name("statsd-flush".into())
            .spawn(move || {
                while !stop_clone.load(Ordering::Relaxed) {
                    thread::sleep(interval);
                    if stop_clone.load(Ordering::Relaxed) {
                        break;
                    }
                    if let Err(e) = flush_sink.flush() {
                        tracing::warn!(
                            error = %e,
                            "periodic statsd flush failed"
                        );
                    }
                }
                // Final flush on shutdown.
                let _ = flush_sink.flush();
            })
            .expect("failed to spawn statsd-flush thread");

        StatsdFlushHandle {
            stop,
            thread: Some(thread),
        }
    });

    Ok(handle)
}
