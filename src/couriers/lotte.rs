use anyhow::{anyhow, Result};
use nipper::Document;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::get_html_string;

use super::{Courier, DeliveryStatus, StateType, TrackingStatus};

pub const URL: &str = "https://www.lotteglogis.com/home/reservation/tracking/linkView";
pub const ID: &str = "kr.lotte";
pub const NAME: &str = "롯데택배";

static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^(\d{10}|\d{12}|\d{13})$"#).unwrap());

pub fn validate(courier: &Courier) -> Result<()> {
    if !REGEX.is_match(&courier.tracking_number) {
        Err(anyhow!(
            "운송장번호 10자리, 12자리 또는 13자리를 입력해주세요."
        ))
    } else {
        Ok(())
    }
}

pub fn state_from(status: &str) -> StateType {
    use StateType::*;
    match status {
        "상품접수" => AtPickup,
        "상품 이동중" => InTransit,
        "배송 출발" => OutForDelivery,
        "배달 완료" => Delivered,
        _ => Unknown,
    }
}

pub async fn track(courier: &Courier) -> Result<DeliveryStatus> {
    let response = surf::post(URL)
        .body(format!("InvNo={}", courier.tracking_number))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.90 Safari/537.36")
        .recv_string()
        .await
        .map_err(|err| anyhow!(err))?;
    let document = Document::from(&response);

    if get_html_string!(
        document,
        "#contents > div > div.contArea > table.tblH.mt60 > tbody > tr > td"
    )
    .contains("배송정보가 없습니다")
    {
        return Err(anyhow!(
            "{} {} 운송장 번호로 조회된 결과가 없습니다.",
            NAME,
            &courier.tracking_number
        ));
    }

    let mut tracks: Vec<TrackingStatus> = Vec::new();
    for element in document
        .select("div.contArea > table:nth-child(4) > tbody > tr")
        .iter()
    {
        let location = get_html_string!(element, "td:nth-child(3)");
        if location == "고객" {
            continue;
        }
        let status = get_html_string!(element, "td:nth-child(1)");
        tracks.push(TrackingStatus {
            state: state_from(&status),
            time: get_html_string!(element, "td:nth-child(2)"),
            location: Some(location),
            status,
            message: Some(get_html_string!(element, "td:nth-child(4)")),
        });
    }

    tracks.reverse();

    Ok(DeliveryStatus {
        id: ID.to_string(),
        name: NAME.to_string(),
        tracking_number: courier.tracking_number.clone(),
        sender: None,
        receiver: None,
        product: None,
        tracks: Some(tracks),
    })
}

/*use anyhow::{anyhow, Result};
use async_trait::async_trait;
use nipper::Document;
use regex::Regex;

use crate::{
    couriers::courier::{Courier, CourierType},
    get_html_string,
    status_struct::{DeliveryStatus, StateType, TrackingStatus},
};

pub struct Lotte {
    pub tracking_number: String,
}

#[async_trait]
impl Courier for Lotte {
    fn get_url() -> &'static str {
        "https://www.lotteglogis.com/home/reservation/tracking/linkView"
    }

    fn get_id() -> &'static str {
        "kr.lotte"
    }

    fn get_name() -> &'static str {
        "롯데택배"
    }

    async fn validate(&self) -> Result<&Self> {
        if !Regex::new(r#"^(\d{10}|\d{12}|\d{13})$"#)?.is_match(&self.tracking_number) {
            return Err(anyhow!(
                "운송장번호 10자리, 12자리 또는 13자리를 입력해주세요."
            ));
        }
        Ok(self)
    }

    async fn track(&self) -> Result<DeliveryStatus> {
        let response = surf::post(Self::get_url())
            .body(format!("InvNo={}", self.tracking_number))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.90 Safari/537.36")
            .recv_string()
            .await
            .map_err(|err| anyhow!(err))?;
        let document = Document::from(&response);

        if get_html_string!(
            document,
            "#contents > div > div.contArea > table.tblH.mt60 > tbody > tr > td"
        )
        .contains("배송정보가 없습니다")
        {
            return Err(anyhow!(
                "{} {} 운송장 번호로 조회된 결과가 없습니다.",
                Self::get_name(),
                &self.tracking_number
            ));
        }

        let mut tracks: Vec<TrackingStatus> = Vec::new();
        for element in document
            .select("div.contArea > table:nth-child(4) > tbody > tr")
            .iter()
        {
            let location = get_html_string!(element, "td:nth-child(3)");
            if location == "고객" {
                continue;
            }
            let status = get_html_string!(element, "td:nth-child(1)");
            tracks.push(TrackingStatus {
                state: StateType::to_type(
                    CourierType::get_courier(Self::get_id().to_string(), None)?,
                    &status,
                ),
                time: get_html_string!(element, "td:nth-child(2)"),
                location: Some(location),
                status,
                message: Some(get_html_string!(element, "td:nth-child(4)")),
            });
        }

        tracks.reverse();

        Ok(DeliveryStatus {
            id: Self::get_id().to_string(),
            name: Self::get_name().to_string(),
            tracking_number: self.tracking_number.clone(),
            sender: None,
            receiver: None,
            product: None,
            tracks: Some(tracks),
        })
    }
}
*/
