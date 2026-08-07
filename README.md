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
requires only a service name and an enable switch:

```bash
DD_SERVICE=my-service DD_ENABLED=true cargo run
```

### Environment Variables

| Variable | Values | Default |
|----------|--------|---------|
| `DD_ENABLED` | `true`, `false` | `false` |
| `DD_SERVICE` | string | required when `DD_ENABLED=true` |
| `DD_TRACE_AGENT_URL` | URL | derived from `DD_AGENT_HOST` |
| `DD_AGENT_HOST` | hostname or IP | `localhost` |
| `DD_TRACE_AGENT_PORT` | port | `8126` |
| `TRACING_ENABLED` | `true`, `false` | `true` |
| `RUST_LOG` | [EnvFilter syntax](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html) | `info` |
| `LOG_FORMAT` | `pretty`, `json`, `compact`, `datadog_json` | `pretty`, or `datadog_json` with Datadog |

`TRACING_ENABLED=false` disables distributed span export without disabling log
output. Use `RUST_LOG=off` when log output should also be disabled.

### Metrics Configuration

Metrics are configured independently:

| Variable | Values | Default |
|----------|--------|---------|
| `METRICS_BACKEND` | `prometheus`, `statsd`, `none` | `none` |
| `PROMETHEUS_MODE` | `http`, `push` | `http` |
| `PROMETHEUS_LISTEN` | `addr:port` | `0.0.0.0:9090` |
| `PROMETHEUS_ENDPOINT` | URL | - |
| `PROMETHEUS_INTERVAL` | seconds | `10` |
| `STATSD_HOST` | string | `localhost` |
| `STATSD_PORT` | u16 | `8125` |
| `STATSD_PREFIX` | string | - |

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
        .datadog_enabled(true)
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
DD_ENABLED=true DD_SERVICE=my-service cargo run

# Datadog with pretty logs for debugging
DD_ENABLED=true DD_SERVICE=my-service LOG_FORMAT=pretty cargo run

# Datadog-formatted logs without distributed span export
DD_ENABLED=true DD_SERVICE=my-service TRACING_ENABLED=false cargo run

# With Prometheus metrics
METRICS_BACKEND=prometheus cargo run
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
DD_ENABLED=true DD_SERVICE=test cargo run --example basic

# Axum server with trace propagation
DD_ENABLED=true DD_SERVICE=my-api cargo run --example axum_tracing
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
