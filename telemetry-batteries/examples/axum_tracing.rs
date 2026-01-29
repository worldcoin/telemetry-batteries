//! Example demonstrating distributed tracing with axum.
//!
//! This example shows how to use the [`TraceLayer`] middleware to automatically
//! propagate trace context in an axum application.
//!
//! Run with Datadog:
//! ```bash
//! TELEMETRY_PRESET=datadog \
//! TELEMETRY_SERVICE_NAME=axum-tracing-example \
//! cargo run -p telemetry-batteries --example axum_tracing
//! ```
//!
//! Then test with:
//! ```bash
//! # Simple request
//! curl http://localhost:3000/
//!
//! # Request with trace context (simulating upstream service)
//! curl -H "traceparent: 00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01" \
//!      http://localhost:3000/
//! ```

use axum::{routing::get, Router};
use telemetry_batteries::tracing::middleware::TraceLayer;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let _guard = telemetry_batteries::init()?;

    let app = Router::new()
        .route("/", get(root))
        .route("/hello/{name}", get(hello))
        .layer(TraceLayer::new());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("Listening on http://0.0.0.0:3000");

    axum::serve(listener, app).await?;

    Ok(())
}

#[tracing::instrument]
async fn root() -> &'static str {
    tracing::info!("Handling root request");
    "Hello, World!"
}

#[tracing::instrument]
async fn hello(axum::extract::Path(name): axum::extract::Path<String>) -> String {
    tracing::info!(name = %name, "Handling hello request");
    format!("Hello, {name}!")
}
