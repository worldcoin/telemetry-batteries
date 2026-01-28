# telemetry-batteries

Batteries-included telemetry for Rust applications. Configure tracing, metrics, and error reporting with a single function call.

## Quick Start

```rust
fn main() -> Result<(), telemetry_batteries::InitError> {
    // Initialize from environment variables
    let _guard = telemetry_batteries::init()?;

    tracing::info!("Hello, telemetry!");

    Ok(())
}
```

The guard must be kept alive for the duration of your application. When dropped, it gracefully shuts down the telemetry providers.

## Configuration

### Environment Variables

All configuration can be done via environment variables:

| Variable | Values | Default |
|----------|--------|---------|
| `TELEMETRY_SERVICE_NAME` | string | required for Datadog |
| `RUST_LOG` or `TELEMETRY_LOG_LEVEL` | [EnvFilter syntax](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html) | `info` (checks `RUST_LOG` first) |
| `TELEMETRY_TRACING_BACKEND` | `stdout`, `datadog`, `none` | `stdout` |
| `TELEMETRY_TRACING_ENDPOINT` | url | `http://localhost:8126` |
| `TELEMETRY_TRACING_LOCATION` | `true`, `false` | `false` |
| `TELEMETRY_LOG_FORMAT` | `pretty`, `json`, `compact` | `json` |
| `TELEMETRY_EYRE_MODE` | `color`, `json` | `color` |
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
    TelemetryConfig, TracingConfig, TracingBackend,
    MetricsConfig, MetricsBackend, StatsdConfig,
};

fn main() -> Result<(), telemetry_batteries::InitError> {
    let config = TelemetryConfig::builder()
        .service_name("my-service".to_owned())
        .tracing(TracingConfig::builder()
            .backend(TracingBackend::Datadog)
            .location(true)
            .build())
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

## Features

### Tracing Backends

- **stdout** (default): Logs to stdout with configurable format
- **datadog**: Sends traces to Datadog Agent with JSON-formatted logs
- **none**: Disables tracing

### Metrics Backends

- **prometheus**: Exposes metrics via HTTP endpoint or pushes to a gateway
- **statsd**: Sends metrics to a StatsD server
- **none** (default): Disables metrics

### Error Reporting

Integrates with [eyre](https://docs.rs/eyre) for error handling:

- **color** (default): Human-readable colored output with backtraces
- **json**: Machine-readable JSON output for production

## Trace Propagation

For distributed tracing, use the trace propagation utilities:

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

## Custom Layer Composition

For advanced use cases, compose tracing layers directly:

```rust
use telemetry_batteries::tracing::layers::{datadog::datadog_layer, stdout_layer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn main() {
    tracing_subscriber::registry()
        .with(stdout_layer())
        .with(datadog_layer("my-service", "http://localhost:8126", true))
        .init();
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
- `datadog.rs` - Datadog tracing with StatsD metrics
- `prometheus.rs` - Prometheus metrics endpoint
- `custom_tracing.rs` - Custom layer composition

Run an example:

```bash
RUST_LOG=info cargo run -p telemetry-batteries --example basic
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
