use anyhow::{anyhow, Result};
use nipper::Document;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::get_html_string;

use super::{Courier, DeliveryStatus, StateType, TrackingStatus};

pub const URL: &str = "https://www.doortodoor.co.kr/parcel/doortodoor.do";
pub const ID: &str = "kr.cjlogistics";
pub const NAME: &str = "CJ대한통운";

static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^(\d{10}|\d{12})$"#).unwrap());

pub fn validate(courier: &Courier) -> Result<()> {
    if !REGEX.is_match(&courier.tracking_number) {
        Err(anyhow!("운송장번호 10자리 또는 12자리를 입력해주세요."))
    } else {
        Ok(())
    }
}

pub fn state_from(status: &str) -> StateType {
    use StateType::*;
    match status {
        "상품인수" => AtPickup,
        "상품이동중" => InTransit,
        "배달지도착" => ReadyForDelivery,
        "배달출발" => OutForDelivery,
        "배달완료" => Delivered,
        _ => Unknown,
    }
}

pub async fn track(courier: &Courier) -> Result<DeliveryStatus> {
    let response = surf::post(URL)
        .body(format!("fsp_action=PARC_ACT_002&fsp_cmd=retrieveInvNoACT2&invc_no={}", courier.tracking_number))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Referer", "https://www.doortodoor.co.kr/parcel/pa_004.jsp")
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

    let tracking_number = get_html_string!(document, "#tabContents > ul > li.first.focus > div > div:nth-child(1) > div > table > tbody > tr:nth-child(2) > td:nth-child(1)");
    let sender = get_html_string!(document, "#tabContents > ul > li.first.focus > div > div:nth-child(1) > div > table > tbody > tr:nth-child(2) > td:nth-child(2)");
    let receiver = get_html_string!(document, "#tabContents > ul > li.first.focus > div > div:nth-child(1) > div > table > tbody > tr:nth-child(2) > td:nth-child(3)");
    let product = get_html_string!(document, "#tabContents > ul > li.first.focus > div > div:nth-child(1) > div > table > tbody > tr:nth-child(2) > td:nth-child(4)");

    let mut tracks: Vec<TrackingStatus> = Vec::new();

    for element in document
        .select(
            "#tabContents > ul > li.first.focus > div > div:nth-child(2) > div > table > tbody >tr",
        )
        .iter()
    {
        if element.html().contains("th") {
            continue;
        }

        let status = get_html_string!(element, "td:nth-child(1)");

        tracks.push(TrackingStatus {
            state: state_from(&status),
            time: get_html_string!(element, "td:nth-child(2)"),
            location: Some(get_html_string!(element, "td > a")),
            status,
            message: Some(get_html_string!(element, "td:nth-child(3)")),
        });
    }

    tracks.sort_by_key(|k| StateType::get_priority(k.state));

    Ok(DeliveryStatus {
        id: ID.to_string(),
        name: NAME.to_string(),
        tracking_number: Some(tracking_number),
        sender: Some(sender),
        receiver: Some(receiver),
        product: Some(product),
        tracks: Some(tracks),
    })
}
