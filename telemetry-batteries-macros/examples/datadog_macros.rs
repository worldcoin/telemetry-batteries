use telemetry_batteries_macros::datadog;

#[datadog(service_name = "datadog-example")]
#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    tracing::info!("foo");
    tracing::info!("bar");
    Ok(())
}
