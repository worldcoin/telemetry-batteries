use crate::tracing::layers::stdout::stdout_layer;
use crate::tracing::TracingShutdownHandle;
use tracing_subscriber::{
    layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer,
};

pub struct StdoutBattery;

impl StdoutBattery {
    pub fn init() -> TracingShutdownHandle {
        let stdout_layer = stdout_layer();
        let layers = EnvFilter::from_default_env().and_then(stdout_layer);
        tracing_subscriber::registry().with(layers).init();

        TracingShutdownHandle
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    #[tokio::test]
    async fn test_init() {
        env::set_var("RUST_LOG", "info");
        let _shutdown_handle = StdoutBattery::init();

        for _ in 0..5 {
            let span = tracing::span!(tracing::Level::INFO, "test_span");
            span.in_scope(|| {
                tracing::info!("test_event");
            });
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }
}
