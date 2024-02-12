use proc_macro::TokenStream;

mod metrics;
mod tracing;

/// Macro to initialize Datadog instrumentation
///
/// # Parameters
///
/// - `service_name`: Required string literal that specifies the name of the service.
///
/// - `endpoint`: Optional string literal that specifies the Datadog agent's endpoint
///   to which telemetry data will be sent. If not specified, this value defaults to http://localhost:8126.
///
/// - `location`: Optional boolean indicates whether to include the location in traces. Defaults to `false` if not specified.
///
/// # Usage
///
/// To use the `datadog` macro, apply it to the main function
/// of your application. You must provide the `service_name` parameter, and you may optionally
/// include `endpoint` and `location` parameters. Due to how the `datadog_layer` from `telemetry-batteries` is configured
/// the `main` function must be asynchronous and use the `tokio::main` macro after the `datadog` macro.

#[proc_macro_attribute]
pub fn datadog(attr: TokenStream, item: TokenStream) -> TokenStream {
    tracing::datadog::datadog(attr, item)
}

/// Macro to initialize Stastd metrics backend
///
/// # Parameters
///
/// - `host`: Optional string literal specifying the StatsD server's IP. Defaults to `"localhost"` if not provided.
///
/// - `port`: Optional u16 specifying the port on which the StatsD server is listening.  Defaults to `8125` if not provided.
///
/// - `buffer_size`: Optional usize specifying the buffer size (in bytes) that should be buffered in StatsdClient's memory before the data is sent to the server. Defaults to 256 if not provided.
///
/// - `queue_size`: Optional usize specifying the size of the queue for storing metrics
///   before sending to the server. Defaults to 5000 if not provided.
///
/// - `prefix`: Optional string literal used as a prefix for all metrics sent. No prefix is added if not provided.
///
/// # Usage
///
/// To use the `statsd` macro, apply it to the main function
/// of your application. Due to how the `StatsdBattery` from `telemetry-batteries` is configured
/// the `main` function must be asynchronous and use the `tokio::main` macro after the `statsd` macro.

#[proc_macro_attribute]
pub fn statsd(attr: TokenStream, item: TokenStream) -> TokenStream {
    metrics::statsd::statsd(attr, item)
}
