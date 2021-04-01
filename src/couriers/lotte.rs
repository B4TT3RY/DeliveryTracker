use anyhow::{anyhow, Result};
use async_trait::async_trait;
use regex::Regex;
use scraper::{Html, Selector};

use crate::{
    couriers::courier::Courier, delivery_status::DeliveryStatus, get_html_string,
    tracking_status::TrackingStatus,
};

pub struct Lotte {
    pub tracking_number: String,
}

#[async_trait]
impl Courier for Lotte {
    fn get_url() -> &'static str {
        "https://www.lotteglogis.com/home/reservation/tracking/linkView"
    }

    fn get_id() -> &'static str {
        "kr.lotte"
    }

    fn get_name() -> &'static str {
        "롯데택배"
    }

    async fn validate(&self) -> Result<&Self> {
        if !Regex::new(r#"^(\d{10}|\d{12}|\d{13})$"#)?.is_match(&self.tracking_number) {
            return Err(anyhow!(
                "운송장번호 10자리, 12자리 또는 13자리를 입력해주세요."
            ));
        }
        Ok(self)
    }

    async fn track(&self) -> Result<DeliveryStatus> {
        let response = surf::post(Self::get_url())
            .body(format!("InvNo={}", self.tracking_number))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.90 Safari/537.36")
            .recv_string()
            .await
            .unwrap();
        let document = Html::parse_document(&response);

        if document
            .select(&Selector::parse(".empty").unwrap())
            .next()
            .is_some()
        {
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

        let mut tracks: Vec<TrackingStatus> = Vec::new();
        let selector = Selector::parse("div.contArea > table:nth-child(4) > tbody > tr").unwrap();
        for element in document.select(&selector) {
            tracks.push(TrackingStatus {
                time: get_html_string!(element, "td:nth-child(2)"),
                location: get_html_string!(element, "td:nth-child(3)"),
                status: get_html_string!(element, "td:nth-child(1)"),
                message: Some(get_html_string!(element, "td:nth-child(4)")),
            });
        }

        tracks.reverse();

        Ok(DeliveryStatus {
            id: Self::get_id().to_string(),
            name: Self::get_name().to_string(),
            tracking_number: self.tracking_number.clone(),
            sender: None,
            receiver: None,
            product: None,
            tracks: Some(tracks),
        })
    }
}
