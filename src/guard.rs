//! RAII guard for telemetry shutdown.

use crate::tracing::TracingShutdownHandle;

/// Guard that ensures telemetry is properly shut down when dropped.
///
/// This guard holds resources that need to remain alive for the duration
/// of the program. When dropped, it gracefully shuts down the tracing provider.
#[must_use]
pub struct TelemetryGuard {
    tracing_handle: Option<TracingShutdownHandle>,
    // Only the StatsD backend holds anything that needs releasing.
    #[cfg_attr(not(feature = "metrics-statsd"), allow(dead_code))]
    metrics_handle: MetricsHandle,
}

/// Metrics resources to release on shutdown.
#[derive(Default)]
pub(crate) struct MetricsHandle {
    /// Flushes buffered StatsD metrics on an interval and once more on drop.
    #[cfg(feature = "metrics-statsd")]
    pub(crate) statsd: Option<crate::metrics::statsd::StatsdHandle>,
}

impl TelemetryGuard {
    pub(crate) fn new(
        tracing_handle: Option<TracingShutdownHandle>,
        metrics_handle: MetricsHandle,
    ) -> Self {
        Self {
            tracing_handle,
            metrics_handle,
        }
    }
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        tracing::info!("Shutting down telemetry");
        // Flush metrics first, so a flush failure can still be logged.
        #[cfg(feature = "metrics-statsd")]
        drop(self.metrics_handle.statsd.take());
        // Explicitly drop to trigger TracingShutdownHandle::drop()
        drop(self.tracing_handle.take());
    }
}
