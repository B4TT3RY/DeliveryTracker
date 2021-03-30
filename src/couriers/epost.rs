use anyhow::Result;
use async_trait::async_trait;
use scraper::{Html, Selector};

use crate::{couriers::courier::Courier, delivery_status::DeliveryStatus, tracking_status::TrackingStatus, get_html_string};

pub struct EPost {}

#[async_trait]
impl Courier for EPost {
    fn get_url() -> &'static str {
        "https://service.epost.go.kr/trace.RetrieveDomRigiTraceList.comm"
    }

    fn get_id() -> &'static str {
        "kr.epost"
    }

    fn get_name() -> &'static str {
        "우체국"
    }

    async fn track(tracking_number: String) -> Result<DeliveryStatus> {
        let response = surf::post(EPost::get_url())
            .body(format!("sid1={}&displayHeader=N", tracking_number))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .recv_string()
            .await
            .unwrap();
        let document = Html::parse_document(&response);

        let tracking_number = get_html_string!(document, "#print > table > tbody > tr > th");
        let sender = get_html_string!(document, "#print > table > tbody > tr > td:nth-child(2)");
        let receiver = get_html_string!(document, "#print > table > tbody > tr > td:nth-child(3)");
        
        let mut tracks:Vec<TrackingStatus> = Vec::new();
        let selector = Selector::parse("#processTable > tbody > tr").unwrap();
        for element in document.select(&selector) {
            tracks.push(
                TrackingStatus {
                    time: format!("{} {}", get_html_string!(element, "td:nth-child(1)"), get_html_string!(element, "td:nth-child(2)")),
                    location: get_html_string!(element, "td > a > span"),
                    status: get_html_string!(element, "td:nth-child(4)"),
                    message: "".to_string(),
                    
                }
            );
        };

        Ok(DeliveryStatus {
            id: EPost::get_id().to_string(),
            name: EPost::get_name().to_string(),
            tracking_number,
            sender,
            receiver,
            product: None,
            tracks,
        })
    }
}
