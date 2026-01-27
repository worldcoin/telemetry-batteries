//! Stdout tracing initialization.

use crate::tracing::layers::stdout::stdout_layer;
use crate::tracing::TracingShutdownHandle;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize stdout tracing.
pub(crate) fn init() -> TracingShutdownHandle {
    let stdout_layer = stdout_layer();
    tracing_subscriber::registry().with(stdout_layer).init();

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
        let _shutdown_handle = init();

        for _ in 0..1000 {
            let span = tracing::span!(tracing::Level::INFO, "test_span");
            span.in_scope(|| {
                tracing::info!("test_event");
            });
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }
}
