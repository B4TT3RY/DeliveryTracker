#![warn(clippy::all)]

use std::net::SocketAddr;

use deliverytracker::{tracker::tracker_server::TrackerServer, DeliveryTracker};
use log::info;
use tokio::runtime::Runtime;
use tonic::transport::Server;

fn main() {
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
