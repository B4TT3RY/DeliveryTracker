use anyhow::Result;

mod api;
mod couriers;
mod graphql;
mod macros;
mod status_struct;

#[async_std::main]
async fn main() -> Result<()> {
    api::start_api_server().await?;

    Ok(())
}
