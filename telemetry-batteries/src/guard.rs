//! RAII guard for telemetry shutdown.

use crate::tracing::TracingShutdownHandle;

/// Guard that ensures telemetry is properly shut down when dropped.
///
/// This guard holds resources that need to remain alive for the duration
/// of the program. When dropped, it gracefully shuts down the tracing provider.
///
/// # Example
///
/// ```ignore
/// fn main() -> Result<(), telemetry_batteries::InitError> {
///     let _guard = telemetry_batteries::init()?;
///
///     // Your application code here...
///     // The guard is dropped when main exits, shutting down telemetry.
///
///     Ok(())
/// }
/// ```
#[must_use]
pub struct TelemetryGuard {
    /// Tracing shutdown handle - shuts down the tracer provider on drop.
    #[allow(dead_code)]
    tracing_handle: Option<TracingShutdownHandle>,
}

impl TelemetryGuard {
    /// Create a new telemetry guard.
    pub(crate) fn new(tracing_handle: Option<TracingShutdownHandle>) -> Self {
        Self { tracing_handle }
    }
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        tracing::info!("Shutting down telemetry");
        // TracingShutdownHandle::drop() handles the actual shutdown
    }
}
