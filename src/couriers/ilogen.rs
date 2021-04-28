use anyhow::{anyhow, Result};
use nipper::Document;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::get_html_string;

use super::{Courier, DeliveryStatus, StateType, TrackingStatus};

pub const URL: &str = "https://www.ilogen.com/web/personal/trace/";
pub const ID: &str = "kr.ilogen";
pub const NAME: &str = "로젠택배";

static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^(\d{11})$"#).unwrap());

pub fn validate(courier: &Courier) -> Result<()> {
    if !REGEX.is_match(&courier.tracking_number) {
        Err(anyhow!("운송장번호 11자리를 입력해주세요."))
    } else {
        Ok(())
    }
}

pub fn state_from(status: &str) -> StateType {
    use StateType::*;
    match status {
        "집하완료" => AtPickup,
        "터미널입고" | "터미널출고" | "배송입고" => InTransit,
        "배송출고" => OutForDelivery,
        "배송완료" => Delivered,
        _ => Unknown,
    }
}

pub async fn track(courier: &Courier) -> Result<DeliveryStatus> {
    let response = surf::get(format!("{}{}", URL, &courier.tracking_number))
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.90 Safari/537.36")
        .recv_string()
        .await
        .map_err(|err| anyhow!(err))?;
    let document = Document::from(&response);

    if document.select(".empty").exists() {
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

    let product = get_html_string!(
        document,
        "table.horizon.pdInfo > tbody > tr:nth-child(1) > td:nth-child(4)"
    );
    let sender = get_html_string!(
        document,
        "table.horizon.pdInfo > tbody > tr:nth-child(4) > td:nth-child(2)"
    );
    let receiver = get_html_string!(
        document,
        "table.horizon.pdInfo > tbody > tr:nth-child(4) > td:nth-child(4)"
    );

    let mut tracks: Vec<TrackingStatus> = Vec::new();
    for element in document.select("table.data.tkInfo > tbody > tr").iter() {
        let status = get_html_string!(element, "td:nth-child(3)");
        tracks.push(TrackingStatus {
            state: state_from(&status),
            time: get_html_string!(element, "td:nth-child(1)"),
            location: Some(get_html_string!(element, "td:nth-child(2)")),
            status: status.clone(),
            message: if status == "배송출고" {
                Some(
                    format!(
                        "{} ({} 배송 예정)",
                        get_html_string!(element, "td:nth-child(4)"),
                        get_html_string!(element, "td:nth-child(6)").replace("배송", ""),
                    )
                    .to_string(),
                )
            } else {
                Some(get_html_string!(element, "td:nth-child(4)"))
            },
        });
    }

    tracks.sort_by_key(|k| StateType::get_priority(k.state));

    Ok(DeliveryStatus {
        id: ID.to_string(),
        name: NAME.to_string(),
        tracking_number: Some(courier.tracking_number.clone()),
        sender: Some(sender),
        receiver: Some(receiver),
        product: Some(product),
        tracks: Some(tracks),
    })
}
