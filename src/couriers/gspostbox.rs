use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::Value;

use super::{Courier, DeliveryStatus, StateType, TrackingStatus};

pub const URL: &str = "https://www.cvsnet.co.kr/invoice/tracking.do?invoice_no=";
pub const ID: &str = "kr.gspostbox";
pub const NAME: &str = "GS Postbox 택배";

static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^(\d{1,13})$"#).unwrap());

pub fn validate(courier: &Courier) -> Result<()> {
    if !REGEX.is_match(&courier.tracking_number) {
        Err(anyhow!("운송장번호를 최대 13자리까지 입력해주세요."))
    } else {
        Ok(())
    }
}

pub fn state_from(status: &str) -> StateType {
    use StateType::*;
    if status == "점포접수" {
        InformationReceived
    } else if status.contains("인수") {
        AtPickup
    } else if status.contains("입고") || status.contains("인계") {
        InTransitReceived
    } else if status.contains("출고") {
        InTransitSend
    } else if status == "점포도착" {
        OutForDelivery
    } else if status == "고객전달" {
        Delivered
    } else {
        Unknown
    }
}

pub async fn track(courier: &Courier) -> Result<DeliveryStatus> {
    let response = surf::get(format!("{}{}", URL, &courier.tracking_number))
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.90 Safari/537.36")
        .recv_string()
        .await
        .map_err(|err| anyhow!(err))?;
    let regex = Regex::new("(var trackingInfo = )(.+)(;)")?;
    let capture = regex.captures(&response).unwrap();
    let json = capture.get(2).map_or("", |m| m.as_str());

    let json = serde_json::from_str::<Value>(json)?;

    if json["code"].as_i64().unwrap() != 200 {
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

    let name = format!(
        "GS Postbox {} ({})",
        json["serviceName"].as_str().unwrap_or(""),
        json["carrierName"].as_str().unwrap_or("")
    );

    if json["carrierName"].as_str().unwrap_or("") == "CJ대한통운" {
        let cj_courier = Courier::new(
            "kr.cjlogistics".to_string(),
            Some(courier.tracking_number.clone()),
        )
        .unwrap();
        let cj = super::cjlogistics::track(&cj_courier).await;
        if let Err(err) = cj {
            return Err(err);
        }
        let mut cj = cj?;
        cj.id = ID.to_string();
        cj.name = name;
        return Ok(cj);
    }

    let sender = json["sender"]["name"].as_str().unwrap_or("").to_string();
    let receiver = json["receiver"]["name"].as_str().unwrap_or("").to_string();
    let product = json["goodsName"].as_str().unwrap_or("").to_string();

    let mut tracks: Vec<TrackingStatus> = Vec::new();
    for value in json["trackingDetails"].as_array().unwrap() {
        let status = value["transKind"]
            .as_str()
            .unwrap_or("")
            .replace("  ", " ")
            .to_string();
        tracks.push(TrackingStatus {
            state: state_from(&status),
            time: value["transTime"]
                .as_str()
                .unwrap_or("")
                .replace("T", " ")
                .to_string(),
            location: Some(value["transWhere"].as_str().unwrap_or("").to_string()),
            status,
            message: None,
        });
    }

    tracks.sort_by_key(|k| StateType::get_priority(k.state));

    Ok(DeliveryStatus {
        id: ID.to_string(),
        name,
        tracking_number: Some(courier.tracking_number.to_string()),
        sender: Some(sender),
        receiver: Some(receiver),
        product: Some(product),
        tracks: Some(tracks),
    })
}
