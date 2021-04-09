use anyhow::{anyhow, Result};
use async_trait::async_trait;
use nipper::Document;
use regex::Regex;

use crate::{
    couriers::courier::{Courier, CourierType},
    get_html_string,
    status_struct::{DeliveryStatus, StateType, TrackingStatus},
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
        let document = Document::from(&response);

        if !document.select(".delivery-tbl").exists() {
            return Err(anyhow!(
                "{} {} 운송장 번호로 조회된 결과가 없습니다.",
                Self::get_name(),
                &self.tracking_number
            ));
        }

        let tracking_number = get_html_string!(document, "div.songjang-num > span.num");
        let sender = get_html_string!(document, r#"td[data-label="보내는 분"]"#);
        let receiver = get_html_string!(document, r#"td[data-label="받는 분"]"#);
        let product = get_html_string!(document, r#"td[data-label="상품명"]"#);

        let regex = Regex::new("(접수|입고|이동중|도착|배송준비중|배송출발|배송완료)").unwrap();
        let mut tracks: Vec<TrackingStatus> = Vec::new();
        
        for element in document.select("div.waybill-tbl > table > tbody > tr").iter() {
            let message = get_html_string!(element, ".stateDesc");
            let status = regex.captures(&message).unwrap().get(0).unwrap().as_str();
            tracks.push(TrackingStatus {
                state: StateType::to_type(
                    CourierType::get_courier(Self::get_id().to_string(), None)?,
                    &status,
                ),
                time: format!(
                    "{} {}",
                    get_html_string!(element, ".w-date"),
                    get_html_string!(element, ".w-time")
                ),
                location: get_html_string!(element, ".w-org"),
                status: status.to_string(),
                message: Some(message),
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
