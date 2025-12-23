use std::{env, iter::successors};

use serde::{Deserialize, Serialize};

/// Convenience trait to get the backtrace from an eyre::Report in case json_eyre is installed.
pub trait BacktraceExt {
    fn backtrace(&self) -> Option<&backtrace::Backtrace>;
}

impl BacktraceExt for eyre::Report {
    fn backtrace(&self) -> Option<&backtrace::Backtrace> {
        self.handler()
            .downcast_ref::<Handler>()
            .and_then(|handler| handler.backtrace.as_ref())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BacktraceSymbol {
    // Note: None will be serialized as 'null'
    pub function: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
}

impl BacktraceSymbol {
    pub fn from_symbol(symbol: &backtrace::BacktraceSymbol) -> Self {
        Self {
            function: symbol.name().map(|name| name.to_string()),
            file: symbol
                .filename()
                .map(|filename| filename.display().to_string()),
            line: symbol.lineno(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct JsonFormatter {
    /// The error chain of the error. The last element is the root cause.
    /// We include the index for convenience.
    pub error_chain: Vec<(u32, String)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backtrace: Option<Vec<(u32, BacktraceSymbol)>>,
}

impl JsonFormatter {
    pub fn new(
        err: &(dyn std::error::Error + 'static),
        backtrace: Option<&backtrace::Backtrace>,
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

        Self {
            error_chain,
            backtrace,
        }
    }
}

#[derive(Debug)]
pub struct Handler {
    backtrace: Option<backtrace::Backtrace>,
}

impl Handler {
    pub fn new(with_default_backtrace: bool) -> Self {
        let with_backtrace = env::var("RUST_LIB_BACKTRACE")
            .or_else(|_| env::var("RUST_BACKTRACE"))
            .map(|val| val != "0")
            .unwrap_or(with_default_backtrace);

        let backtrace = if with_backtrace {
            Some(backtrace::Backtrace::new())
        } else {
            None
        };

        Self { backtrace }
    }
}

impl eyre::EyreHandler for Handler {
    fn debug(
        &self,
        error: &(dyn std::error::Error + 'static),
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        let formatter = JsonFormatter::new(error, self.backtrace.as_ref());
        match serde_json::to_string(&formatter) {
            Ok(json) => write!(f, "{}", json),
            Err(formatter_error) => write!(
                f,
                "JSON formatting failed with error \"{formatter_error}\", when trying to format error \"{error}\"",
            ),
        }
    }
}

pub fn install_hook(with_default_backtrace: bool) -> eyre::Result<()> {
    eyre::set_hook(Box::new(move |_| {
        Box::new(Handler::new(with_default_backtrace))
    }))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_formatter_no_backtrace() {
        let error = anyhow::anyhow!("root cause")
            .context("context 0")
            .context("context 1");

        let formatter = JsonFormatter::new(error.as_ref(), None);

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
        let error = anyhow::anyhow!("root cause")
            .context("context 0")
            .context("context 1");

        let backtrace = backtrace::Backtrace::new();

        let formatter = JsonFormatter::new(error.as_ref(), Some(&backtrace));

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
}
