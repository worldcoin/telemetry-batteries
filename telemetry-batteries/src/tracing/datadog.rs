//! Datadog tracing initialization.

use crate::tracing::layers::datadog::datadog_layer;
use opentelemetry_datadog::DatadogPropagator;
use tracing_error::ErrorLayer;
use tracing_subscriber::{
    layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer,
};

use super::TracingShutdownHandle;

pub(crate) const DEFAULT_DATADOG_AGENT_ENDPOINT: &str = "http://localhost:8126";

/// Initialize Datadog tracing with the given configuration.
pub(crate) fn init(
    endpoint: Option<&str>,
    service_name: &str,
    location: bool,
    log_level: &str,
) -> TracingShutdownHandle {
    opentelemetry::global::set_text_map_propagator(DatadogPropagator::new());

    let endpoint = endpoint.unwrap_or(DEFAULT_DATADOG_AGENT_ENDPOINT);

    let (dd_layer, provider) = datadog_layer(service_name, endpoint, location);
    let layers = EnvFilter::new(log_level).and_then(dd_layer);
    tracing_subscriber::registry()
        .with(layers)
        .with(ErrorLayer::default())
        .init();

    TracingShutdownHandle::new(provider)
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    #[ignore]
    #[tokio::test]
    async fn test_init() {
        env::set_var("RUST_LOG", "info");
        let service_name = "test_service";
        let _shutdown_handle = init(None, service_name, false, "info");

        for _ in 0..10 {
            tracing::info!("test");
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    }
}
