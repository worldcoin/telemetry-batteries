# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Outgoing trace-context propagation for `reqwest::RequestBuilder` and
  `reqwest-middleware` clients.
- `TRACING_ENABLED` can disable distributed span export independently of log
  output.

### Changed

- **Breaking:** environment configuration no longer uses the `TELEMETRY_*`
  namespace. Enable Datadog with only `DD_ENABLED=true` and `DD_SERVICE`; use
  provider-specific names such as `DD_AGENT_HOST`, `METRICS_BACKEND`,
  `PROMETHEUS_*`, and `STATSD_*` for optional settings.
- `RUST_LOG` is now the sole log-filter environment variable, and `LOG_FORMAT`
  controls formatting.

## [0.3.2](https://github.com/worldcoin/telemetry-batteries/compare/v0.3.1...v0.3.2) - 2026-06-24

### Other

- remove dead code ([#70](https://github.com/worldcoin/telemetry-batteries/pull/70))
- Nice panic & top level error handling ([#71](https://github.com/worldcoin/telemetry-batteries/pull/71))
- Improve CI workflow ([#68](https://github.com/worldcoin/telemetry-batteries/pull/68))
