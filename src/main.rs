use anyhow::Result;
use deliverytracker::api;

#[async_std::main]
async fn main() -> Result<()> {
    api::start_api_server().await?;

    Ok(())
}
