use async_trait::async_trait;
use chrono::TimeZone;
use chrono_tz::Asia::Seoul;
use regex::Regex;
use serde_json::Value;

use crate::{
    structs::{Courier, TrackingError},
    tracker,
};

pub struct Gspostbox {}

#[async_trait]
impl Courier for Gspostbox {
    fn id() -> &'static str {
        "kr.gspostbox"
    }

    fn name() -> &'static str {
        "GS Postbox"
    }

    fn validate(tracking_number: &str) -> bool {
        tracking_number.parse::<u64>().is_ok()
            && (tracking_number.len() == 10 || tracking_number.len() == 12)
    }

    async fn track(tracking_number: &str) -> crate::structs::TrackingResult {
        if !Self::validate(tracking_number) {
            return Err(TrackingError::WrongTrackingNumber(
                "숫자 10자리 또는 12자리".to_string(),
            ));
        }

        let url = format!(
            "https://www.cvsnet.co.kr/invoice/tracking.do?invoice_no={}",
            tracking_number
        );

        let body = reqwest::Client::new()
            .get(&url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.0.4664.45 Safari/537.36")
            .send()
            .await?
            .text()
            .await?;

        if body.contains("해당 운송장의 배송정보를 조회할 수 없습니다") {
            return Err(TrackingError::NotExistsTrackingNumber);
        }

        let regex = Regex::new("(var trackingInfo = )(.+)(;)")?;
        let capture = regex.captures(&body).unwrap();
        let json = capture.get(2).map_or("", |m| m.as_str());
    
        let json: Value = serde_json::from_str(json)?;

        let mut tracks: Vec<tracker::TrackingDetail> = vec![];

        for element in json["trackingDetails"].as_array().unwrap() {
            let datetime = Seoul.datetime_from_str(
                &element["transTime"].as_str().unwrap(),
                "%Y-%m-%dT%H:%M:%S",
            )?;

            tracks.push(tracker::TrackingDetail {
                time: datetime.format("%Y-%m-%d %H:%M:%S").to_string(),
                message: None,
                status: Some(element["transKind"].as_str().unwrap().replace("  ", " ")),
                location: Some(element["transWhere"].as_str().unwrap().to_string()),
                live_tracking_url: None,
            });
        }

        Ok(tracker::TrackingInfo {
            id: Self::id().to_string(),
            name: format!("{} {}", Self::name(), json["serviceName"].as_str().unwrap().to_string()),
            url: url.to_string(),
            tracking_number: json["invoiceNo"].as_str().unwrap().to_string(),
            is_delivered: json["latestTrackingDetail"]["transKind"] == "고객전달",
            sender: Some(json["sender"]["name"].as_str().unwrap().to_string()),
            receiver: Some(json["receiver"]["name"].as_str().unwrap().to_string()),
            product: Some(json["goodsName"].as_str().unwrap().to_string()),
            tracks,
        })
    }
}
