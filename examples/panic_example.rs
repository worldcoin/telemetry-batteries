use telemetry_batteries::TopLevelResultExt;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let _guard = telemetry_batteries::init()?;

    run().await.panic_on_top_level_error();

    Ok(())
}

async fn run() -> eyre::Result<()> {
    eyre::bail!("top level error! What will it look like!");
}
