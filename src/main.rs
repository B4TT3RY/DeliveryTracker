use couriers::{cjlogistics::CJLogistics, courier::Courier, epost::EPost};

mod couriers;
mod delivery_status;
mod tracking_status;

#[async_std::main]
async fn main() {
    println!("Hello, world!");
    let courier = "cjlogistics";
    let tracking_number = "638991190880".to_string();
    let delivery_status = match courier {
        "cjlogistics" => CJLogistics::track(tracking_number).await,
        "epost" => EPost::track(tracking_number).await,
        _ => return
    }.unwrap();    
    println!("{:?}", &delivery_status);
}
