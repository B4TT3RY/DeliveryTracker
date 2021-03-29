use anyhow::Result;
use async_trait::async_trait;

use crate::{couriers::courier::Courier, delivery_status::DeliveryStatus};

pub struct CJLogistics {}

#[async_trait]
impl Courier for CJLogistics {
    fn get_url() -> &'static str {
        "http://nexs.cjgls.com/web/info.jsp?slipno="
    }

    fn get_name() -> &'static str {
        "CJ대한통운"
    }

    async fn track(tracking_number: String) -> Result<DeliveryStatus> {
        let request_url = format!("{}{}", CJLogistics::get_url(), tracking_number);
        let response = surf::get(request_url)
            .recv_string()
            .await
            .unwrap();
        println!("{}", response);
        todo!()
    }
}
