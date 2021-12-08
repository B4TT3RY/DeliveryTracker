use async_trait::async_trait;
use chrono::TimeZone;
use chrono_tz::Asia::Seoul;
use serde_json::Value;

use crate::{
    structs::{Courier, TrackingError, TrackingResult},
    tracker,
};

pub struct Kyoungdong {}

#[async_trait]
impl Courier for Kyoungdong {
    fn id() -> &'static str {
        "kr.kyoungdong"
    }

    fn name() -> &'static str {
        "경동택배"
    }

    fn validate(tracking_number: &str) -> bool {
        tracking_number.parse::<u64>().is_ok() && tracking_number.len() == 13
    }

    async fn track(tracking_number: &str) -> TrackingResult {
        if !Self::validate(tracking_number) {
            return Err(TrackingError::WrongTrackingNumber(
                "숫자 13자리".to_string(),
            ));
        }

        let json: Value = reqwest::get(format!(
            "https://kdexp.com/newDeliverySearch.kd?barcode={}",
            tracking_number
        ))
        .await?
        .json()
        .await?;

        if json["result"].as_str().unwrap() == "fail" {
            return Err(TrackingError::NotExistsTrackingNumber);
        }

        let mut tracks: Vec<tracker::TrackingDetail> = vec![];

        for element in json["items"].as_array().unwrap() {
            let datetime = Seoul.datetime_from_str(
                element["reg_date"].as_str().unwrap(),
                "%Y-%m-%d %H:%M:%S.%f",
            )?;

            tracks.push(tracker::TrackingDetail {
                time: datetime.format("%Y-%m-%d %H:%M:%S").to_string(),
                message: None,
                status: Some(element["stat"].as_str().unwrap().to_string()),
                location: Some(element["location"].as_str().unwrap().to_string()),
                live_tracking_url: None,
            });
        }

        Ok(tracker::TrackingInfo {
            id: Self::id().to_string(),
            name: Self::name().to_string(),
            url: "https://kdexp.com/main.kd".to_string(),
            tracking_number: json["info"]["barcode"].as_str().unwrap().to_string(),
            is_delivered: !json["info"]["rec_dt"].is_null(),
            sender: Some(format!(
                "{} ({})",
                json["info"]["send_name"].as_str().unwrap(),
                json["info"]["branch_start"].as_str().unwrap()
            )),
            receiver: Some(format!(
                "{} ({})",
                json["info"]["re_name"].as_str().unwrap(),
                json["info"]["branch_end"].as_str().unwrap()
            )),
            product: Some(json["info"]["prod"].as_str().unwrap().to_string()),
            tracks,
        })
    }
}
