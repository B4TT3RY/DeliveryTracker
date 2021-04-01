use anyhow::Result;

mod api;
mod couriers;
mod delivery_status;
mod macros;
mod tracking_status;

#[async_std::main]
async fn main() -> Result<()> {
    api::start_api_server().await?;

    Ok(())
}
