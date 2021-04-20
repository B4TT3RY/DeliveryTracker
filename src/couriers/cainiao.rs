use anyhow::{anyhow, Result};
use nipper::Document;
use serde_json::Value;

use crate::get_html_string;

use super::{Courier, DeliveryStatus, StateType, TrackingStatus};

pub const URL: &str = "https://global.cainiao.com/detail.htm?lang=en&mailNoList=";
pub const ID: &str = "cn.cainiao";
pub const NAME: &str = "CAINIAO";

pub fn validate(courier: &Courier) -> Result<()> {
    if courier.tracking_number.is_empty() {
        Err(anyhow!("운송장번호를 입력해주세요."))
    } else {
        Ok(())
    }
}

pub fn state_from(_status: &str) -> StateType {
    use StateType::*;
    Unknown
}

pub async fn track(courier: &Courier) -> Result<DeliveryStatus> {
    let response = surf::get(format!("{}{}", URL, courier.tracking_number))
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.90 Safari/537.36")
        .recv_string()
        .await
        .map_err(|err| anyhow!(err))?;
    let document = Document::from(&response);

    let json = get_html_string!(document, "#waybill_list_val_box").replace("&quot;", "\"");
    let json: Value = serde_json::from_str(&json)?;

    if !json["data"][0]["errorCode"].is_null() {
        return Err(anyhow!(
            "{} {} 운송장 번호로 조회된 결과가 없습니다.",
            NAME,
            &courier.tracking_number
        ));
    }

    let tracking_number = json["data"][0]["mailNo"].as_str().unwrap().to_string();
    let sender = json["data"][0]["originCountry"]
        .as_str()
        .unwrap()
        .to_string();
    let receiver = json["data"][0]["destCountry"].as_str().unwrap().to_string();

    let mut tracks: Vec<TrackingStatus> = Vec::new();

    for value in json["data"][0]["section2"]["detailList"]
        .as_array()
        .unwrap()
    {
        let status = value["desc"].as_str().unwrap().to_string();

        tracks.push(TrackingStatus {
            state: state_from(&status),
            time: value["time"].as_str().unwrap().to_string(),
            location: None,
            status,
            message: None,
        });
    }

    tracks.reverse();

    Ok(DeliveryStatus {
        id: ID.to_string(),
        name: NAME.to_string(),
        tracking_number,
        sender: Some(sender),
        receiver: Some(receiver),
        product: None,
        tracks: Some(tracks),
    })
}
