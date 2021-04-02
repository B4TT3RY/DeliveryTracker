use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use regex::Regex;
use scraper::{Html, Selector};

use crate::{
    couriers::courier::Courier, delivery_status::DeliveryStatus, get_html_string,
    tracking_status::TrackingStatus,
};

pub struct Hanjin {
    pub tracking_number: String,
}

#[async_trait]
impl Courier for Hanjin {
    fn get_url() -> &'static str {
        "https://www.hanjin.co.kr/kor/CMS/DeliveryMgr/WaybillResult.do?mCode=MN038&schLang=KR&wblnumText=&wblnum="
    }

    fn get_id() -> &'static str {
        "kr.hanjin"
    }

    fn get_name() -> &'static str {
        "한진택배"
    }

    async fn validate(&self) -> Result<&Self> {
        if !Regex::new(r#"^(\d{10}|\d{12})$"#)?.is_match(&self.tracking_number) {
            return Err(anyhow!("운송장번호 10자리 또는 12자리를 입력해주세요."));
        }
        Ok(self)
    }

    async fn track(&self) -> Result<DeliveryStatus> {
        let response = surf::get(format!("{}{}", Self::get_url(), &self.tracking_number))
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.90 Safari/537.36")
            .recv_string()
            .await
            .map_err(|err| anyhow!(err))?;
        let document = Html::parse_document(&response);

        if document
            .select(&Selector::parse(".delivery-tbl").unwrap())
            .next()
            .is_none()
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

        let tracking_number = get_html_string!(document, "div.songjang-num > span.num");
        let sender = get_html_string!(document, r#"td[data-label="보내는 분"]"#);
        let receiver = get_html_string!(document, r#"td[data-label="받는 분"]"#);
        let product = get_html_string!(document, r#"td[data-label="상품명"]"#);

        let mut tracks: Vec<TrackingStatus> = Vec::new();
        let selector = Selector::parse("div.waybill-tbl > table > tbody > tr").unwrap();
        for element in document.select(&selector) {
            tracks.push(TrackingStatus {
                time: format!(
                    "{} {}",
                    get_html_string!(element, ".w-date"),
                    get_html_string!(element, ".w-time")
                ),
                location: get_html_string!(element, ".w-org"),
                status: get_html_string!(element, ".stateDesc > strong"),
                message: Some(get_html_string!(element, ".stateDesc")),
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
