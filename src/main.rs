#![warn(clippy::all)]

use couriers::cjlogistics::Cjlogistics;
use structs::Courier;

mod couriers;
mod structs;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    Ok(())
}
