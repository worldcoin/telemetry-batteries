# Eyre Integration Summary

This document summarizes the eyre error JSON formatting integration into telemetry-batteries.

## What Was Implemented

### 1. Core Module Structure
- **`src/eyre/mod.rs`**: Main public API with `EyreBattery` and `EyreConfig`
- **`src/eyre/formatter.rs`**: `ErrorReport` struct for JSON serialization
- **`src/eyre/ext.rs`**: `ReportExt` trait for convenient error conversion

### 2. Dependencies
- Dependencies: `eyre = "0.6"`, `tracing-error = "0.2"`
- Always available (no feature flag required)

### 3. Two Modes of Operation

#### Mode 1: Global Hook (All errors as JSON)
```rust
use telemetry_batteries::eyre::{EyreBattery, EyreConfig};

EyreBattery::install(EyreConfig::default())?;
// Now all eyre errors format as JSON
```

**Use case**: When you want consistent JSON formatting everywhere

#### Mode 2: Manual Conversion
```rust
use telemetry_batteries::eyre::ReportExt;

let err = eyre!("failed");
let json = err.to_json_value();
tracing::error!(error = ?json, "Operation failed");
```

**Use case**: Explicit control - pretty errors in CLI, JSON in logs

### 4. Key Features

#### Faithful Serialization
The `ErrorReport` struct serializes exactly what eyre provides in its text output:

```rust
pub struct ErrorReport {
    pub message: String,
    pub causes: Vec<String>,
    pub backtrace: Option<String>,
    pub spantrace: Option<String>,
}
```

#### EyreConfig Options
- `EyreConfig::default()`: JSON enabled, no backtraces
- `EyreConfig::full()`: JSON + backtraces enabled
- `EyreConfig::datadog()`: Same as default (compatibility alias)

### 5. Documentation
- Updated CLAUDE.md with eyre integration section
- Inline documentation in all modules
- Usage examples in doc comments

## Design Decisions

### Why Global Hook?
The global hook approach provides:

1. **Simplicity**: Single initialization point
2. **Automatic**: Works with all logging patterns
3. **No boilerplate**: Users don't need to manually convert errors

### Why Two Modes?
Different use cases require different trade-offs:
- **Mode 1** (Global): Best for services that always want JSON errors
- **Mode 2** (Manual): Best for CLI tools that need pretty errors but want JSON in specific contexts

## Migration from eyre-json

For users of the standalone `eyre-json` crate:

**Before:**
```rust
use eyre_json::{install_json_hook, ReportExt};
install_json_hook(false)?;
```

**After:**
```rust
use telemetry_batteries::eyre::{EyreBattery, EyreConfig};
EyreBattery::install(EyreConfig::default())?;
```

The API is nearly identical, providing JSON serialization of eyre's standard output.

## Summary

The eyre JSON formatting feature provides faithful JSON serialization of eyre error reports, making them suitable for structured logging systems while preserving all the information eyre normally provides (message, causes, backtrace, spantrace).
