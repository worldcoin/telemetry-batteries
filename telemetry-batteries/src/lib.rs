pub mod metrics;
pub mod tracing;

/// Reexports of crates that appear in the public API.
///
/// Using these directly instead of adding them yourself to Cargo.toml will help avoid
/// errors where types have the same name but actually are distinct types from different
/// crate versions.
pub mod reexports {
    pub use ::opentelemetry;
}
