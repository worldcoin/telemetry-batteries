//! Helpers for handling fatal top-level application errors.

use std::any::type_name;
use std::panic;

/// Extension trait for turning fatal top-level `Result` errors into panics.
///
/// This is intended for application boundaries, e.g. `main`, where returning an
/// error means the process should terminate anyway. Panicking lets the telemetry
/// panic hook log the error with the same structured panic fields.
pub trait TopLevelResultExt<T> {
    /// Return the success value, or panic with a structured top-level error.
    ///
    /// Call this after [`crate::init`] so telemetry-batteries' panic hook can
    /// log the structured payload.
    #[track_caller]
    fn panic_on_top_level_error(self) -> T;
}

impl<T, E> TopLevelResultExt<T> for Result<T, E>
where
    E: Into<eyre::Report> + Send + Sync + 'static,
{
    #[track_caller]
    fn panic_on_top_level_error(self) -> T {
        match self {
            Ok(value) => value,
            Err(error) => panic::panic_any(TopLevelError::from_error(error)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TopLevelError {
    message: String,
    error_type: &'static str,
    error_chain: Vec<String>,
    debug: String,
}

impl TopLevelError {
    pub(crate) fn from_error<E>(error: E) -> Self
    where
        E: Into<eyre::Report> + Send + Sync + 'static,
    {
        let error_type = type_name::<E>();
        let report = error.into();
        let message = report.to_string();
        let debug = format!("{report:?}");
        let error_chain = report.chain().map(ToString::to_string).collect();

        Self {
            message,
            error_type,
            error_chain,
            debug,
        }
    }

    pub(crate) fn message(&self) -> &str {
        &self.message
    }

    pub(crate) fn error_type(&self) -> &'static str {
        self.error_type
    }

    pub(crate) fn error_chain(&self) -> &[String] {
        &self.error_chain
    }

    pub(crate) fn debug(&self) -> &str {
        &self.debug
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::*;

    #[test]
    fn common_errors_satisfy_top_level_error_bounds() {
        fn assert_bounds<E: Into<eyre::Report> + Send + Sync + 'static>() {}

        assert_bounds::<eyre::Report>();
        assert_bounds::<io::Error>();
    }

    #[test]
    fn panic_on_top_level_error_panics_with_structured_payload() {
        let panic = std::panic::catch_unwind(|| {
            let result: Result<(), io::Error> =
                Err(io::Error::other("top-level failed"));
            result.panic_on_top_level_error();
        })
        .expect_err("result should panic");

        let error = panic
            .downcast_ref::<TopLevelError>()
            .expect("panic payload should be TopLevelError");

        assert_eq!(error.message(), "top-level failed");
        assert_eq!(error.error_chain(), &["top-level failed".to_owned()]);
        assert_eq!(error.error_type(), "std::io::error::Error");
    }
}
