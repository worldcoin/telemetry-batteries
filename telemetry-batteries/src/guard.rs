//! RAII guard for telemetry shutdown.

use crate::tracing::TracingShutdownHandle;

/// Guard that ensures telemetry is properly shut down when dropped.
///
/// This guard holds resources that need to remain alive for the duration
/// of the program. When dropped, it gracefully shuts down the tracing provider.
#[must_use]
pub struct TelemetryGuard {
    tracing_handle: Option<TracingShutdownHandle>,
}

impl TelemetryGuard {
    pub(crate) fn new(tracing_handle: Option<TracingShutdownHandle>) -> Self {
        Self { tracing_handle }
    }
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        tracing::info!("Shutting down telemetry");
        // Explicitly drop to trigger TracingShutdownHandle::drop()
        drop(self.tracing_handle.take());
    }
}
