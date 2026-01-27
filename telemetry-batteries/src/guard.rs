//! RAII guard for telemetry shutdown.

use tracing_appender::non_blocking::WorkerGuard;

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
    /// Worker guard for non-blocking file appender (if configured).
    #[allow(dead_code)]
    worker_guard: Option<WorkerGuard>,
}

impl TelemetryGuard {
    /// Create a new telemetry guard.
    pub(crate) fn new(worker_guard: Option<WorkerGuard>) -> Self {
        Self { worker_guard }
    }
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        tracing::info!("Shutting down telemetry");
        opentelemetry::global::shutdown_tracer_provider();
    }
}
