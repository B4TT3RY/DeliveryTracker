use anyhow::{anyhow, Result};
use async_trait::async_trait;
use regex::Regex;
use scraper::{Html, Selector};

use crate::{couriers::courier::Courier, delivery_status::DeliveryStatus, tracking_status::TrackingStatus, get_html_string};

pub struct ILogen {
    pub tracking_number: String,
}

#[async_trait]
impl Courier for ILogen {
    fn get_url() -> &'static str {
        "https://www.ilogen.com/web/personal/trace/"
    }

    fn get_id() -> &'static str {
        "kr.ilogen"
    }

    fn get_name() -> &'static str {
        "로젠택배"
    }

    async fn validate(&self) -> Result<&Self> {
        if !Regex::new(r#"^(\d{11})$"#)?.is_match(&self.tracking_number) {
            return Err(anyhow!("운송장번호 11자리를 입력해주세요."));
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

        if document.select(&Selector::parse(".empty").unwrap()).next().is_some() {
            return Ok(DeliveryStatus {
                id: Self::get_id().to_string(),
                name: Self::get_name().to_string(),
                tracking_number: self.tracking_number.clone(),
                sender: None,
                receiver: None,
                product: None,
                tracks: None,
            });
        }

        let product = get_html_string!(document, "table.horizon.pdInfo > tbody > tr:nth-child(1) > td:nth-child(4)");
        let sender = get_html_string!(document, "table.horizon.pdInfo > tbody > tr:nth-child(4) > td:nth-child(2)");
        let receiver = get_html_string!(document, "table.horizon.pdInfo > tbody > tr:nth-child(4) > td:nth-child(4)");
        
        let mut tracks:Vec<TrackingStatus> = Vec::new();
        let selector = Selector::parse("table.data.tkInfo > tbody > tr").unwrap();
        for element in document.select(&selector) {
            tracks.push(
                TrackingStatus {
                    time: get_html_string!(element, "td:nth-child(1)"),
                    location: get_html_string!(element, "td:nth-child(2)"),
                    status: get_html_string!(element, "td:nth-child(3)"),
                    message: Some(get_html_string!(element, "td:nth-child(4)")),
                    
                }
            );
        };

        Ok(DeliveryStatus {
            id: Self::get_id().to_string(),
            name: Self::get_name().to_string(),
            tracking_number: self.tracking_number.clone(),
            sender: Some(sender),
            receiver: Some(receiver),
            product: Some(product),
            tracks: Some(tracks),
        })
    }
}
