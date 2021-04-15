use anyhow::{anyhow, Result};
use async_trait::async_trait;
use nipper::Document;
use serde_json::Value;

use crate::{
    couriers::courier::{Courier, CourierType},
    get_html_string,
    status_struct::{DeliveryStatus, StateType, TrackingStatus},
};

pub struct Cainiao {
    pub tracking_number: String,
}

#[async_trait]
impl Courier for Cainiao {
    fn get_url() -> &'static str {
        "https://global.cainiao.com/detail.htm?lang=en&mailNoList="
    }

    fn get_id() -> &'static str {
        "cn.cainiao"
    }

    fn get_name() -> &'static str {
        "CAINIAO"
    }

    async fn validate(&self) -> Result<&Self> {
        if self.tracking_number.is_empty() {
            return Err(anyhow!("운송장번호를 입력해주세요."));
        }
        Ok(self)
    }

    async fn track(&self) -> Result<DeliveryStatus> {
        let response = surf::get(format!("{}{}", Self::get_url(), self.tracking_number))
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.90 Safari/537.36")
            .recv_string()
            .await
            .map_err(|err| anyhow!(err))?;
        let document = Document::from(&response);

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

        let json = get_html_string!(document, "#waybill_list_val_box")
            .replace("&quot;", "\"");
        let json: Value = serde_json::from_str(&json)?;

        let tracking_number = json["data"][0]["mailNo"].as_str().unwrap().to_string();
        let sender = json["data"][0]["originCountry"].as_str().unwrap().to_string();
        let receiver = json["data"][0]["destCountry"].as_str().unwrap().to_string();

        let mut tracks: Vec<TrackingStatus> = Vec::new();

        for value in json["data"][0]["section2"]["detailList"].as_array().unwrap() {
            let status = value["desc"].as_str().unwrap().to_string();

            tracks.push(TrackingStatus {
                state: StateType::to_type(
                    CourierType::get_courier(Self::get_id().to_string(), None)?,
                    &status,
                ),
                time: value["time"].as_str().unwrap().to_string(),
                location: "".to_string(),
                status,
                message: None,
            });
        }
        
        tracks.reverse();

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
