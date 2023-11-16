pub mod datadog;

use std::path::PathBuf;
use std::{fs, io};
use tracing_opentelemetry::OpenTelemetrySpanExt;

pub trait TracingBattery {
    fn init(&self);
}

pub fn trace_from_headers(headers: &http::HeaderMap) {
    tracing::Span::current().set_parent(opentelemetry::global::get_text_map_propagator(
        |propagator| propagator.extract(&opentelemetry_http::HeaderExtractor(headers)),
    ));
}

pub fn trace_to_headers(headers: &mut http::HeaderMap) {
    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.inject_context(
            &tracing::Span::current().context(),
            &mut opentelemetry_http::HeaderInjector(headers),
        );
    });
}

/// Platform agnostic function to get the path to the log directory. If the directory does not
/// exist, it will be created.
///
/// # Returns
/// * `Ok(PathBuf)` containing the path to the `.logs` directory in the user's home directory.
/// * `Err(io::Error)` if the home directory cannot be determined, or the `.logs` directory
///   cannot be created.
///
/// # Errors
/// This function will return an `Err` if the home directory cannot be found or the `.logs`
/// directory cannot be created. It does not guarantee that the `.logs` directory is writable.
pub fn get_log_directory() -> Result<PathBuf, io::Error> {
    let home_dir = dirs::home_dir().ok_or(io::ErrorKind::NotFound)?;
    let log_dir = home_dir.join(".logs");

    // Create the `.logs` directory if it does not exist
    if !log_dir.exists() {
        fs::create_dir_all(&log_dir)?;
    }

    Ok(log_dir)
}
