use anyhow::Result;
use couriers::courier::CourierType;

mod couriers;
mod delivery_status;
mod macros;
mod tracking_status;
mod api;

#[async_std::main]
async fn main() -> Result<()> {
    api::start_api_server().await?;
    CourierType::get_courier_by_id("kr.cjlogistics".to_string())?;
    Ok(())
}
