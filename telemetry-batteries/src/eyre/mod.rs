//! Eyre error reporting with two modes:
//!
//! - **Color**: Human-readable colored multi-line output (default). Useful for local development and debugging.
//! - **Json**: Single-line JSON output (see [`json_eyre`] for schema). Useful in production environments where logs are collected e.g., by Datadog.
//!
//! # Environment Variables
//!
//! Backtrace/spantrace capture can be controlled via environment variables.
//! If set, they override the `with_default_*` config flags:
//!
//! | Feature   | Env Var                                  | Enabled     | Disabled |
//! |-----------|------------------------------------------|-------------|----------|
//! | Backtrace | `RUST_LIB_BACKTRACE` or `RUST_BACKTRACE` | any (not 0) | `0`      |
//! | Spantrace | `RUST_SPANTRACE`                         | any (not 0) | `0`      |

pub mod json_eyre;

use crate::config::{EyreConfig, EyreMode};

/// Initialize eyre error reporting with the given configuration.
pub(crate) fn init(config: &EyreConfig) -> eyre::Result<()> {
    match config.mode {
        EyreMode::Color => {
            color_eyre::install()?;
            Ok(())
        }
        EyreMode::Json => {
            json_eyre::install(
                config.with_default_backtrace,
                config.with_default_spantrace,
            )?;
            Ok(())
        }
    }
}
