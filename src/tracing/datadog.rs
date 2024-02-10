use crate::tracing::layers::{
    datadog::datadog_layer, non_blocking_writer_layer,
};
use tracing_appender::rolling::RollingFileAppender;
use tracing_subscriber::{
    layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer,
};

pub const DEFAULT_DATADOG_AGENT_ENDPOINT: &str = "http://localhost:8126";

pub struct DatadogBattery;

impl DatadogBattery {
    pub fn init(
        endpoint: Option<&str>,
        service_name: &str,
        file_appender: Option<RollingFileAppender>,
        location: bool,
    ) {
        let endpoint = endpoint.unwrap_or(DEFAULT_DATADOG_AGENT_ENDPOINT);

        let datadog_layer = datadog_layer(service_name, endpoint, location);

        if let Some(file_appender) = file_appender {
            let file_writer_layer = non_blocking_writer_layer(file_appender);

            let layers = EnvFilter::from_default_env()
                .and_then(datadog_layer)
                .and_then(file_writer_layer);

            tracing_subscriber::registry().with(layers).init();
        } else {
            let layers = EnvFilter::from_default_env().and_then(datadog_layer);
            tracing_subscriber::registry().with(layers).init();
        }
    }
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
        DatadogBattery::init(None, service_name, None, false);

        for _ in 0..1000 {
            tracing::info!("test");
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }
}
