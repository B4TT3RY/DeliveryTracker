use async_trait::async_trait;
use chrono::TimeZone;
use chrono_tz::Asia::Seoul;
use nipper::Document;
use regex::Regex;
use serde_json::Value;

use crate::{
    structs::{Courier, TrackingError, TrackingResult},
    tracker,
};

pub struct Cainiao {}

#[async_trait]
impl Courier for Cainiao {
    fn id() -> &'static str {
        "cn.cainiao"
    }

    fn name() -> &'static str {
        "CAINIAO"
    }

    fn validate(tracking_number: &str) -> bool {
        let regex = Regex::new(r"^\d{13}|LP\w{14}$").unwrap();
        regex.is_match(tracking_number)
    }

    async fn track(tracking_number: &str) -> TrackingResult {
        if !Self::validate(tracking_number) {
            return Err(TrackingError::WrongTrackingNumber(
                "숫자 13자리 또는 LP + 숫자 14자리".to_string(),
            ));
        }

        let url = format!("https://global.cainiao.com/detail.htm?lang=en&mailNoList={}", tracking_number);

        let body = reqwest::Client::new()
            .get(&url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.0.4664.45 Safari/537.36")
            .send()
            .await?
            .text()
            .await?;

        let document = Document::from(&body);

        let json = document.select("#waybill_list_val_box").text().replace("&quot;", "\"");
        let json: Value = serde_json::from_str(&json)?;

        if !json["data"][0]["errorCode"].is_null() {
            return Err(TrackingError::NotExistsTrackingNumber);
        }

        let mut tracks: Vec<tracker::TrackingDetail> = vec![];

        for element in json["data"][0]["section2"]["detailList"]
            .as_array()
            .unwrap()
        {
            let datetime = Seoul
                .datetime_from_str(element["time"].as_str().unwrap(), "%Y-%m-%d %H:%M:%S")?;

            tracks.push(tracker::TrackingDetail {
                time: datetime.format("%Y-%m-%d %H:%M:%S").to_string(),
                message: Some(
                    element["desc"]
                        .as_str()
                        .unwrap()
                        .to_string(),
                ),
                status: None,
                location: None,
                live_tracking_url: None,
            });
        }

        tracks.reverse();

        Ok(tracker::TrackingInfo {
            id: Self::id().to_string(),
            name: Self::name().to_string(),
            url: url.to_string(),
            tracking_number: json["data"][0]["mailNo"].as_str().unwrap().to_string(),
            is_delivered: json["data"][0]["statusDesc"].as_str().unwrap() == "Delivered",
            sender: Some(json["data"][0]["originCountry"].as_str().unwrap().to_string()),
            receiver: Some(json["data"][0]["destCountry"].as_str().unwrap().to_string()),
            product: None,
            tracks,
        })
    }
}
