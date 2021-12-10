#![warn(clippy::all)]

use tokio::runtime::Runtime;

mod telegram;

fn main() {
    Runtime::new().unwrap().block_on(run());
}

async fn run() {
    
}
