use tracing::{Level, Subscriber};
use tracing_subscriber::{fmt, registry::LookupSpan, EnvFilter, Layer};

pub mod datadog;

pub struct StdoutLayer;

impl StdoutLayer {
    pub fn layer<S>(level: Level) -> impl Layer<S>
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        let filter = EnvFilter::from_default_env().add_directive(level.into());
        let fmt_layer = fmt::layer().with_target(false).with_level(true);

        filter.and_then(fmt_layer)
    }
}
