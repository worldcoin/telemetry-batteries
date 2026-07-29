use axum::{Router, routing::get};
use opentelemetry::Context;
use opentelemetry::propagation::text_map_propagator::FieldIter;
use opentelemetry::propagation::{Extractor, Injector, TextMapPropagator};
use telemetry_batteries::tracing::reqwest::RequestBuilderExt;

const TRACE_HEADER: &str = "x-test-trace";
const TRACE_VALUE: &str = "current-trace";

#[derive(Debug)]
struct TestPropagator {
    fields: Vec<String>,
}

impl TestPropagator {
    fn new() -> Self {
        Self {
            fields: vec![TRACE_HEADER.to_owned()],
        }
    }
}

impl TextMapPropagator for TestPropagator {
    fn inject_context(&self, _context: &Context, injector: &mut dyn Injector) {
        injector.set(TRACE_HEADER, TRACE_VALUE.to_owned());
    }

    fn extract_with_context(
        &self,
        context: &Context,
        _extractor: &dyn Extractor,
    ) -> Context {
        context.clone()
    }

    fn fields(&self) -> FieldIter<'_> {
        FieldIter::new(&self.fields)
    }
}

#[tokio::test]
async fn outgoing_reqwest_requests_receive_trace_context() -> eyre::Result<()> {
    opentelemetry::global::set_text_map_propagator(TestPropagator::new());

    let app = Router::new().route(
        "/",
        get(|headers: http::HeaderMap| async move {
            headers
                .get(TRACE_HEADER)
                .and_then(|value| value.to_str().ok())
                .unwrap_or_default()
                .to_owned()
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let server = tokio::spawn(axum::serve(listener, app).into_future());
    let url = format!("http://{address}");

    let trace_header = reqwest::Client::new()
        .get(&url)
        .header(TRACE_HEADER, "stale-trace")
        .inject_trace_context()
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    assert_eq!(trace_header, TRACE_VALUE);

    #[cfg(feature = "reqwest-middleware")]
    {
        use telemetry_batteries::tracing::reqwest::middleware::TraceContextMiddleware;

        let client =
            reqwest_middleware::ClientBuilder::new(reqwest::Client::new())
                .with(TraceContextMiddleware::new())
                .build();
        let trace_header = client
            .get(&url)
            .header(TRACE_HEADER, "stale-trace")
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        assert_eq!(trace_header, TRACE_VALUE);
    }

    server.abort();
    Ok(())
}
