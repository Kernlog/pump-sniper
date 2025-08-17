use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    pump_sniper::run().await
}
