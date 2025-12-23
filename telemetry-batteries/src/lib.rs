pub mod eyre;
#[cfg(any(feature = "metrics-prometheus", feature = "metrics-statsd"))]
pub mod metrics;
pub mod tracing;

/// Reexports of crates that appear in the public API.
///
/// Using these directly instead of adding them yourself to Cargo.toml will help avoid
/// errors where types have the same name but actually are distinct types from different
/// crate versions.
pub mod reexports {
    #[cfg(any(
        feature = "metrics-prometheus",
        feature = "metrics-statsd"
    ))]
    pub use ::metrics;
    pub use ::opentelemetry;
}
