use tracing::Subscriber;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::{registry::LookupSpan, EnvFilter, Layer};

pub fn stdout_layer<S>() -> impl Layer<S>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout)
        .pretty()
        .with_span_events(FmtSpan::NEW)
        .with_target(false)
        .with_line_number(true)
        .with_file(true)
        .with_filter(EnvFilter::from_default_env())
}
