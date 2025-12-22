use std::iter::successors;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct JsonFormatter {
    /// The error chain of the error. The last element is the root cause.
    /// We include the index for convenience.
    pub error_chain: Vec<(u32, String)>,
}

impl JsonFormatter {
    pub fn from_err(err: &(dyn std::error::Error + 'static)) -> Self {
        let error_chain = successors(Some(err), |e| e.source())
            .enumerate()
            .map(|(i, e)| (i as u32, e.to_string()))
            .collect::<Vec<_>>();

        Self { error_chain }
    }
}

#[derive(Debug)]
pub struct Handler {}

impl eyre::EyreHandler for Handler {
    fn debug(
        &self,
        error: &(dyn std::error::Error + 'static),
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        let formatter = JsonFormatter::from_err(error);
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
    fn test_formatter() {
        let error = anyhow::anyhow!("root cause")
            .context("context 0")
            .context("context 1");

        let formatter = JsonFormatter::from_err(error.as_ref());

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
}
