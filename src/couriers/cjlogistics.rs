use anyhow::{anyhow, Result};
use async_trait::async_trait;
use regex::Regex;
use scraper::{Html, Selector};

use crate::{
    couriers::courier::Courier, delivery_status::DeliveryStatus, get_html_string,
    tracking_status::TrackingStatus,
};

pub struct CJLogistics {
    pub tracking_number: String,
}

#[async_trait]
impl Courier for CJLogistics {
    fn get_url() -> &'static str {
        "https://www.doortodoor.co.kr/parcel/doortodoor.do"
    }

    fn get_id() -> &'static str {
        "kr.cjlogistics"
    }

    fn get_name() -> &'static str {
        "CJ대한통운"
    }

    async fn validate(&self) -> Result<&Self> {
        if !Regex::new(r#"^(\d{10}|\d{12})$"#)?.is_match(&self.tracking_number) {
            return Err(anyhow!("운송장번호 10자리 또는 12자리를 입력해주세요."));
        }
        Ok(self)
    }

    async fn track(&self) -> Result<DeliveryStatus> {
        let response = surf::post(Self::get_url())
            .body(format!("fsp_action=PARC_ACT_002&fsp_cmd=retrieveInvNoACT2&invc_no={}", self.tracking_number))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Referer", "https://www.doortodoor.co.kr/parcel/pa_004.jsp")
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.90 Safari/537.36")
            .recv_string()
            .await
            .unwrap();
        let document = Html::parse_document(&response);

        let tracking_number = get_html_string!(document, ".last_b:nth-child(1)");
        let sender = get_html_string!(document, ".last_b:nth-child(2)");
        let receiver = get_html_string!(document, ".last_b:nth-child(3)");
        let product = get_html_string!(document, ".last_b:nth-child(4)");

        let mut tracks: Vec<TrackingStatus> = Vec::new();
        let selector = Selector::parse(
            "#tabContents > ul > li.first.focus > div > div:nth-child(2) > div > table > tbody",
        )
        .unwrap();
        let tr_selector = Selector::parse("tr").unwrap();
        let parent = document.select(&selector).next().unwrap();

        for element in parent.select(&tr_selector) {
            if element.inner_html().contains("th") {
                continue;
            }

            tracks.push(TrackingStatus {
                time: get_html_string!(element, "td:nth-child(2)"),
                location: get_html_string!(element, "td > a"),
                status: get_html_string!(element, "td:nth-child(1)"),
                message: Some(get_html_string!(element, "td:nth-child(3)")),
            });
        }

        Ok(DeliveryStatus {
            id: Self::get_id().to_string(),
            name: Self::get_name().to_string(),
            tracking_number,
            sender: Some(sender),
            receiver: Some(receiver),
            product: Some(product),
            tracks: Some(tracks),
        })
    }
}
