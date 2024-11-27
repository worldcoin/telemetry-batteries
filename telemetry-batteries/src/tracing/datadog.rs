use std::{backtrace::Backtrace, panic};

use crate::tracing::layers::{
    datadog::datadog_layer, non_blocking_writer_layer,
};
use opentelemetry_datadog::DatadogPropagator;
use tracing_appender::rolling::RollingFileAppender;
use tracing_subscriber::{
    layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer,
};

use super::TracingShutdownHandle;

pub const DEFAULT_DATADOG_AGENT_ENDPOINT: &str = "http://localhost:8126";

pub struct DatadogBattery;

impl DatadogBattery {
    pub fn init(
        endpoint: Option<&str>,
        service_name: &str,
        file_appender: Option<RollingFileAppender>,
        location: bool,
    ) -> TracingShutdownHandle {
        opentelemetry::global::set_text_map_propagator(DatadogPropagator::new());

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
        // Set a custom panic hook to print errors on one line with backtrace
        panic::set_hook(Box::new(|panic_info| {
            let message = match panic_info.payload().downcast_ref::<&str>() {
                Some(s) => *s,
                None => match panic_info.payload().downcast_ref::<String>() {
                    Some(s) => s.as_str(),
                    None => "Unknown panic message",
                },
            };
            let location = if let Some(location) = panic_info.location() {
                format!(
                    "{}:{}:{}",
                    location.file(),
                    location.line(),
                    location.column()
                )
            } else {
                "Unknown location".to_string()
            };

            let backtrace = Backtrace::capture();
            let backtrace_string = format!("{:?}", backtrace);

            let backtrace_single_line = backtrace_string.replace('\n', " | ");

            tracing::error!(
                backtrace = %backtrace_single_line,
                location = %location,
                "Panic occurred with message: {}",
                message
            );
        }));

        TracingShutdownHandle
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::panic;
    use std::sync::{Arc, Mutex};

    use tracing::subscriber::set_global_default;
    use tracing_subscriber::{fmt, EnvFilter};

    #[ignore]
    #[tokio::test]
    async fn test_init() {
        env::set_var("RUST_LOG", "info");
        let service_name = "test_service";
        let _shutdown_handle =
            DatadogBattery::init(None, service_name, None, false);

        for _ in 0..10 {
            tracing::info!("test");
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    }

    #[tokio::test]
    async fn test_panic_hook() {
        let service_name = "test_service";
        // Create an in-memory buffer to capture logs
        let log_buffer = Arc::new(Mutex::new(Vec::new()));
        let writer = InMemoryWriter {
            buffer: log_buffer.clone(),
        };

        // Set up tracing subscriber with in-memory writer
        let subscriber = fmt::Subscriber::builder()
            .with_env_filter(EnvFilter::new("error"))
            .with_writer(writer)
            .finish();

        set_global_default(subscriber)
            .expect("Failed to set global subscriber");

        let _shutdown_handle =
            DatadogBattery::init(None, service_name, None, false);

        // Use `catch_unwind` to prevent the panic from aborting the test
        let result = panic::catch_unwind(|| {
            panic!("Test panic message");
        });

        // Assert that a panic occurred
        assert!(result.is_err(), "Expected a panic, but none occurred");

        // Retrieve the logs from the in-memory buffer
        let logs = log_buffer.lock().unwrap();
        let logs_string = String::from_utf8_lossy(&logs);

        assert!(
            logs_string
                .contains("Panic occurred with message: Test panic message"),
            "Panic message not found in logs: {}",
            logs_string
        );

        assert!(
            logs_string.contains("location")
                && logs_string.contains("backtrace"),
            "Expected location and backtrace information not found in logs: {}",
            logs_string
        );
    }

    // Helper struct to act as an in-memory writer
    struct InMemoryWriter {
        buffer: Arc<Mutex<Vec<u8>>>,
    }

    impl std::io::Write for InMemoryWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            let mut buffer = self.buffer.lock().unwrap();
            buffer.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl<'a> fmt::MakeWriter<'a> for InMemoryWriter {
        type Writer = Self;

        fn make_writer(&'a self) -> Self::Writer {
            InMemoryWriter {
                buffer: self.buffer.clone(),
            }
        }
    }
}
