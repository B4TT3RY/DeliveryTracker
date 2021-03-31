use anyhow::{anyhow, Result};
use async_trait::async_trait;
use regex::Regex;
use scraper::{Html, Selector};

use crate::{couriers::courier::Courier, delivery_status::DeliveryStatus, tracking_status::TrackingStatus, get_html_string};

pub struct EPost {
    pub tracking_number: String,
}

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

    async fn validate(&self) -> Result<&Self> {
        if !Regex::new(r#"^(\d{13})$"#)?.is_match(&self.tracking_number) {
            return Err(anyhow!("운송장번호 13자리를 입력해주세요."));
        }
        Ok(self)
    }

    async fn track(&self) -> Result<DeliveryStatus> {
        let response = surf::get(format!("{}{}", Self::get_url(), &self.tracking_number))
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.90 Safari/537.36")
            .recv_string()
            .await
            .unwrap();
        let document = Html::parse_document(&response);

        let tracking_number = get_html_string!(document, "#print > table > tbody > tr > th");
        
        if document.select(&Selector::parse("#print > table > tbody > tr:nth-child(2) > td").unwrap()).next().is_some() {
            return Ok(DeliveryStatus {
                id: Self::get_id().to_string(),
                name: Self::get_name().to_string(),
                tracking_number,
                sender: None,
                receiver: None,
                product: None,
                tracks: None,
            });
        }
        let sender = get_html_string!(document, "#print > table > tbody > tr > td:nth-child(2)");
        let receiver = get_html_string!(document, "#print > table > tbody > tr > td:nth-child(3)");
        
        let mut tracks:Vec<TrackingStatus> = Vec::new();
        let selector = Selector::parse("#processTable > tbody > tr").unwrap();
        for element in document.select(&selector) {
            tracks.push(
                TrackingStatus {
                    time: format!("{} {}", get_html_string!(element, "td:nth-child(1)"), get_html_string!(element, "td:nth-child(2)")),
                    location: get_html_string!(element, "td:nth-child(3)"),
                    status: get_html_string!(element, "td:nth-child(4)"),
                    message: None,
                    
                }
            );
        };

        Ok(DeliveryStatus {
            id: Self::get_id().to_string(),
            name: Self::get_name().to_string(),
            tracking_number,
            sender: Some(sender),
            receiver: Some(receiver),
            product: None,
            tracks: Some(tracks),
        })
    }
}
