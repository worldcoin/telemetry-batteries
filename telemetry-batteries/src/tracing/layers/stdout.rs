use tracing::Subscriber;
use tracing_subscriber::{EnvFilter, Layer, registry::LookupSpan};

pub fn stdout_layer<S>() -> impl Layer<S>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout)
        .pretty()
        .with_target(false)
        .with_line_number(true)
        .with_file(true)
        .with_filter(EnvFilter::from_default_env())
}
