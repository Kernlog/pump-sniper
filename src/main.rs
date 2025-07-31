use anyhow::Result;
use asuga_trial;

#[tokio::main]
async fn main() -> Result<()> {
    asuga_trial::run().await
}
