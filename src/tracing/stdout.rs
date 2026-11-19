//! Stdout tracing initialization.

use crate::config::LogFormat;
use crate::tracing::TracingShutdownHandle;
use tracing_subscriber::{
    EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt,
};

/// Initialize stdout tracing with the specified format and log level.
///
/// Note: For `DatadogJson` format without Datadog span export, this falls back to
/// standard JSON format since trace correlation requires the OpenTelemetry layer.
pub(crate) fn init(
    format: LogFormat,
    log_level: &str,
) -> TracingShutdownHandle {
    // The filter is applied to the whole subscriber, not to the fmt layer: a
    // per-layer filter loses the event or span that follows a
    // `log::log_enabled!` call
    // (https://github.com/tokio-rs/tracing/issues/2519).
    let filter = EnvFilter::new(log_level);

    match format {
        LogFormat::Pretty => {
            let layer = fmt::layer()
                .with_writer(std::io::stdout)
                .pretty()
                .with_target(false)
                .with_line_number(true)
                .with_file(true);
            tracing_subscriber::registry()
                .with(filter)
                .with(layer)
                .init();
        }
        LogFormat::Json | LogFormat::DatadogJson => {
            // DatadogJson without the Datadog backend falls back to standard JSON
            // since trace correlation requires the OpenTelemetry layer.
            let layer = fmt::layer().with_writer(std::io::stdout).json();
            tracing_subscriber::registry()
                .with(filter)
                .with(layer)
                .init();
        }
        LogFormat::Compact => {
            let layer = fmt::layer().with_writer(std::io::stdout).compact();
            tracing_subscriber::registry()
                .with(filter)
                .with(layer)
                .init();
        }
    }

    TracingShutdownHandle::empty()
}
