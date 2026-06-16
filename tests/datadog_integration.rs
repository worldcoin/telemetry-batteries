use axum::{Json, Router, routing::get};
use opentelemetry::trace::{SpanId, TraceId, TracerProvider};
use opentelemetry_datadog::DatadogPropagator;
use opentelemetry_sdk::{
    error::OTelSdkResult,
    trace::{SdkTracerProvider, SpanData, SpanExporter},
};
use std::sync::{Arc, Mutex};
use telemetry_batteries::tracing::middleware::TraceLayer;
use tokio::sync::oneshot;
use tracing_subscriber::prelude::*;

const TRACE_ID: u64 = 7_690_679_301_650_107_577;
const SPAN_ID: u64 = 16_133_904_967_205_640_245;
const REQUEST_SPAN_NAME: &str = "datadog_integration_request";

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct TraceIds {
    trace_id: String,
    span_id: String,
}

#[derive(Clone, Debug, Default)]
struct RecordingExporter {
    spans: Arc<Mutex<Vec<SpanData>>>,
}

impl RecordingExporter {
    fn spans(&self) -> Vec<SpanData> {
        self.spans.lock().unwrap().clone()
    }
}

impl SpanExporter for RecordingExporter {
    async fn export(&self, batch: Vec<SpanData>) -> OTelSdkResult {
        self.spans.lock().unwrap().extend(batch);
        Ok(())
    }
}

#[tokio::test(flavor = "current_thread")]
async fn datadog_integration() -> eyre::Result<()> {
    opentelemetry::global::set_text_map_propagator(DatadogPropagator::new());

    let exporter = RecordingExporter::default();
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(exporter.clone())
        .build();
    let tracer = provider.tracer("datadog-integration-test");
    let otel_layer =
        telemetry_batteries::tracing_opentelemetry::OpenTelemetryLayer::new(
            tracer,
        );
    let subscriber = tracing_subscriber::registry().with(otel_layer);
    let _subscriber_guard = tracing::subscriber::set_default(subscriber);

    let app = Router::new()
        .route("/trace", get(trace_ids))
        .layer(TraceLayer::new().with_make_span(make_span));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let server = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            })
            .await
    });

    let body = reqwest::Client::new()
        .get(format!("http://{addr}/trace"))
        .header("x-datadog-trace-id", TRACE_ID.to_string())
        .header("x-datadog-parent-id", SPAN_ID.to_string())
        .header("x-datadog-sampling-priority", "1")
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let response: TraceIds = serde_json::from_str(&body)?;

    let _ = shutdown_tx.send(());
    server.await??;

    provider.force_flush()?;
    let spans = exporter.spans();
    let request_span = spans
        .iter()
        .find(|span| span.name.as_ref() == REQUEST_SPAN_NAME)
        .expect("request span was not exported");

    assert_eq!(response.trace_id, TRACE_ID.to_string());
    assert_eq!(
        response.span_id,
        span_id_to_u64(request_span.span_context.span_id()).to_string()
    );
    assert_eq!(
        trace_id_to_u128(request_span.span_context.trace_id()),
        TRACE_ID as u128
    );
    assert_eq!(span_id_to_u64(request_span.parent_span_id), SPAN_ID);
    assert!(request_span.parent_span_is_remote);

    Ok(())
}

async fn trace_ids() -> Json<TraceIds> {
    let (trace_id, span_id) = telemetry_batteries::tracing::extract_span_ids();

    Json(TraceIds {
        trace_id: trace_id_to_u128(trace_id).to_string(),
        span_id: span_id_to_u64(span_id).to_string(),
    })
}

fn make_span(_: &http::Request<()>) -> tracing::Span {
    tracing::info_span!("datadog_integration_request")
}

fn trace_id_to_u128(trace_id: TraceId) -> u128 {
    u128::from_be_bytes(trace_id.to_bytes())
}

fn span_id_to_u64(span_id: SpanId) -> u64 {
    u64::from_be_bytes(span_id.to_bytes())
}
