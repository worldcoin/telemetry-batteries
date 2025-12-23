# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`telemetry-batteries` is a Rust library providing "batteries included" configuration for tracing, logging, and metrics. It offers pre-configured backends for Datadog, Prometheus, and StatsD with minimal setup required.

## Workspace Structure

This is a Cargo workspace with two crates:
- `telemetry-batteries`: Main library with runtime functionality
- `telemetry-batteries-macros`: Procedural macros for ergonomic initialization

## Development Commands

### Building
```bash
cargo build                    # Build workspace
cargo build --all-features     # Build with all features
```

### Testing
```bash
cargo test --workspace         # Run all tests
cargo test -p telemetry-batteries  # Run tests for main crate only
```

Note: Many tests are marked with `#[ignore]` as they require external services (Datadog agent, StatsD server). Run these manually when needed.

### Linting and Formatting
```bash
cargo clippy --workspace --all-targets --all-features  # Run clippy
cargo fmt --all --check        # Check formatting
cargo fmt --all                # Apply formatting
```

Note: Code is formatted with `max_width = 80` (see rustfmt.toml).

### Documentation
```bash
cargo doc --workspace --all-features --no-deps --document-private-items
```

### Running Examples
```bash
cargo run --example datadog
cargo run --example prometheus
cargo run --example custom_tracing
```

## Architecture

### Core Design Pattern: "Battery" Structs

The library uses a "Battery" pattern - stateless structs with `init()` methods that configure and install global telemetry backends. Examples:
- `DatadogBattery::init()` - configures Datadog tracing with OpenTelemetry
- `StdoutBattery::init()` - configures stdout tracing
- `PrometheusBattery::init()` - configures Prometheus metrics exporter
- `StatsdBattery::init()` - configures StatsD metrics exporter

### Tracing Architecture

Tracing is built on the `tracing` ecosystem with OpenTelemetry integration:

1. **Layers** (`src/tracing/layers/`): Composable tracing layers
   - `datadog.rs`: Datadog-specific layer with OpenTelemetry exporter
   - `stdout.rs`: JSON stdout logging layer

2. **Utilities** (`src/tracing/mod.rs`):
   - `trace_from_headers()` / `trace_to_headers()`: Distributed tracing propagation
   - `opentelemetry_trace_id()` / `opentelemetry_span_id()`: Extract OpenTelemetry IDs from tracing spans
   - `extract_span_ids()`: Get current span's trace/span IDs
   - `TracingShutdownHandle`: RAII handle ensuring graceful OpenTelemetry shutdown

3. **ID Generation** (`src/tracing/id_generator.rs`): Custom OpenTelemetry ID generators compatible with Datadog

### Metrics Architecture

Metrics use the `metrics` crate facade with pluggable backends:

- **Prometheus** (`src/metrics/prometheus.rs`): Supports HTTP listener or push gateway modes
- **StatsD** (`src/metrics/statsd.rs`): UDP-based metrics via `metrics-exporter-statsd`

Both follow the Battery pattern for initialization.

### Macros Architecture

The `telemetry-batteries-macros` crate provides procedural macros:

- `#[datadog]`: Wraps async main functions to initialize Datadog tracing
- `#[statsd]`: Wraps async main functions to initialize StatsD metrics

These macros expect `#[tokio::main]` to be applied after them.

## Feature Flags

Default features: `["metrics-prometheus", "metrics-statsd", "rustls"]`

- `metrics-prometheus`: Enable Prometheus metrics backend
- `metrics-statsd`: Enable StatsD metrics backend
- `rustls`: Use rustls TLS for reqwest (default)
- `native-tls`: Use native TLS for reqwest (mutually exclusive with rustls)

## Key Dependencies

- OpenTelemetry ecosystem: Core telemetry abstractions
- `tracing-subscriber`: Composable tracing layers
- `opentelemetry-datadog`: Datadog exporter with trace propagation
- `metrics-exporter-prometheus` / `metrics-exporter-statsd`: Metrics backends

## Common Patterns

### Initializing Datadog Tracing
```rust
use telemetry_batteries::tracing::datadog::DatadogBattery;

let _guard = DatadogBattery::init(
    Some("http://localhost:8126"),  // endpoint
    "my-service",                    // service name
    None,                            // optional file appender
    false                            // include location in traces
);
```

### Distributed Tracing
```rust
use telemetry_batteries::tracing::{trace_from_headers, trace_to_headers};

// Extract parent trace from incoming HTTP headers
trace_from_headers(&request.headers());

// Inject current trace into outgoing HTTP headers
trace_to_headers(&mut request.headers_mut());
```

### Eyre Error JSON Formatting

The library provides JSON formatting for eyre error reports, faithfully serializing what eyre already provides in its text output.

**Two modes of operation:**

1. **Global Hook Mode** - All eyre errors formatted as JSON everywhere:
```rust
use telemetry_batteries::eyre::{EyreBattery, EyreConfig};

EyreBattery::install(EyreConfig::default())?;
// Now all errors print as JSON
```

2. **Manual Conversion Mode** - Explicit control over when to use JSON:
```rust
use telemetry_batteries::eyre::ReportExt;

let err = eyre!("failed");
let json = err.to_json_value();
tracing::error!(error = ?json, "Operation failed");
```

**Key features:**
- Serializes eyre's error message, cause chain, backtrace, and spantrace
- Optional backtrace inclusion via config
- No additional fields beyond what eyre provides

### Environment Variables
- `RUST_LOG`: Controls tracing level (uses `EnvFilter` from `tracing-subscriber`)

## Testing Notes

- Tests requiring external services are marked `#[ignore]`
- The library is designed to work with async runtimes (primarily tokio)
- Shutdown behavior is tested via the `TracingShutdownHandle` drop implementation
