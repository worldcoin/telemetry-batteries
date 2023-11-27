use tracing::Level;
use tracing_subscriber::{fmt, EnvFilter, Layer};

pub mod datadog;

pub struct StdoutLayer;

impl StdoutLayer {
    pub fn new<S>(level: Level) -> impl Layer<S>
    where
        S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
    {
        let filter = EnvFilter::from_default_env().add_directive(level.into());
        let fmt_layer = fmt::layer().with_target(false).with_level(true);

        filter.and_then(fmt_layer)
    }
}
