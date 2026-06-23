//! Panic hook that routes panics through tracing.

use std::any::{Any, type_name};
use std::backtrace::Backtrace;
use std::panic::PanicHookInfo;
use std::thread;

use crate::top_level::TopLevelError;

const PANIC_LOG_TARGET: &str = "telemetry_batteries::panic";

/// Install a global panic hook that logs panics through tracing.
pub(crate) fn install() {
    let previous_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |info| {
        if tracing::enabled!(target: PANIC_LOG_TARGET, tracing::Level::ERROR) {
            let details = PanicDetails::from_hook_info(info);
            let backtrace = Backtrace::force_capture();

            log_panic(&details, &backtrace);
        } else if let Some(error) =
            info.payload().downcast_ref::<TopLevelError>()
        {
            print_top_level_error(info, error);
        } else {
            previous_hook(info);
        }
    }));
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PanicDetails {
    source: &'static str,
    message: String,
    payload_type: &'static str,
    top_level_error: Option<TopLevelError>,
    location_file: String,
    location_line: u32,
    location_column: u32,
    thread_name: Option<String>,
    thread_id: String,
}

impl PanicDetails {
    fn from_hook_info(info: &PanicHookInfo<'_>) -> Self {
        let payload = panic_payload(info.payload());
        let current_thread = thread::current();
        let location = info.location();

        Self {
            source: payload.source,
            message: payload.message,
            payload_type: payload.payload_type,
            top_level_error: payload.top_level_error,
            location_file: location
                .map(|location| location.file().to_owned())
                .unwrap_or_else(|| "<unknown>".to_owned()),
            location_line: location.map_or(0, |location| location.line()),
            location_column: location.map_or(0, |location| location.column()),
            thread_name: current_thread.name().map(ToOwned::to_owned),
            thread_id: format!("{:?}", current_thread.id()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PanicPayloadDetails {
    source: &'static str,
    message: String,
    payload_type: &'static str,
    top_level_error: Option<TopLevelError>,
}

fn panic_payload(payload: &(dyn Any + Send)) -> PanicPayloadDetails {
    if let Some(error) = payload.downcast_ref::<TopLevelError>() {
        PanicPayloadDetails {
            source: "top_level_error",
            message: error.message().to_owned(),
            payload_type: type_name::<TopLevelError>(),
            top_level_error: Some(error.clone()),
        }
    } else if let Some(message) = payload.downcast_ref::<&'static str>() {
        PanicPayloadDetails {
            source: "panic",
            message: (*message).to_owned(),
            payload_type: "&str",
            top_level_error: None,
        }
    } else if let Some(message) = payload.downcast_ref::<String>() {
        PanicPayloadDetails {
            source: "panic",
            message: message.clone(),
            payload_type: "String",
            top_level_error: None,
        }
    } else {
        PanicPayloadDetails {
            source: "panic",
            message: "<non-string panic payload>".to_owned(),
            payload_type: "unknown",
            top_level_error: None,
        }
    }
}

fn log_panic(details: &PanicDetails, backtrace: &Backtrace) {
    let thread_name = details.thread_name.as_deref().unwrap_or("<unnamed>");

    if let Some(error) = &details.top_level_error {
        tracing::error!(
            target: PANIC_LOG_TARGET,
            source = details.source,
            payload_type = details.payload_type,
            error_type = error.error_type(),
            error_chain = ?error.error_chain(),
            error_debug = %error.debug(),
            location_file = details.location_file.as_str(),
            location_line = details.location_line,
            location_column = details.location_column,
            thread_name = thread_name,
            thread_id = details.thread_id.as_str(),
            backtrace = %backtrace,
            "{}",
            details.message,
        );
    } else {
        tracing::error!(
            target: PANIC_LOG_TARGET,
            source = details.source,
            payload_type = details.payload_type,
            location_file = details.location_file.as_str(),
            location_line = details.location_line,
            location_column = details.location_column,
            thread_name = thread_name,
            thread_id = details.thread_id.as_str(),
            backtrace = %backtrace,
            "{}",
            details.message,
        );
    }
}

fn print_top_level_error(info: &PanicHookInfo<'_>, error: &TopLevelError) {
    eprintln!("top-level error: {}", error.message());
    eprintln!("error type: {}", error.error_type());

    if error.error_chain().len() > 1 {
        eprintln!("error chain:");
        for (index, source) in error.error_chain().iter().enumerate() {
            eprintln!("  {index}: {source}");
        }
    }

    if let Some(location) = info.location() {
        eprintln!(
            "location: {}:{}:{}",
            location.file(),
            location.line(),
            location.column()
        );
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::sync::{Arc, Mutex};

    use tracing_subscriber::fmt::MakeWriter;
    use tracing_subscriber::prelude::*;

    use super::*;

    #[test]
    fn panic_payload_extracts_string_messages() {
        let static_message: Box<dyn Any + Send> = Box::new("static panic");
        let payload = panic_payload(static_message.as_ref());
        assert_eq!(payload.source, "panic");
        assert_eq!(payload.message, "static panic");
        assert_eq!(payload.payload_type, "&str");

        let owned_message: Box<dyn Any + Send> =
            Box::new("owned panic".to_owned());
        let payload = panic_payload(owned_message.as_ref());
        assert_eq!(payload.source, "panic");
        assert_eq!(payload.message, "owned panic");
        assert_eq!(payload.payload_type, "String");
    }

    #[test]
    fn panic_payload_extracts_top_level_errors() {
        let error = TopLevelError::from_error(io::Error::other("boom"));
        let payload: Box<dyn Any + Send> = Box::new(error.clone());
        let details = panic_payload(payload.as_ref());

        assert_eq!(details.source, "top_level_error");
        assert_eq!(details.message, "boom");
        assert_eq!(details.payload_type, type_name::<TopLevelError>());
        assert_eq!(details.top_level_error, Some(error));
    }

    #[test]
    fn panic_log_contains_structured_fields() {
        let logs = BufWriter::new();
        let subscriber = tracing_subscriber::registry().with(
            tracing_subscriber::fmt::Layer::new()
                .json()
                .with_writer(logs.clone()),
        );

        tracing::subscriber::with_default(subscriber, || {
            log_panic(
                &PanicDetails {
                    source: "panic",
                    message: "boom".to_owned(),
                    payload_type: "&str",
                    top_level_error: None,
                    location_file: "src/main.rs".to_owned(),
                    location_line: 42,
                    location_column: 7,
                    thread_name: Some("main".to_owned()),
                    thread_id: "ThreadId(1)".to_owned(),
                },
                &Backtrace::force_capture(),
            );
        });

        let log: serde_json::Value =
            serde_json::from_str(logs.contents().trim())
                .expect("log line is not valid JSON");
        let fields = &log["fields"];

        assert_eq!(fields["message"], "boom");
        assert_eq!(fields["source"], "panic");
        assert_eq!(fields["payload_type"], "&str");
        assert_eq!(fields["location_file"], "src/main.rs");
        assert_eq!(fields["location_line"], 42);
        assert_eq!(fields["location_column"], 7);
        assert_eq!(fields["thread_name"], "main");
        assert_eq!(fields["thread_id"], "ThreadId(1)");
        assert!(fields["backtrace"].is_string());
    }

    #[test]
    fn top_level_error_log_contains_error_fields() {
        let logs = BufWriter::new();
        let subscriber = tracing_subscriber::registry().with(
            tracing_subscriber::fmt::Layer::new()
                .json()
                .with_writer(logs.clone()),
        );
        let error = TopLevelError::from_error(io::Error::other("boom"));

        tracing::subscriber::with_default(subscriber, || {
            log_panic(
                &PanicDetails {
                    source: "top_level_error",
                    message: error.message().to_owned(),
                    payload_type: type_name::<TopLevelError>(),
                    top_level_error: Some(error),
                    location_file: "src/main.rs".to_owned(),
                    location_line: 42,
                    location_column: 7,
                    thread_name: Some("main".to_owned()),
                    thread_id: "ThreadId(1)".to_owned(),
                },
                &Backtrace::force_capture(),
            );
        });

        let log: serde_json::Value =
            serde_json::from_str(logs.contents().trim())
                .expect("log line is not valid JSON");
        let fields = &log["fields"];

        assert_eq!(fields["message"], "boom");
        assert_eq!(fields["source"], "top_level_error");
        assert_eq!(fields["payload_type"], type_name::<TopLevelError>());
        assert_eq!(fields["error_type"], "std::io::error::Error");
        assert!(fields["error_debug"].as_str().unwrap().contains("boom"));
        assert!(fields["error_chain"].as_str().unwrap().contains("boom"));
        assert!(fields["backtrace"].is_string());
    }

    #[derive(Clone)]
    struct BufWriter(Arc<Mutex<Vec<u8>>>);

    impl BufWriter {
        fn new() -> Self {
            Self(Arc::new(Mutex::new(Vec::new())))
        }

        fn contents(&self) -> String {
            String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
        }
    }

    impl io::Write for BufWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl<'a> MakeWriter<'a> for BufWriter {
        type Writer = BufWriter;

        fn make_writer(&'a self) -> Self::Writer {
            self.clone()
        }
    }
}
