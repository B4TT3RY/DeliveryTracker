use anyhow::Result;
use async_trait::async_trait;
use scraper::{Html, Selector};

use crate::{couriers::courier::Courier, delivery_status::DeliveryStatus, tracking_status::TrackingStatus};

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
        let document = Html::parse_document(&response);

        let tracking_number = {
            let selector = Selector::parse("table:nth-child(3) > tbody > tr:nth-child(2) > td:nth-child(1) > b").unwrap();
            document.select(&selector)
                .next()
                .unwrap()
                .text()
                .collect::<String>()
                .replace("운송장 번호 : ", "")
        };

        let sender = {
            let selector = Selector::parse("table:nth-child(6) > tbody > tr:nth-child(2) > td:nth-child(1)").unwrap();
            document.select(&selector)
                .next()
                .unwrap()
                .text()
                .collect::<String>()
                .trim()
                .to_string()
        };

        let receiver = {
            let selector = Selector::parse("table:nth-child(6) > tbody > tr:nth-child(2) > td:nth-child(2)").unwrap();
            document.select(&selector)
                .next()
                .unwrap()
                .text()
                .collect::<String>()
                .trim()
                .to_string()
        };
        
        Ok(DeliveryStatus {
            tracking_number,
            sender,
            receiver,
            product: "".to_string(),
            last_track: TrackingStatus {
                time: "".to_string(),
                location: "".to_string(),
                status: "".to_string(),
                message: "".to_string(),
            },
        })
    }
}
