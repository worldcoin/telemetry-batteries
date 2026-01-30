use std::io::Write;

use tokio::sync::OnceCell;
use tracing::Subscriber;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{Layer, fmt, registry::LookupSpan};

pub mod datadog;
pub mod stdout;

pub fn stdout_layer<S>() -> impl Layer<S>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fmt::layer().with_target(false).with_level(true)
}

static WORKER_GUARD: OnceCell<WorkerGuard> = OnceCell::const_new();

pub fn non_blocking_writer_layer<S, W>(writer: W) -> impl Layer<S>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    W: Write + Send + Sync + 'static,
{
    let (non_blocking, guard) = tracing_appender::non_blocking(writer);
    WORKER_GUARD.set(guard).expect("Could not set worker guard");

    tracing_subscriber::fmt::layer().with_writer(non_blocking)
}
