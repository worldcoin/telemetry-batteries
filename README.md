# telemetry-batteries

Batteries-included telemetry for Rust applications. Configure tracing, metrics, and error reporting with a single function call.

## Quick Start

```rust
#[tokio::main]
async fn main() -> eyre::Result<()> {
    // Initialize from environment variables
    let _guard = telemetry_batteries::init()?;

    tracing::info!("Hello, telemetry!");

    Ok(())
}
```

The guard must be kept alive for the duration of your application. When dropped, it gracefully shuts down the telemetry providers.

`init()` also installs a global panic hook. Panics are logged with `tracing::error`
and structured fields like `source`, `payload_type`, location, thread, and
backtrace; normal panic unwind/abort behavior is unchanged.

For fatal top-level errors, opt into the same panic path:

```rust,no_run
use telemetry_batteries::TopLevelResultExt;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let _guard = telemetry_batteries::init()?;
    run().await.panic_on_top_level_error();
    Ok(())
}

async fn run() -> eyre::Result<()> {
    eyre::bail!("fatal startup error")
}
```

## Configuration

Configuration is done through environment variables for each telemetry
backend. Local pretty logs are enabled by default. Datadog distributed tracing
is enabled automatically when a service name is present:

```bash
DD_SERVICE=my-service cargo run
```

Standard `OTEL_*` variables follow the
[OpenTelemetry SDK environment-variable specification](https://opentelemetry.io/docs/specs/otel/configuration/sdk-environment-variables/)
for the exporters supported by this crate. Empty `OTEL_*` values are treated as
unset. Datadog-specific variables are retained for selecting and connecting to
the Datadog integration.

### Environment Variables

| Variable | Values | Default |
|----------|--------|---------|
| `DD_SERVICE` | string | enables Datadog when set |
| `DD_TRACE_AGENT_URL` | URL | derived from `DD_AGENT_HOST` |
| `DD_AGENT_HOST` | hostname or IP | `localhost` |
| `DD_TRACE_AGENT_PORT` | port | `8126` |
| `DD_TRACE_ENABLED` | `true`, `false` | `true` |
| `OTEL_TRACES_EXPORTER` | `none` | - |
| `RUST_LOG` | [EnvFilter syntax](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html) | `info` |
| `LOG_FORMAT` | `pretty`, `json`, `compact`, `datadog_json` | `pretty`, or `datadog_json` with Datadog |

`OTEL_TRACES_EXPORTER=none` or the Datadog compatibility setting
`DD_TRACE_ENABLED=false` disables distributed span export without disabling log
output. The standard OpenTelemetry setting takes precedence when both are set.
Use `RUST_LOG=off` to disable logs.

### Metrics Configuration

Metrics are configured independently:

| Variable | Values | Default |
|----------|--------|---------|
| `OTEL_METRICS_EXPORTER` | `prometheus`, `none` | `none` |
| `OTEL_EXPORTER_PROMETHEUS_HOST` | IP address or `localhost` | `localhost` |
| `OTEL_EXPORTER_PROMETHEUS_PORT` | port | `9464` |

Prometheus is enabled by `OTEL_METRICS_EXPORTER=prometheus`, while
`OTEL_METRICS_EXPORTER=none` explicitly disables metrics. This crate does not
currently provide the specification's OTLP or console metrics exporters, so it
defaults to `none` instead of the specification's `otlp`. StatsD remains
available through programmatic configuration; it is not exposed as an
`OTEL_METRICS_EXPORTER` value because StatsD is not defined by the specification.

### Programmatic Configuration

For more control, use the builder pattern:

```rust
use telemetry_batteries::{
    TelemetryConfig, LogFormat,
    MetricsConfig, MetricsBackend, StatsdConfig,
};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let config = TelemetryConfig::builder()
        .service_name("my-service".to_owned())
        .log_format(LogFormat::Pretty)
        .tracing_enabled(true)
        .metrics(MetricsConfig::builder()
            .backend(MetricsBackend::Statsd)
            .statsd(StatsdConfig::builder()
                .host("localhost".to_owned())
                .port(8125)
                .build())
            .build())
        .build();

    let _guard = telemetry_batteries::init_with_config(config)?;

    tracing::info!("Configured programmatically!");

    Ok(())
}
```

## Usage Examples

```bash
# Local development - pretty logs, no span export
cargo run

# Datadog production
DD_SERVICE=my-service cargo run

# Datadog with pretty logs for debugging
DD_SERVICE=my-service LOG_FORMAT=pretty cargo run

# Datadog-formatted logs without distributed span export
DD_SERVICE=my-service OTEL_TRACES_EXPORTER=none cargo run

# With Prometheus metrics
OTEL_METRICS_EXPORTER=prometheus cargo run
```

## Distributed Tracing

For distributed tracing with axum or any Tower-compatible framework, use `TraceLayer`:

```rust,ignore
use axum::{routing::get, Router};
use telemetry_batteries::tracing::middleware::TraceLayer;

let app = Router::new()
    .route("/", get(handler))
    .layer(TraceLayer::new());
```

The middleware automatically:

- Creates a span for each request
- Extracts trace context from incoming headers (e.g., `traceparent`)
- Injects trace context into response headers

Custom span creation:

```rust
use telemetry_batteries::tracing::middleware::TraceLayer;
use tracing::info_span;

let layer = TraceLayer::new().with_make_span(|req| {
    info_span!(
        "http_request",
        method = %req.method(),
        path = %req.uri().path(),
    )
});
```

### Outgoing requests

Inject the current span's trace context into an individual `reqwest` request:

```rust,ignore
use telemetry_batteries::tracing::reqwest::RequestBuilderExt;

let response = reqwest::Client::new()
    .get("https://example.com")
    .inject_trace_context()
    .send()
    .await?;
```

For `reqwest-middleware`, enable the `reqwest-middleware` feature and attach
`TraceContextMiddleware` once to the client:

```rust,ignore
use telemetry_batteries::tracing::reqwest::middleware::TraceContextMiddleware;

let client = reqwest_middleware::ClientBuilder::new(reqwest::Client::new())
    .with(TraceContextMiddleware::new())
    .build();
```

This integration propagates the current context but does not create client spans.

## Cargo Features

| Feature | Default | Description |
|---------|---------|-------------|
| `metrics-prometheus` | Yes | Prometheus metrics exporter |
| `metrics-statsd` | Yes | StatsD metrics exporter |
| `reqwest-middleware` | No | Automatic outgoing trace-context propagation for `reqwest-middleware` clients |
| `rustls` | Yes | TLS via rustls |
| `native-tls` | No | TLS via native-tls |

## Examples

See the examples directory:

- `basic.rs` - Minimal setup with environment variables
- `axum_tracing.rs` - Axum server with distributed trace propagation

Run the examples:

```bash
# Basic example with local logging
cargo run --example basic

# Basic example with Datadog
DD_SERVICE=test cargo run --example basic

# Axum server with trace propagation
DD_SERVICE=my-api cargo run --example axum_tracing
```

## License

Unless otherwise specified, all code in this repository is dual-licensed under
either:

- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0, with LLVM Exceptions ([LICENSE-APACHE](LICENSE-APACHE))

at your option. This means you may select the license you prefer to use.

Any contribution intentionally submitted for inclusion in the work by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
