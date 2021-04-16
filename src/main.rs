#![warn(clippy::all)]

use anyhow::Result;

#[cfg(feature = "tide")]
use deliverytracker::api;

#[async_std::main]
async fn main() -> Result<()> {
    #[cfg(feature = "tide")]
    api::start_api_server().await?;

    Ok(())
}
