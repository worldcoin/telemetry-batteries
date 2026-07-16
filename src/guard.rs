//! RAII guard for telemetry shutdown.

use crate::tracing::TracingShutdownHandle;

/// Guard that ensures telemetry is properly shut down when dropped.
///
/// This guard holds resources that need to remain alive for the duration
/// of the program. When dropped, it gracefully shuts down the tracing
/// provider and stops any background metric-flush threads.
#[must_use]
pub struct TelemetryGuard {
    tracing_handle: Option<TracingShutdownHandle>,
    /// Keep the statsd flush handle alive; dropping it signals the flush
    /// thread to stop and joins it.
    #[cfg(feature = "metrics-statsd")]
    _statsd_flush: Option<crate::metrics::statsd::StatsdFlushHandle>,
}

impl TelemetryGuard {
    pub(crate) fn new(tracing_handle: Option<TracingShutdownHandle>) -> Self {
        Self {
            tracing_handle,
            #[cfg(feature = "metrics-statsd")]
            _statsd_flush: None,
        }
    }

    /// Attach a statsd flush handle to keep alive for the guard's lifetime.
    #[cfg(feature = "metrics-statsd")]
    pub(crate) fn with_statsd_flush(
        mut self,
        handle: Option<crate::metrics::statsd::StatsdFlushHandle>,
    ) -> Self {
        self._statsd_flush = handle;
        self
    }
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        tracing::info!("Shutting down telemetry");
        // Explicitly drop to trigger TracingShutdownHandle::drop()
        drop(self.tracing_handle.take());
    }
}
