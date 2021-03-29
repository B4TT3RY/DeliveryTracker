use anyhow::Result;
use async_trait::async_trait;

use crate::{couriers::courier::Courier, delivery_status::DeliveryStatus};

pub struct EPost {}

#[async_trait]
impl Courier for EPost {
    fn get_url() -> &'static str {
        "http://nexs.cjgls.com/web/info.jsp?slipno="
    }

    fn get_name() -> &'static str {
        "우체국"
    }

    async fn track(tracking_number: String) -> Result<DeliveryStatus> {
        let request_url = format!("{}{}", EPost::get_url(), tracking_number);
        let response = surf::get(request_url)
            .recv_string()
            .await
            .unwrap();
        println!("{}", response);
        todo!()
    }
}
