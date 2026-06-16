use std::io;
use std::sync::{Arc, Mutex};

use axum::{Json, Router, routing::get};
use opentelemetry::Context;
use opentelemetry::trace::{
    SpanContext, SpanId, TraceContextExt, TraceFlags, TraceId, TraceState,
    TracerProvider,
};
use telemetry_batteries::tracing::layers::datadog::DatadogFormat;
use telemetry_batteries::tracing::middleware::TraceLayer;
use telemetry_batteries::tracing_opentelemetry::OpenTelemetrySpanExt;
use telemetry_batteries::{
    LogFormat, TelemetryConfig, TelemetryPreset, init_with_config,
};
use tokio::sync::oneshot;
use tracing_subscriber::Layer;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::prelude::*;

const TRACE_ID: u64 = 7_690_679_301_650_107_577;
const PARENT_SPAN_ID: u64 = 16_133_904_967_205_640_245;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct TraceIds {
    trace_id: String,
    span_id: String,
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn datadog_integration() -> eyre::Result<()> {
    let _guard = init_with_config(TelemetryConfig {
        preset: TelemetryPreset::Datadog,
        service_name: Some("datadog-integration-test".to_owned()),
        log_format: Some(LogFormat::DatadogJson),
        ..TelemetryConfig::default()
    })?;

    let app = Router::new()
        .route("/trace", get(trace_ids))
        .layer(TraceLayer::new())
        .route("/no-trace", get(trace_ids));

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
        .header("x-datadog-parent-id", PARENT_SPAN_ID.to_string())
        .header("x-datadog-sampling-priority", "1")
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    let response: TraceIds = serde_json::from_str(&body)?;

    assert_eq!(response.trace_id, TRACE_ID.to_string());
    // The incoming span id becomes the request span's parent; the request span
    // itself gets a new span id.
    assert_ne!(response.span_id, PARENT_SPAN_ID.to_string());

    let body = reqwest::Client::new()
        .get(format!("http://{addr}/no-trace"))
        .header("x-datadog-trace-id", TRACE_ID.to_string())
        .header("x-datadog-parent-id", PARENT_SPAN_ID.to_string())
        .header("x-datadog-sampling-priority", "1")
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let response: TraceIds = serde_json::from_str(&body)?;

    // no traces or span because TraceLayer is not applied
    assert_eq!(response.trace_id, "0");
    assert_eq!(response.span_id, "0");

    let _ = shutdown_tx.send(());
    server.await??;

    Ok(())
}

#[test]
fn datadog_logs_include_current_trace_and_span_ids() {
    let logs = BufWriter::new();
    let provider =
        opentelemetry_sdk::trace::SdkTracerProvider::builder().build();
    let tracer = provider.tracer("datadog-log-test");
    let otel_layer =
        telemetry_batteries::tracing_opentelemetry::OpenTelemetryLayer::new(
            tracer,
        );
    let format_layer = tracing_subscriber::fmt::Layer::new()
        .json()
        .event_format(DatadogFormat)
        .with_writer(logs.clone());
    let subscriber =
        tracing_subscriber::registry().with(format_layer.and_then(otel_layer));

    let span_id = tracing::subscriber::with_default(subscriber, || {
        let span = tracing::info_span!("log_test_span");
        let parent = SpanContext::new(
            TraceId::from(TRACE_ID as u128),
            SpanId::from(PARENT_SPAN_ID),
            TraceFlags::SAMPLED,
            true,
            TraceState::default(),
        );
        span.set_parent(Context::new().with_remote_span_context(parent))
            .unwrap();

        let _entered = span.enter();
        let (_, span_id) = telemetry_batteries::tracing::extract_span_ids();
        tracing::info!("inside propagated span");
        span_id_to_u64(span_id)
    });

    let log: serde_json::Value = serde_json::from_str(logs.contents().trim())
        .expect("log line is not valid JSON");

    assert_eq!(log["message"], "inside propagated span");
    assert_eq!(log["dd.trace_id"], TRACE_ID.to_string());
    assert_eq!(log["dd.span_id"], span_id.to_string());
}

async fn trace_ids() -> Json<TraceIds> {
    tracing::info!("handling trace request");

    let (trace_id, span_id) = telemetry_batteries::tracing::extract_span_ids();

    Json(TraceIds {
        trace_id: trace_id_to_u128(trace_id).to_string(),
        span_id: span_id_to_u64(span_id).to_string(),
    })
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

fn trace_id_to_u128(trace_id: TraceId) -> u128 {
    u128::from_be_bytes(trace_id.to_bytes())
}

fn span_id_to_u64(span_id: SpanId) -> u64 {
    u64::from_be_bytes(span_id.to_bytes())
}
