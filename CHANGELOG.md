# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Outgoing trace-context propagation for `reqwest::RequestBuilder` and
  `reqwest-middleware` clients.
- Datadog and OpenTelemetry-native variables configure tracing and metrics
  without library-specific enable or backend switches.

### Changed

- **Breaking:** environment configuration no longer uses the `TELEMETRY_*`
  namespace. `DD_SERVICE` enables Datadog tracing, while standard
  `OTEL_TRACES_EXPORTER` and `OTEL_METRICS_EXPORTER` settings select the
  supported exporters. StatsD configuration remains programmatic because it is
  not an OpenTelemetry-defined exporter value.
- `RUST_LOG` is now the sole log-filter environment variable, and `LOG_FORMAT`
  controls formatting.
- **Breaking:** `TelemetryPreset` has been removed. Setting `service_name` now
  enables Datadog directly in programmatic configuration.

## [0.3.2](https://github.com/worldcoin/telemetry-batteries/compare/v0.3.1...v0.3.2) - 2026-06-24

### Other

- remove dead code ([#70](https://github.com/worldcoin/telemetry-batteries/pull/70))
- Nice panic & top level error handling ([#71](https://github.com/worldcoin/telemetry-batteries/pull/71))
- Improve CI workflow ([#68](https://github.com/worldcoin/telemetry-batteries/pull/68))
