use telemetry_batteries::TopLevelResultExt;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let _guard = telemetry_batteries::init()?;

    let _ = tokio::spawn(async { panic!("this is a panic in a tokio task!") })
        .await;

    run().await.panic_on_top_level_error();

    Ok(())
}

async fn run() -> eyre::Result<()> {
    eyre::bail!("top level error!");
}
