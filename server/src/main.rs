#![warn(clippy::all)]

use std::net::SocketAddr;

use log::info;
use server::{tracker::tracker_server::TrackerServer, DeliveryTracker};
use tokio::runtime::Runtime;
use tonic::transport::Server;
use dotenv::dotenv;

fn main() {
    dotenv().ok();
    pretty_env_logger::init();

    let bind_address = std::env::var("BIND_ADDR")
        .ok()
        .and_then(|addr| addr.parse().ok())
        .expect("cannot find bind address from BIND_ADDR");

    Runtime::new().unwrap().block_on(run(bind_address));
}

async fn run(address: SocketAddr) {
    info!("Try running server...");
    Server::builder()
        .add_service(TrackerServer::new(DeliveryTracker::default()))
        .serve(address)
        .await
        .expect("Can't run server");
}
