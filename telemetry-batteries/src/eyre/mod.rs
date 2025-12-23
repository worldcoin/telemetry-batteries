//! Eyre error reporting with two modes:
//!
//! - **ColorEyre**: Human-readable colored multi-line output (default). Useful for local development and debugging.
//! - **JsonEyre**: Single-line JSON output (see [`json_eyre`] for schema). Useful in production environments where logs are collected e.g., by Datadog.
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EyreMode {
    ColorEyre,
    JsonEyre,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EyreConfig {
    mode: EyreMode,
    with_default_backtrace: bool,
    with_default_spantrace: bool,
}

impl Default for EyreConfig {
    fn default() -> Self {
        Self {
            mode: EyreMode::ColorEyre,
            with_default_backtrace: true,
            with_default_spantrace: true,
        }
    }
}

pub struct EyreBattery;

impl EyreBattery {
    pub fn init(config: EyreConfig) -> eyre::Result<()> {
        match config.mode {
            EyreMode::ColorEyre => {
                color_eyre::install()?;
                Ok(())
            }
            EyreMode::JsonEyre => {
                json_eyre::install(
                    config.with_default_backtrace,
                    config.with_default_spantrace,
                )?;
                Ok(())
            }
        }
    }
}
