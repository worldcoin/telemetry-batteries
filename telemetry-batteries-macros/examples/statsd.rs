use telemetry_batteries_macros::statsd;

#[statsd(
    host = "localhost",
    port = 8125,
    buffer_size = 1024,
    queue_size = 100,
    prefix = "my_service"
)]
#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    metrics::counter!("my_counter");
    Ok(())
}
