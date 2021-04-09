use anyhow::{anyhow, Result};
use async_trait::async_trait;
use nipper::Document;
use regex::Regex;

use crate::{
    couriers::courier::{Courier, CourierType},
    get_html_string,
    status_struct::{DeliveryStatus, StateType, TrackingStatus},
};

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
        let response = surf::post(Self::get_url())
            .body(format!("sid1={}&displayHeader=N", self.tracking_number))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.90 Safari/537.36")
            .recv_string()
            .await
            .map_err(|err| anyhow!(err))?;
        let document = Document::from(&response);

        if document.select("#print > table > tbody > tr:nth-child(2) > td").exists() {
            return Err(anyhow!(
                "{} {} 운송장 번호로 조회된 결과가 없습니다.",
                Self::get_name(),
                &self.tracking_number
            ));
        }

        let tracking_number = get_html_string!(document, "#print > table > tbody > tr > th");
        let sender = get_html_string!(document, "#print > table > tbody > tr > td:nth-child(2)");
        let receiver = get_html_string!(document, "#print > table > tbody > tr > td:nth-child(3)");

        let mut tracks: Vec<TrackingStatus> = Vec::new();

        for element in document.select("#processTable > tbody > tr").iter() {
            let status = get_html_string!(element, "td:nth-child(4)");
            tracks.push(TrackingStatus {
                state: StateType::to_type(
                    CourierType::get_courier(Self::get_id().to_string(), None)?,
                    &status,
                ),
                time: format!(
                    "{} {}",
                    get_html_string!(element, "td:nth-child(1)"),
                    get_html_string!(element, "td:nth-child(2)")
                ),
                location: get_html_string!(element, "td:nth-child(3)"),
                status,
                message: None,
            });
        }

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
