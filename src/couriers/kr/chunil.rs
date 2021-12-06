use async_trait::async_trait;
use chrono::TimeZone;
use chrono_tz::Asia::Seoul;
use nipper::Document;

use crate::{
    structs::{Courier, TrackingError},
    tracker,
};

pub struct Chunil {}

#[async_trait]
impl Courier for Chunil {
    fn id() -> &'static str {
        "kr.chunil"
    }

    fn name() -> &'static str {
        "천일택배"
    }

    fn validate(tracking_number: &str) -> bool {
        tracking_number.parse::<u64>().is_ok() && tracking_number.len() == 11
    }

    async fn track(tracking_number: &str) -> crate::structs::TrackingResult {
        if !Self::validate(tracking_number) {
            return Err(TrackingError::WrongTrackingNumber(
                "숫자 11자리".to_string(),
            ));
        }

        let url = format!(
            "https://www.chunil.co.kr/HTrace/HTrace.jsp?transNo={}",
            tracking_number
        );

        let body = reqwest::get(&url)
            .await?
            .text()
            .await?;

        if body.contains("결과가 없습니다") {
            return Err(TrackingError::NotExistsTrackingNumber);
        }

        let document = Document::from(&body);

        let mut tracks: Vec<tracker::TrackingDetail> = vec![];

        for element in document
            .select("#tracking > tbody > tr")
            .iter()
        {
            if element.html().contains("날짜") {
                continue;
            }

            let datetime = Seoul.datetime_from_str(
                &format!("{} 00:00:00", element.select("td:nth-child(1)").text()),
                "%Y-%m-%d %H:%M:%S",
            )?;

            tracks.push(tracker::TrackingDetail {
                time: datetime.format("%Y-%m-%d %H:%M:%S").to_string(),
                message: None,
                status: Some(element.select("td:nth-child(4)").text().to_string()),
                location: Some(element.select("td:nth-child(2)").text().to_string()),
                live_tracking_url: None,
            });
        }

        Ok(tracker::TrackingInfo {
            id: Self::id().to_string(),
            name: Self::name().to_string(),
            url: url.to_string(),
            tracking_number: tracking_number.to_string(),
            is_delivered: document
                .select("table.table01 > tbody > tr:nth-child(2) > td:nth-child(2) > table.table02 > tbody > tr:nth-child(2) > td:nth-child(2)")
                .text()
                .contains("배송완료"),
            sender: Some(
                document
                    .select("table.table01 > tbody > tr:nth-child(1) > td:nth-child(1) > table.table02 > tbody > tr:nth-child(1) > td:nth-child(2)")
                    .text()
                    .to_string(),
            ),
            receiver: Some(
                document
                .select("table.table01 > tbody > tr:nth-child(1) > td:nth-child(2) > table.table02 > tbody > tr:nth-child(1) > td:nth-child(2)")
                    .text()
                    .to_string(),
            ),
            product: Some(
                document
                    .select("table.table01 > tbody > tr:nth-child(2) > td:nth-child(1) > table.table02 > tbody > tr:nth-child(1) > td:nth-child(2)")
                    .text()
                    .to_string(),
            ),
            tracks,
        })
    }
}
