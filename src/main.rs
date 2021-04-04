use anyhow::Result;

pub mod api;
pub mod couriers;
pub mod graphql;
pub mod macros;
pub mod status_struct;

#[async_std::main]
async fn main() -> Result<()> {
    api::start_api_server().await?;

    Ok(())
}
