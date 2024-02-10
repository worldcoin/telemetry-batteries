use telemetry_batteries_macros::datadog;

#[datadog(service_name = "datadog-example")]
pub fn main() -> eyre::Result<()> {
    tracing::info!("foo");

    Ok(())
}
