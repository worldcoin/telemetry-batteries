//! Datadog tracing initialization.

use crate::config::LogFormat;
use crate::tracing::layers::datadog::datadog_layer;
use opentelemetry_datadog::DatadogPropagator;
use tracing_error::ErrorLayer;
use tracing_subscriber::{
    EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt,
};

use super::TracingShutdownHandle;

pub(crate) const DEFAULT_DATADOG_AGENT_ENDPOINT: &str = "http://localhost:8126";

/// Initialize Datadog tracing with the given configuration.
///
/// This sets up both logging (with Datadog trace correlation) and span export to the Datadog Agent.
pub(crate) fn init(
    endpoint: Option<&str>,
    service_name: &str,
    log_format: LogFormat,
    log_level: &str,
) -> TracingShutdownHandle {
    opentelemetry::global::set_text_map_propagator(DatadogPropagator::new());

    let endpoint = endpoint.unwrap_or(DEFAULT_DATADOG_AGENT_ENDPOINT);

    let (dd_layer, provider) =
        datadog_layer(service_name, endpoint, log_format);
    let layers = EnvFilter::new(log_level).and_then(dd_layer);
    tracing_subscriber::registry()
        .with(layers)
        .with(ErrorLayer::default())
        .init();

    TracingShutdownHandle::new(provider)
}
