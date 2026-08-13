# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0](https://github.com/worldcoin/telemetry-batteries/compare/v0.3.2...v0.4.0) - 2026-08-13

### Added

- [**breaking**] bump otel 0.32 deps & reqwest 0.13 ([#77](https://github.com/worldcoin/telemetry-batteries/pull/77))
- propagate traces in reqwest clients ([#74](https://github.com/worldcoin/telemetry-batteries/pull/74))

### Added

- Outgoing trace-context propagation for `reqwest::RequestBuilder` and
  `reqwest-middleware` clients.

## [0.3.2](https://github.com/worldcoin/telemetry-batteries/compare/v0.3.1...v0.3.2) - 2026-06-24

### Other

- remove dead code ([#70](https://github.com/worldcoin/telemetry-batteries/pull/70))
- Nice panic & top level error handling ([#71](https://github.com/worldcoin/telemetry-batteries/pull/71))
- Improve CI workflow ([#68](https://github.com/worldcoin/telemetry-batteries/pull/68))
