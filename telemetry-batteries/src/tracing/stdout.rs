//! Stdout tracing initialization.

use crate::config::LogFormat;
use crate::tracing::TracingShutdownHandle;
use tracing_subscriber::{
    fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer,
};

/// Initialize stdout tracing with the specified format.
pub(crate) fn init(format: LogFormat) -> TracingShutdownHandle {
    let filter = EnvFilter::from_default_env();

    match format {
        LogFormat::Pretty => {
            let layer = fmt::layer()
                .with_writer(std::io::stdout)
                .pretty()
                .with_target(false)
                .with_line_number(true)
                .with_file(true)
                .with_filter(filter);
            tracing_subscriber::registry().with(layer).init();
        }
        LogFormat::Json => {
            let layer = fmt::layer()
                .with_writer(std::io::stdout)
                .json()
                .with_filter(filter);
            tracing_subscriber::registry().with(layer).init();
        }
        LogFormat::Compact => {
            let layer = fmt::layer()
                .with_writer(std::io::stdout)
                .compact()
                .with_filter(filter);
            tracing_subscriber::registry().with(layer).init();
        }
    }

    TracingShutdownHandle
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    #[ignore]
    #[tokio::test]
    async fn test_init() {
        env::set_var("RUST_LOG", "info");
        let _shutdown_handle = init(LogFormat::Pretty);

        for _ in 0..1000 {
            let span = tracing::span!(tracing::Level::INFO, "test_span");
            span.in_scope(|| {
                tracing::info!("test_event");
            });
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }
}
