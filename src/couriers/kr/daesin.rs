use async_trait::async_trait;
use chrono::TimeZone;
use chrono_tz::Asia::Seoul;
use nipper::Document;

use crate::{
    structs::{Courier, TrackingError},
    tracker,
};

pub struct Daesin {}

#[async_trait]
impl Courier for Daesin {
    fn id() -> &'static str {
        "kr.daesin"
    }

    fn name() -> &'static str {
        "대신택배"
    }

    fn validate(tracking_number: &str) -> bool {
        tracking_number.parse::<u64>().is_ok()
            && (tracking_number.len() == 12 || tracking_number.len() == 13)
    }

    async fn track(tracking_number: &str) -> crate::structs::TrackingResult {
        if !Self::validate(tracking_number) {
            return Err(TrackingError::WrongTrackingNumber(
                "숫자 12자리 또는 13자리".to_string(),
            ));
        }

        let url = format!(
            "https://www.ds3211.co.kr/freight/internalFreightSearch.ht?billno={}",
            tracking_number
        );

        let body = reqwest::get(&url).await?.text().await?;

        if body.contains("검색하신 운송장번호로 운송된 내역이 없습니다") {
            return Err(TrackingError::NotExistsTrackingNumber);
        }

        let document = Document::from(&body);

        let mut tracks: Vec<tracker::TrackingDetail> = vec![];

        for element in document
            .select("#printarea > table:nth-child(5) > tbody > tr")
            .iter()
        {
            if element.html().contains("th") {
                continue;
            }

            let dealer_type = element.select("td:nth-child(1)").text().to_string();

            let datetime = Seoul.datetime_from_str(
                &element.select("td:nth-child(4)").text(),
                "%Y-%m-%d %H:%M",
            )?;

            tracks.push(tracker::TrackingDetail {
                time: datetime.format("%Y-%m-%d %H:%M:%S").to_string(),
                message: None,
                status: Some(
                    if dealer_type == "발송취급점" {
                        "인수".to_string()
                    } else {
                        "도착".to_string()
                    }
                ),
                location: Some(format!("[{}] {}", element.select("td:nth-child(1)").text().trim(), element.select("td:nth-child(2)").text().trim())),
                live_tracking_url: None,
            });

            let start_time = element.select("td:nth-child(5)").text().to_string();

            if !start_time.is_empty() {
                let datetime = Seoul.datetime_from_str(
                    &start_time,
                    "%Y-%m-%d %H:%M",
                )?;
    
                tracks.push(tracker::TrackingDetail {
                    time: datetime.format("%Y-%m-%d %H:%M:%S").to_string(),
                    message: None,
                    status: Some(
                        if dealer_type == "도착취급점" {
                            "배송완료".to_string()
                        } else {
                            "출발".to_string()
                        }
                    ),
                    location: Some(format!("[{}] {}", element.select("td:nth-child(1)").text().trim(), element.select("td:nth-child(2)").text().trim())),
                    live_tracking_url: None,
                });
            }
        }

        Ok(tracker::TrackingInfo {
            id: Self::id().to_string(),
            name: Self::name().to_string(),
            url: url.to_string(),
            tracking_number: tracking_number.to_string(),
            is_delivered: document
                .select("#printarea > table:nth-child(5) > tbody > tr:last-child > td:nth-child(6)")
                .text()
                .contains("배송완료"),
            sender: Some(
                document
                    .select("#printarea > table.depth01.tmar_15.bmar_50 > tbody > tr:nth-child(1) > td:nth-child(2)")
                    .text()
                    .to_string()
            ),
            receiver: Some(
                document
                    .select("#printarea > table.depth01.tmar_15.bmar_50 > tbody > tr:nth-child(2) > td:nth-child(2)")
                    .text()
                    .to_string()
            ),
            product: Some(
                document
                    .select("#printarea > table.depth01.tmar_15.bmar_50 > tbody > tr:nth-child(3) > td:nth-child(2)")
                    .text()
                    .to_string()
            ),
            tracks,
        })
    }
}
