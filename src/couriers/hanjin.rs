use anyhow::{anyhow, Result};
use nipper::Document;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::get_html_string;

use super::{Courier, DeliveryStatus, StateType, TrackingStatus};

pub const URL: &str = "https://www.hanjin.co.kr/kor/CMS/DeliveryMgr/WaybillResult.do?mCode=MN038&schLang=KR&wblnumText=&wblnum=";
pub const ID: &str = "kr.hanjin";
pub const NAME: &str = "한진택배";

static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^\d{12}$"#).unwrap());

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
        "접수" => InformationReceived,
        "입고" => AtPickup,
        "이동중" | "도착" | "배송준비중" => InTransit,
        "배송출발" => OutForDelivery,
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

    if !document.select(".delivery-tbl").exists() {
        return Err(anyhow!(
            "{} {} 운송장 번호로 조회된 결과가 없습니다.",
            NAME,
            &courier.tracking_number
        ));
    }

    let tracking_number = get_html_string!(document, "div.songjang-num > span.num");
    let sender = get_html_string!(document, r#"td[data-label="보내는 분"]"#);
    let receiver = get_html_string!(document, r#"td[data-label="받는 분"]"#);
    let product = get_html_string!(document, r#"td[data-label="상품명"]"#);

    let regex = Regex::new("(접수|입고|이동중|도착|배송준비중|배송출발|배송완료)").unwrap();
    let mut tracks: Vec<TrackingStatus> = Vec::new();

    for element in document
        .select("div.waybill-tbl > table > tbody > tr")
        .iter()
    {
        let message = get_html_string!(element, ".stateDesc");
        let status = regex.captures(&message).unwrap().get(0).unwrap().as_str();
        tracks.push(TrackingStatus {
            state: state_from(&status),
            time: format!(
                "{} {}",
                get_html_string!(element, ".w-date"),
                get_html_string!(element, ".w-time")
            ),
            location: Some(get_html_string!(element, ".w-org")),
            status: status.to_string(),
            message: Some(message),
        });
    }

    tracks.sort_by_key(|k| StateType::get_priority(k.state));

    Ok(DeliveryStatus {
        id: ID.to_string(),
        name: NAME.to_string(),
        tracking_number,
        sender: Some(sender),
        receiver: Some(receiver),
        product: Some(product),
        tracks: Some(tracks),
    })
}
