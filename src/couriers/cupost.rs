use anyhow::{anyhow, Result};
use nipper::Document;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::get_html_string;

use super::{Courier, DeliveryStatus, StateType, TrackingStatus};

pub const URL: &str = "https://www.cupost.co.kr/postbox/delivery/localResult.cupost";
pub const ID: &str = "kr.cupost";
pub const NAME: &str = "CU 편의점택배";

static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^(\d{1,12})$"#).unwrap());

pub fn validate(courier: &Courier) -> Result<()> {
    if !REGEX.is_match(&courier.tracking_number) {
        Err(anyhow!("운송장번호를 최대 12자리까지 입력해주세요."))
    } else {
        Ok(())
    }
}

pub fn state_from(status: &str) -> StateType {
    use StateType::*;
    if status.contains("점포접수") {
        AtPickup
    } else if status.contains("입고") || status.contains("출고") {
        InTransit
    } else if status.contains("도착") {
        OutForDelivery
    } else if status.contains("수령") {
        Delivered
    } else {
        Unknown
    }
}

pub async fn track(courier: &Courier) -> Result<DeliveryStatus> {
    let response = surf::post(URL)
        .body(format!("invoice_no={}", courier.tracking_number))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.90 Safari/537.36")
        .header("Referer", "https://www.cupost.co.kr/postbox/delivery/localResult.cupost")
        .recv_string()
        .await
        .map_err(|err| anyhow!(err))?;

    if response.contains("<iframe") {
        let cj_courier = Courier::new(
            "kr.cjlogistics".into(),
            Some(courier.tracking_number.clone()),
        )
        .unwrap();
        let cj = super::cjlogistics::track(&cj_courier).await;
        if let Err(err) = cj {
            return Err(err);
        }
        let mut cj = cj?;
        cj.id = ID.to_string();
        cj.name = format!("{} (CJ대한통운 국내택배)", NAME).to_string();
        return Ok(cj);
    }

    let document = Document::from(&response);

    if document.select(".ac").exists() {
        return Err(anyhow!(
            "{} {} 운송장 번호로 조회된 결과가 없습니다.",
            NAME,
            courier.tracking_number,
        ));
    }

    let tracking_number = get_html_string!(
        document,
        "#gotoMainContents > table:nth-child(5) > tbody > tr:nth-child(1) > td:nth-child(2)"
    );
    let product = get_html_string!(
        document,
        "#gotoMainContents > table:nth-child(5) > tbody > tr:nth-child(1) > td:nth-child(4)"
    );
    let sender = format!(
        "{} ({})",
        get_html_string!(
            document,
            "#gotoMainContents > table:nth-child(5) > tbody > tr:nth-child(3) > td:nth-child(2)"
        ),
        get_html_string!(
            document,
            "#gotoMainContents > table:nth-child(5) > tbody > tr:nth-child(4) > td"
        )
    );
    let receiver = format!(
        "{} ({})",
        get_html_string!(
            document,
            "#gotoMainContents > table:nth-child(5) > tbody > tr:nth-child(3) > td:nth-child(4)"
        ),
        get_html_string!(
            document,
            "#gotoMainContents > table:nth-child(5) > tbody > tr:nth-child(5) > td:nth-child(2)"
        )
    );

    let regex = Regex::new(r#"^(\d{4}.\d{2}.\d{2})(.|\n)*(\d{2}:\d{2})$"#)?;
    let mut tracks: Vec<TrackingStatus> = Vec::new();

    for element in document
        .select("#gotoMainContents > table:nth-child(10) > tbody > tr")
        .iter()
    {
        let status = get_html_string!(element, "td:nth-child(3)");
        let time = get_html_string!(element, "td:nth-child(1)");
        let cap = regex.captures(&time).unwrap();
        let time = format!(
            "{} {}",
            cap.get(1).map_or("", |m| m.as_str()),
            cap.get(3).map_or("", |m| m.as_str())
        );
        tracks.push(TrackingStatus {
            state: state_from(&status),
            time,
            location: Some(get_html_string!(element, "td:nth-child(2)")),
            status,
            message: None,
        });
    }

    Ok(DeliveryStatus {
        id: ID.to_string(),
        name: format!("{} (CU끼리택배)", NAME).to_string(),
        tracking_number,
        sender: Some(sender),
        receiver: Some(receiver),
        product: Some(product),
        tracks: Some(tracks),
    })
}
