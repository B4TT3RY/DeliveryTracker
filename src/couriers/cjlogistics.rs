use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use regex::Regex;
use scraper::{Html, Selector};

use crate::{
    couriers::courier::{Courier, CourierType},
    get_html_string,
    status_struct::{DeliveryStatus, StateType, TrackingStatus},
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
            .map_err(|err| anyhow!(err))?;
        let document = Html::parse_document(&response);

        if get_html_string!(
            document,
            "#tabContents div:nth-child(1) table tr:nth-child(2) td"
        )
        .contains("조회된 데이터가 없습니다")
        {
            return Err(anyhow!(
                "{} {} 운송장 번호로 조회된 결과가 없습니다.",
                Self::get_name(),
                &self.tracking_number
            ));
        }

        let tracking_number = get_html_string!(document, ".last_b:nth-child(1)");
        let sender = get_html_string!(document, ".last_b:nth-child(2)");
        let receiver = get_html_string!(document, ".last_b:nth-child(3)");
        let product = get_html_string!(document, ".last_b:nth-child(4)");

        let mut tracks: Vec<TrackingStatus> = Vec::new();
        let selector = Selector::parse(
            "#tabContents > ul > li.first.focus > div > div:nth-child(2) > div > table > tbody >tr",
        )
        .unwrap();

        for element in document.select(&selector) {
            if element.inner_html().contains("th") {
                continue;
            }

            let status = get_html_string!(element, "td:nth-child(1)");

            tracks.push(TrackingStatus {
                state: StateType::to_type(
                    CourierType::get_courier(Self::get_id().to_string(), None)?,
                    &status,
                ),
                time: get_html_string!(element, "td:nth-child(2)"),
                location: get_html_string!(element, "td > a"),
                status,
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
