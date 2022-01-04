use async_trait::async_trait;
use chrono::TimeZone;
use chrono_tz::Asia::Seoul;
use serde_json::Value;

use crate::{
    structs::{Courier, TrackingError},
    tracker,
};

pub struct Fedex {}

#[async_trait]
impl Courier for Fedex {
    fn id() -> &'static str {
        "us.fedex"
    }

    fn name() -> &'static str {
        "FedEx"
    }

    fn validate(tracking_number: &str) -> bool {
        tracking_number.parse::<u64>().is_ok() && tracking_number.len() == 12
    }

    async fn track(tracking_number: &str) -> crate::structs::TrackingResult {
        if !Self::validate(tracking_number) {
            return Err(TrackingError::WrongTrackingNumber(
                "숫자 12자리".to_string(),
            ));
        }

        let url = "https://www.fedex.com/trackingCal/track";

        let params = [
            ("action", "trackpackages"),
            ("format", "json"),
            ("data", ""),
            ("format", "ko_KR"),
            ("version", "1"),
        ];

        let json: Value = reqwest::Client::new()
            .post(url)
            .form(&params)
            .send()
            .await?
            .json()
            .await?;

        let json = &json["TrackPackagesResponse"];

        if json["packageList"][0]["errorList"][0]["code"].is_null() {
            return Err(TrackingError::NotExistsTrackingNumber);
        }

        let mut tracks: Vec<tracker::TrackingDetail> = vec![];

        let package_info = &json["packageList"][0];

        for scan in package_info["scanEventList"].as_array().unwrap() {
            let datetime =
                Seoul.datetime_from_str(&format!(
                    "{} {} {}",
                    scan["date"].as_str().unwrap(),
                    scan["time"].as_str().unwrap(),
                    scan["gmtOffset"].as_str().unwrap(),
                ), "%Y-%m-%d %H:%M:%S %:z")?;

            tracks.push(tracker::TrackingDetail {
                time: datetime.format("%Y-%m-%d %H:%M:%S").to_string(),
                message: scan["status"].as_str().and_then(|s| Some(s.to_string())),
                status: None,
                location: Some(scan["scanLocation"].as_str().unwrap_or("FedEx").to_string()),
                live_tracking_url: None,
            });
        }

        Ok(tracker::TrackingInfo {
            id: Self::id().to_string(),
            name: Self::name().to_string(),
            url: format!("https://www.fedex.com/fedextrack/?action=track&trackingnumber={}&cntry_code=kr&locale=ko_kr", tracking_number),
            tracking_number: tracking_number.to_string(),
            is_delivered: package_info["isDelivered"].as_bool().unwrap(),
            sender: Some(format!(
                "{}, {} {}",
                package_info["shipperCity"].as_str().unwrap(),
                package_info["shipperStateCD"].as_str().unwrap(),
                package_info["shipperCntryCD"].as_str().unwrap(),
            )),
            receiver: Some(format!(
                "{}, {} {}",
                package_info["recipientCity"].as_str().unwrap(),
                package_info["recipientStateCD"].as_str().unwrap(),
                package_info["recipientCntryCD"].as_str().unwrap(),
            )),
            product: None,
            tracks,
        })
    }
}
