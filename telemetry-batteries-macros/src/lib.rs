use proc_macro::TokenStream;

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
///
/// ```rust
/// #[datadog(service_name = "my_service", endpoint = "http://localhost:8126", location = true)]
/// #[tokio::main]
/// fn async main() {
///     // Application logic here
/// }
/// ```

#[proc_macro_attribute]
pub fn datadog(attr: TokenStream, item: TokenStream) -> TokenStream {
    tracing::datadog::datadog(attr, item)
}
