#![cfg(feature = "axum")]

use std::sync::{Arc, Mutex};

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
    routing::get,
};
use opentelemetry::trace::{SpanKind, TracerProvider as _};
use opentelemetry_sdk::{
    error::OTelSdkResult,
    trace::{SdkTracerProvider, SpanData, SpanExporter},
};
use telemetry_batteries::tracing::middleware::TraceLayer;
use tower::ServiceExt;
use tracing::instrument::WithSubscriber;
use tracing_subscriber::prelude::*;

#[derive(Clone, Debug, Default)]
struct TestExporter(Arc<Mutex<Vec<SpanData>>>);

impl SpanExporter for TestExporter {
    async fn export(&self, mut batch: Vec<SpanData>) -> OTelSdkResult {
        self.0.lock().unwrap().append(&mut batch);
        Ok(())
    }
}

#[tokio::test]
async fn axum_spans_use_the_matched_route_template() {
    let exporter = TestExporter::default();
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(exporter.clone())
        .build();
    let tracer = provider.tracer("axum-route-test");
    let subscriber = tracing_subscriber::registry()
        .with(tracing_opentelemetry::layer().with_tracer(tracer));

    let app = Router::new()
        .route("/ready", get(StatusCode::NO_CONTENT))
        .route("/users/{user_id}", get(StatusCode::NO_CONTENT))
        .nest(
            "/v1",
            Router::new()
                .route("/users/{user_id}", get(StatusCode::NO_CONTENT)),
        )
        .layer(TraceLayer::new_for_axum());

    let cases = [
        ("/ready", Some("/ready"), StatusCode::NO_CONTENT),
        (
            "/users/1234",
            Some("/users/{user_id}"),
            StatusCode::NO_CONTENT,
        ),
        (
            "/users/5678",
            Some("/users/{user_id}"),
            StatusCode::NO_CONTENT,
        ),
        (
            "/v1/users/1234",
            Some("/v1/users/{user_id}"),
            StatusCode::NO_CONTENT,
        ),
        ("/unknown/1234", None, StatusCode::NOT_FOUND),
    ];
    async move {
        for (path, _, status) in cases {
            let response = app
                .clone()
                .oneshot(
                    Request::builder().uri(path).body(Body::empty()).unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), status);
        }
    }
    .with_subscriber(subscriber)
    .await;

    provider.force_flush().unwrap();

    let spans = exporter.0.lock().unwrap();
    assert_eq!(spans.len(), cases.len());
    for (path, route, status) in cases {
        let span = spans
            .iter()
            .find(|span| {
                span.span_kind == SpanKind::Server
                    && attribute(span, "url.path").as_deref() == Some(path)
            })
            .expect("server span was not exported");
        let expected_name = route
            .map_or_else(|| "GET".to_owned(), |route| format!("GET {route}"));
        assert_eq!(span.name, expected_name);
        assert_eq!(attribute(span, "http.route").as_deref(), route);
        assert_eq!(
            attribute(span, "http.status_code"),
            Some(status.as_u16().to_string())
        );
    }
}

fn attribute(span: &SpanData, name: &str) -> Option<String> {
    span.attributes.iter().find_map(|attribute| {
        (attribute.key.as_str() == name)
            .then(|| attribute.value.as_str().into_owned())
    })
}
