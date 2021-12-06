use async_trait::async_trait;
use chrono::TimeZone;
use chrono_tz::Asia::Seoul;
use nipper::Document;

use crate::{
    structs::{Courier, TrackingError},
    tracker,
};

pub struct Logen {}

#[async_trait]
impl Courier for Logen {
    fn id() -> &'static str {
        "kr.logen"
    }

    fn name() -> &'static str {
        "로젠택배"
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
            "https://www.ilogen.com/web/personal/trace/{}",
            tracking_number
        );

        let body = reqwest::get(&url).await?.text().await?;

        if body.contains("배송자료를 조회할 수 없습니다") {
            return Err(TrackingError::NotExistsTrackingNumber);
        }

        let document = Document::from(&body);

        let mut tracks: Vec<tracker::TrackingDetail> = vec![];

        for element in document.select("table.data.tkInfo > tbody > tr").iter() {
            let datetime = Seoul
                .datetime_from_str(&element.select("td:nth-child(1)").text(), "%Y.%m.%d %H:%M")?;

            let status = element.select("td:nth-child(3)").text().trim().to_string();

            let extra_message = if status == "배송출고" {
                format!(
                    " ({} 배달예정)",
                    element.select("td:nth-child(6)").text().trim()
                )
            } else {
                String::new()
            };

            tracks.push(tracker::TrackingDetail {
                time: datetime.format("%Y-%m-%d %H:%M:%S").to_string(),
                message: Some(format!(
                    "{}{}",
                    element.select("td:nth-child(4)").text().trim(),
                    extra_message
                )),
                status: Some(status),
                location: Some(element.select("td:nth-child(2)").text().to_string()),
                live_tracking_url: None,
            });
        }

        Ok(tracker::TrackingInfo {
            id: Self::id().to_string(),
            name: Self::name().to_string(),
            url: url.to_string(),
            tracking_number: tracking_number.to_string(),
            is_delivered: document.select("li.on").text().contains("배송완료"),
            sender: Some(
                document
                    .select("table.horizon.pdInfo > tbody > tr:nth-child(4) > td:nth-child(2)")
                    .text()
                    .to_string(),
            ),
            receiver: Some(
                document
                    .select("table.horizon.pdInfo > tbody > tr:nth-child(4) > td:nth-child(4)")
                    .text()
                    .to_string(),
            ),
            product: Some(
                document
                    .select("table.horizon.pdInfo > tbody > tr:nth-child(1) > td:nth-child(4)")
                    .text()
                    .to_string(),
            ),
            tracks,
        })
    }
}
