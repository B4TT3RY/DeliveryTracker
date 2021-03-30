use couriers::{cjlogistics::CJLogistics, courier::Courier, epost::EPost};

mod couriers;
mod delivery_status;
mod macros;
mod tracking_status;

#[async_std::main]
async fn main() {
    println!("Hello, world!");
    // let courier = "kr.cjlogistics";
    // let tracking_number = "638991190880".to_string();
    let courier = "kr.epost";
    let tracking_number = "6078616932735".to_string();
    let delivery_status = match courier {
        "kr.cjlogistics" => CJLogistics::track(tracking_number),
        "kr.epost" => EPost::track(tracking_number),
        _ => return,
    }
    .await
    .unwrap();
    println!("{:#?}", &delivery_status);
}
