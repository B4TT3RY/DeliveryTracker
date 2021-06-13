use anyhow::{anyhow, Result};
use nipper::Document;
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::Value;

use crate::get_html_string;

use super::{Courier, DeliveryStatus, StateType, TrackingStatus};

pub const URL: &str = "https://global.cainiao.com/detail.htm?lang=en&mailNoList=";
pub const ID: &str = "cn.cainiao";
pub const NAME: &str = "CAINIAO";

static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^[a-zA-Z0-9]+$"#).unwrap());

pub fn validate(courier: &Courier) -> Result<()> {
    if !REGEX.is_match(&courier.tracking_number) {
        Err(anyhow!("운송장번호를 다시 확인해주세요."))
    } else {
        Ok(())
    }
}

pub fn state_from(status: &str) -> StateType {
    use StateType::*;
    if status.contains("Delivered") {
        Delivered
    } else {
        Unknown
    }
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
        return Ok(
            DeliveryStatus {
                id: ID.to_string(),
                name: NAME.to_string(),
                tracking_number: None,
                sender: None,
                receiver: None,
                product: None,
                tracks: None,
            }
        );
    }

    let sender = json["data"][0]["originCountry"]
        .as_str()
        .ok_or_else(|| anyhow!("Can't find originCountry in cn.cainiao"))?
        .to_string();
    let receiver = json["data"][0]["destCountry"]
        .as_str()
        .ok_or_else(|| anyhow!("Can't find destCountry in cn.cainiao"))?
        .to_string();

    let mut tracks: Vec<TrackingStatus> = Vec::new();

    for value in json["data"][0]["section2"]["detailList"]
        .as_array()
        .ok_or_else(|| anyhow!("Can't find detailList in cn.cainiao"))?
    {
        let status = value["desc"]
            .as_str()
            .ok_or_else(|| anyhow!("Can't find desc in cn.cainiao"))?
            .to_string();

        tracks.push(TrackingStatus {
            state: state_from(&status),
            time: value["time"]
                .as_str()
                .ok_or_else(|| anyhow!("Can't find time in cn.cainiao"))?
                .to_string(),
            location: None,
            status,
            message: None,
        });
    }

    tracks.reverse();
    tracks.sort_by_key(|k| StateType::get_priority(k.state));

    Ok(DeliveryStatus {
        id: ID.to_string(),
        name: NAME.to_string(),
        tracking_number: Some(courier.tracking_number.to_string()),
        sender: Some(sender),
        receiver: Some(receiver),
        product: None,
        tracks: Some(tracks),
    })
}
