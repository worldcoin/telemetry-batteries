# telemetry-batteries

Batteries-included telemetry for Rust applications. Configure tracing, metrics, and error reporting with a single function call.

## Quick Start

```rust
fn main() -> eyre::Result<()> {
    // Initialize from environment variables
    let _guard = telemetry_batteries::init()?;

    tracing::info!("Hello, telemetry!");

    Ok(())
}
```

The guard must be kept alive for the duration of your application. When dropped, it gracefully shuts down the telemetry providers.

## Configuration

Configuration is done via environment variables using **presets**:

### Presets

| Preset | Log Format | Log Output | Span Export | Use Case |
|--------|------------|------------|-------------|----------|
| `local` | pretty | stdout | none | Local development |
| `datadog` | datadog_json | stdout | Datadog Agent | Production with Datadog |
| `none` | - | none | none | Disable telemetry |

### Environment Variables

| Variable | Values | Default |
|----------|--------|---------|
| `TELEMETRY_PRESET` | `local`, `datadog`, `none` | `local` |
| `TELEMETRY_SERVICE_NAME` | string | required for datadog |
| `RUST_LOG` or `TELEMETRY_LOG_LEVEL` | [EnvFilter syntax](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html) | `info` |
| `TELEMETRY_LOG_FORMAT` | `pretty`, `json`, `compact`, `datadog_json` | (from preset) |
| `TELEMETRY_DATADOG_ENDPOINT` | url | `http://localhost:8126` |
| `TELEMETRY_EYRE_MODE` | `color`, `json` | `color` |

### Metrics Configuration

Metrics are configured independently from presets:

| Variable | Values | Default |
|----------|--------|---------|
| `TELEMETRY_METRICS_BACKEND` | `prometheus`, `statsd`, `none` | `none` |
| `TELEMETRY_PROMETHEUS_MODE` | `http`, `push` | `http` |
| `TELEMETRY_PROMETHEUS_LISTEN` | `addr:port` | `0.0.0.0:9090` |
| `TELEMETRY_PROMETHEUS_ENDPOINT` | url | - |
| `TELEMETRY_PROMETHEUS_INTERVAL` | seconds | `10` |
| `TELEMETRY_STATSD_HOST` | string | `localhost` |
| `TELEMETRY_STATSD_PORT` | u16 | `8125` |
| `TELEMETRY_STATSD_PREFIX` | string | - |

### Programmatic Configuration

For more control, use the builder pattern:

```rust
use telemetry_batteries::{
    TelemetryConfig, TelemetryPreset, LogFormat,
    MetricsConfig, MetricsBackend, StatsdConfig,
};

fn main() -> eyre::Result<()> {
    let config = TelemetryConfig::builder()
        .preset(TelemetryPreset::Datadog)
        .service_name("my-service".to_owned())
        .log_format(LogFormat::Pretty)  // Override preset's log format
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
# Local development - pretty logs, no tracing
cargo run

# Datadog production
TELEMETRY_PRESET=datadog TELEMETRY_SERVICE_NAME=my-service cargo run

# Datadog with pretty logs for debugging
TELEMETRY_PRESET=datadog TELEMETRY_SERVICE_NAME=my-service TELEMETRY_LOG_FORMAT=pretty cargo run

# With Prometheus metrics
TELEMETRY_METRICS_BACKEND=prometheus cargo run
```

## Distributed Tracing

For distributed tracing, use the `TraceLayer` middleware with axum or any Tower-compatible framework:

```rust
use axum::{routing::get, Router};
use telemetry_batteries::tracing::middleware::TraceLayer;

let app = Router::new()
    .route("/", get(handler))
    .layer(TraceLayer::new());
```

The middleware automatically:
- Extracts trace context from incoming request headers (e.g., `traceparent`)
- Injects trace context into outgoing response headers

For manual control, use the lower-level functions:

```rust
use telemetry_batteries::tracing::{trace_from_headers, trace_to_headers};

// Extract trace context from incoming request
fn handle_request(headers: &http::HeaderMap) {
    trace_from_headers(headers);
    // ... handle request within the trace context
}

// Inject trace context into outgoing request
fn make_request(headers: &mut http::HeaderMap) {
    trace_to_headers(headers);
    // ... send request with trace headers
}
```

## Cargo Features

| Feature | Default | Description |
|---------|---------|-------------|
| `metrics-prometheus` | Yes | Prometheus metrics exporter |
| `metrics-statsd` | Yes | StatsD metrics exporter |
| `rustls` | Yes | TLS via rustls |
| `native-tls` | No | TLS via native-tls |

## Examples

See the [examples](telemetry-batteries/examples) directory:

- `basic.rs` - Minimal setup with environment variables
- `axum_tracing.rs` - Axum server with distributed trace propagation

Run the examples:

```bash
# Basic example with local preset
cargo run -p telemetry-batteries --example basic

# Basic example with Datadog
TELEMETRY_PRESET=datadog TELEMETRY_SERVICE_NAME=test cargo run -p telemetry-batteries --example basic

# Axum server with trace propagation
TELEMETRY_PRESET=datadog TELEMETRY_SERVICE_NAME=my-api cargo run -p telemetry-batteries --example axum_tracing
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
