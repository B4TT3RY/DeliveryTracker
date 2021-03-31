use anyhow::Result;

mod couriers;
mod delivery_status;
mod macros;
mod tracking_status;
mod api;

#[async_std::main]
async fn main() -> Result<()> {
    api::start_api_server().await?;

    Ok(())
}
