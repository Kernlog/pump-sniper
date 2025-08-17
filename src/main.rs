use anyhow::Result;
use pump_sniper;

#[tokio::main]
async fn main() -> Result<()> {
    pump_sniper::run().await
}
