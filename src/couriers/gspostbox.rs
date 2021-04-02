use anyhow::{anyhow, Result};
use async_trait::async_trait;
use regex::Regex;
use serde_json::Value;

use crate::{
    couriers::courier::{Courier, CourierType},
    delivery_status::DeliveryStatus,
    tracking_status::TrackingStatus,
};

pub struct GSPostbox {
    pub tracking_number: String,
}

#[async_trait]
impl Courier for GSPostbox {
    fn get_url() -> &'static str {
        "https://www.cvsnet.co.kr/invoice/tracking.do?invoice_no="
    }

    fn get_id() -> &'static str {
        "kr.gspostbox"
    }

    fn get_name() -> &'static str {
        "GS Postbox 택배"
    }

    async fn validate(&self) -> Result<&Self> {
        if !Regex::new(r#"^(\d{1,13})$"#)?.is_match(&self.tracking_number) {
            return Err(anyhow!("운송장번호를 최대 13자리까지 입력해주세요."));
        }
        Ok(self)
    }

    async fn track(&self) -> Result<DeliveryStatus> {
        let response = surf::get(format!("{}{}", Self::get_url(), &self.tracking_number))
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.90 Safari/537.36")
            .recv_string()
            .await
            .map_err(|err| anyhow!(err))?;
        let regex = Regex::new("(var trackingInfo = )(.+)(;)")?;
        let capture = regex.captures(&response).unwrap();
        let json = capture.get(2).map_or("", |m| m.as_str());

        let json = serde_json::from_str::<Value>(json)?;

        if json["code"].as_i64().unwrap() != 200 {
            return Err(anyhow!("{} {} 운송장 번호로 조회된 결과가 없습니다.", Self::get_name(), &self.tracking_number));
        }

        let name = format!(
            "GS Postbox {} ({})",
            json["serviceName"].as_str().unwrap_or(""),
            json["carrierName"].as_str().unwrap_or("")
        );

        if json["carrierName"].as_str().unwrap_or("") == "CJ대한통운" {
            let cj = CourierType::track("kr.cjlogistics".to_string(), self.tracking_number.clone())
                .await;
            if let Err(err) = cj {
                return Err(err);
            }
            let mut cj = cj?;
            cj.id = Self::get_id().to_string();
            cj.name = name;
            return Ok(cj);
        }

        let tracking_number = json["invoiceNo"].as_str().unwrap_or("").to_string();
        let sender = json["sender"]["name"].as_str().unwrap_or("").to_string();
        let receiver = json["receiver"]["name"].as_str().unwrap_or("").to_string();
        let product = json["goodsName"].as_str().unwrap_or("").to_string();

        let mut tracks: Vec<TrackingStatus> = Vec::new();
        for value in json["trackingDetails"].as_array().unwrap() {
            tracks.push(TrackingStatus {
                time: value["transTime"]
                    .as_str()
                    .unwrap_or("")
                    .replace("T", " ")
                    .to_string(),
                location: value["transWhere"].as_str().unwrap_or("").to_string(),
                status: value["transKind"]
                    .as_str()
                    .unwrap_or("")
                    .replace("  ", " ")
                    .to_string(),
                message: None,
            });
        }

        Ok(DeliveryStatus {
            id: Self::get_id().to_string(),
            name,
            tracking_number,
            sender: Some(sender),
            receiver: Some(receiver),
            product: Some(product),
            tracks: Some(tracks),
        })
    }
}
