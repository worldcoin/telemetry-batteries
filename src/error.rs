use std::io;

use metrics::SetRecorderError;
use metrics_exporter_statsd::StatsdError;
use opentelemetry::trace::TraceError;
use thiserror::Error;
use tokio::sync::SetError;
use tracing_appender::non_blocking::WorkerGuard;

#[derive(Error, Debug)]
pub enum BatteryError {
    #[error(transparent)]
    TraceError(#[from] TraceError),
    #[error(transparent)]
    WorkerGuardSetError(#[from] SetError<WorkerGuard>),
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error(transparent)]
    SetRecorderError(#[from] SetRecorderError),
    #[error(transparent)]
    StatsdError(#[from] StatsdError),
}
