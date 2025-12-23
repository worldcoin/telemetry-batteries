//! JSON formatter for eyre error reports.
//!
//! # JSON Schema
//!
//! ```json
//! {
//!   "error_chain": [[0, "message"], [1, "cause"], ...],
//!   "backtrace": [[0, {"function": "...", "file": "...", "line": 42}], ...],
//!   "spantrace": [[0, {"full_name": "...", "file": "...", "line": 42, "fields": "key1=value1 key2=value2 ..."}], ...]
//! }
//! ```
//!
//! - `error_chain`: Indexed error messages, last element is root cause
//! - `backtrace`: Optional, omitted if backtrace capture is disabled. Uses the backtrace crate to capture the backtrace.
//! - `spantrace`: Optional, omitted if spantrace capture is disabled. Uses the tracing-error crate to capture the spantrace.

use std::{env, iter::successors};

use eyre::{EyreHandler, Report, Result};
use serde::{Deserialize, Serialize};
use tracing::Metadata;
use tracing_error::SpanTrace;

/// Install the json_eyre hook globally.
pub fn install(
    with_default_backtrace: bool,
    with_default_spantrace: bool,
) -> Result<()> {
    eyre::set_hook(Box::new(move |_| {
        Box::new(Handler::new(with_default_backtrace, with_default_spantrace))
    }))?;

    Ok(())
}
/// Convenience trait to get the backtrace from an eyre::Report in case json_eyre is installed.
pub trait BacktraceExt {
    fn backtrace(&self) -> Option<&backtrace::Backtrace>;
}

impl BacktraceExt for Report {
    fn backtrace(&self) -> Option<&backtrace::Backtrace> {
        self.handler()
            .downcast_ref::<Handler>()
            .and_then(|handler| handler.backtrace.as_ref())
    }
}

/// Convenience trait to get the spantrace from an eyre::Report in case json_eyre is installed.
pub trait SpantraceExt {
    fn spantrace(&self) -> Option<&SpanTrace>;
}

impl SpantraceExt for Report {
    fn spantrace(&self) -> Option<&SpanTrace> {
        self.handler()
            .downcast_ref::<Handler>()
            .and_then(|handler| handler.spantrace.as_ref())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct BacktraceSymbol {
    // Note: None will be serialized as 'null'
    pub function: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub fields: Option<String>,
}

impl BacktraceSymbol {
    pub fn from_symbol(symbol: &backtrace::BacktraceSymbol) -> Self {
        Self {
            function: symbol.name().map(|name| name.to_string()),
            file: symbol
                .filename()
                .map(|filename| filename.display().to_string()),
            line: symbol.lineno(),
            fields: None, // Backtraces don't have fields, only spantraces do
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct SpanFrame {
    pub full_name: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub fields: Option<String>,
}

impl SpanFrame {
    pub fn from_span_info(metadata: &Metadata<'_>, fields: &str) -> Self {
        let fields = if fields.is_empty() {
            None
        } else {
            Some(fields.to_string())
        };

        Self {
            full_name: format!("{}::{}", metadata.target(), metadata.name()),
            file: metadata.file().map(|file| file.to_string()),
            line: metadata.line().map(|line| line as u32),
            fields,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct JsonFormatter {
    /// The error chain of the error. The last element is the root cause.
    /// We include the index for convenience.
    pub error_chain: Vec<(u32, String)>,
    /// The backtrace of the error.
    /// None if backtrace capturing is disabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backtrace: Option<Vec<(u32, BacktraceSymbol)>>,
    /// The spantrace of the error.
    /// None if spantrace capturing is disabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spantrace: Option<Vec<(u32, SpanFrame)>>,
}

impl JsonFormatter {
    pub fn new(
        err: &(dyn std::error::Error + 'static),
        backtrace: Option<&backtrace::Backtrace>,
        spantrace: Option<&SpanTrace>,
    ) -> Self {
        let error_chain = successors(Some(err), |e| e.source())
            .enumerate()
            .map(|(i, e)| (i as u32, e.to_string()))
            .collect::<Vec<_>>();

        let backtrace = backtrace.map(|bt| {
            bt.frames()
                .iter()
                .flat_map(|frame| frame.symbols().iter())
                .enumerate()
                .map(|(i, symbol)| {
                    (i as u32, BacktraceSymbol::from_symbol(symbol))
                })
                .collect::<Vec<_>>()
        });

        let spantrace = spantrace.map(|st| {
            let mut spantrace = Vec::new();
            st.with_spans(|metadata, fields| {
                spantrace.push(SpanFrame::from_span_info(metadata, fields));
                true
            });

            spantrace
                .iter()
                .enumerate()
                .map(|(i, span_frame)| (i as u32, span_frame.clone()))
                .collect::<Vec<_>>()
        });

        Self {
            error_chain,
            backtrace,
            spantrace,
        }
    }
}

#[derive(Debug)]
struct Handler {
    backtrace: Option<backtrace::Backtrace>,
    spantrace: Option<SpanTrace>,
}

impl Handler {
    pub fn new(
        with_default_backtrace: bool,
        with_default_spantrace: bool,
    ) -> Self {
        let with_backtrace = env::var("RUST_LIB_BACKTRACE")
            .or_else(|_| env::var("RUST_BACKTRACE"))
            .map(|val| val != "0")
            .unwrap_or(with_default_backtrace);

        let backtrace = if with_backtrace {
            Some(backtrace::Backtrace::new())
        } else {
            None
        };

        let with_spantrace = env::var("RUST_SPANTRACE")
            .map(|val| val != "0")
            .unwrap_or(with_default_spantrace);

        let spantrace = if with_spantrace {
            Some(SpanTrace::capture())
        } else {
            None
        };

        Self {
            backtrace,
            spantrace,
        }
    }
}

impl EyreHandler for Handler {
    fn debug(
        &self,
        error: &(dyn std::error::Error + 'static),
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        let formatter = JsonFormatter::new(
            error,
            self.backtrace.as_ref(),
            self.spantrace.as_ref(),
        );
        match serde_json::to_string(&formatter) {
            Ok(json) => write!(f, "{}", json),
            Err(formatter_error) => write!(
                f,
                "JSON formatting failed with error \"{formatter_error}\", when trying to format error \"{error}\"",
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_formatter_no_backtrace() {
        let error = anyhow::anyhow!("root cause")
            .context("context 0")
            .context("context 1");

        let formatter = JsonFormatter::new(error.as_ref(), None, None);

        assert_eq!(
            formatter.error_chain,
            vec![
                (0, "context 1".to_string()),
                (1, "context 0".to_string()),
                (2, "root cause".to_string()),
            ]
        );

        let json = serde_json::to_string(&formatter).unwrap();
        assert_eq!(
            json,
            "{\"error_chain\":[[0,\"context 1\"],[1,\"context 0\"],[2,\"root cause\"]]}"
        );
    }

    #[test]
    fn test_formatter_with_backtrace() {
        let error = anyhow::anyhow!("Some error");

        let backtrace = backtrace::Backtrace::new();

        let formatter =
            JsonFormatter::new(error.as_ref(), Some(&backtrace), None);

        let json = serde_json::to_string(&formatter).unwrap();

        // Parse the JSON back to verify roundtrip
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Count symbols in original backtrace
        let original_symbol_count: usize = backtrace
            .frames()
            .iter()
            .flat_map(|frame| frame.symbols().iter())
            .count();

        // Verify backtrace array length matches
        let json_backtrace = parsed["backtrace"].as_array().unwrap();
        assert_eq!(
            json_backtrace.len(),
            original_symbol_count,
            "Backtrace symbol count mismatch: JSON has {}, original has {}",
            json_backtrace.len(),
            original_symbol_count
        );

        // Verify symbols match
        let original_symbols: Vec<_> = backtrace
            .frames()
            .iter()
            .flat_map(|frame| frame.symbols().iter())
            .collect();

        for (i, (json_entry, original_symbol)) in json_backtrace
            .iter()
            .zip(original_symbols.iter())
            .enumerate()
        {
            // JSON structure is [index, {function, file, line}]
            let json_idx = json_entry[0].as_u64().unwrap() as usize;
            let json_symbol = &json_entry[1];

            assert_eq!(json_idx, i, "Symbol index mismatch at position {}", i);

            // Compare function name
            let expected_function =
                original_symbol.name().map(|n| n.to_string());
            let json_function =
                json_symbol["function"].as_str().map(|s| s.to_string());
            assert_eq!(
                json_function, expected_function,
                "Function name mismatch at symbol {}",
                i
            );

            // Compare file name
            let expected_file =
                original_symbol.filename().map(|f| f.display().to_string());
            let json_file = json_symbol["file"].as_str().map(|s| s.to_string());
            assert_eq!(
                json_file, expected_file,
                "File name mismatch at symbol {}",
                i
            );

            // Compare line number
            let expected_line = original_symbol.lineno();
            let json_line = json_symbol["line"].as_u64().map(|l| l as u32);
            assert_eq!(
                json_line, expected_line,
                "Line number mismatch at symbol {}",
                i
            );
        }
    }

    #[test]
    fn test_formatter_with_spantrace() {
        use tracing_error::{ErrorLayer, SpanTraceStatus};
        use tracing_subscriber::prelude::*;

        // Install subscriber with ErrorLayer - required for SpanTrace to capture
        let subscriber =
            tracing_subscriber::registry().with(ErrorLayer::default());
        let _guard = tracing::subscriber::set_default(subscriber);

        // Create nested spans with fields
        let outer_span =
            tracing::info_span!("outer_span", user_id = 42, action = "test");
        let _outer_enter = outer_span.enter();

        let inner_span =
            tracing::info_span!("inner_span", request_id = "abc-123");
        let _inner_enter = inner_span.enter();

        let spantrace = SpanTrace::capture();
        assert_eq!(
            spantrace.status(),
            SpanTraceStatus::CAPTURED,
            "SpanTrace should be captured"
        );

        let error = anyhow::anyhow!("test error");
        let formatter =
            JsonFormatter::new(error.as_ref(), None, Some(&spantrace));

        let json = serde_json::to_string(&formatter).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        let mut original_spans = Vec::new();
        spantrace.with_spans(|metadata, fields| {
            original_spans.push((metadata, fields.to_string()));
            true
        });

        // Verify spantrace array exists and has correct length
        let json_spantrace = parsed["spantrace"]
            .as_array()
            .expect("spantrace should be an array");
        assert_eq!(
            json_spantrace.len(),
            original_spans.len(),
            "Spantrace span count mismatch: JSON has {}, original has {}",
            json_spantrace.len(),
            original_spans.len()
        );

        // Verify each span frame matches
        for (i, ((metadata, fields), json_entry)) in
            original_spans.iter().zip(json_spantrace.iter()).enumerate()
        {
            // JSON structure is [index, {full_name, file, line, fields}]
            let json_idx = json_entry[0].as_u64().unwrap() as usize;
            let json_frame = &json_entry[1];

            assert_eq!(json_idx, i, "Span index mismatch at position {}", i);

            // Compare full_name (target::name)
            let expected_full_name =
                format!("{}::{}", metadata.target(), metadata.name());
            let json_full_name = json_frame["full_name"].as_str().unwrap();
            assert_eq!(
                json_full_name, expected_full_name,
                "Full name mismatch at span {}",
                i
            );

            // Compare file
            let expected_file = metadata.file().map(|s| s.to_string());
            let json_file = json_frame["file"].as_str().map(|s| s.to_string());
            assert_eq!(json_file, expected_file, "File mismatch at span {}", i);

            // Compare line (should be present for spans created with macros)
            let expected_line = metadata.line();
            let json_line = json_frame["line"].as_u64().map(|l| l as u32);
            assert_eq!(json_line, expected_line, "Line mismatch at span {}", i);

            // Compare fields
            let expected_fields = if fields.is_empty() {
                None
            } else {
                Some(fields.clone())
            };
            let json_fields =
                json_frame["fields"].as_str().map(|s| s.to_string());
            assert_eq!(
                json_fields, expected_fields,
                "Fields mismatch at span {}",
                i
            );
        }

        // Inner span should be first (most recent)
        let first_span = &json_spantrace[0][1];
        assert!(
            first_span["full_name"]
                .as_str()
                .unwrap()
                .contains("inner_span"),
            "First span should be inner_span"
        );
        assert!(
            first_span["fields"]
                .as_str()
                .unwrap()
                .contains("request_id"),
            "Inner span should have request_id field"
        );

        // Outer span should be second
        let second_span = &json_spantrace[1][1];
        assert!(
            second_span["full_name"]
                .as_str()
                .unwrap()
                .contains("outer_span"),
            "Second span should be outer_span"
        );
        assert!(
            second_span["fields"].as_str().unwrap().contains("user_id"),
            "Outer span should have user_id field"
        );
    }
}
