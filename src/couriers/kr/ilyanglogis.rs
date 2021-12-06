use async_trait::async_trait;
use chrono::TimeZone;
use chrono_tz::Asia::Seoul;
use nipper::Document;

use crate::{
    structs::{Courier, TrackingError},
    tracker,
};

pub struct IlyangLogis {}

#[async_trait]
impl Courier for IlyangLogis {
    fn id() -> &'static str {
        "kr.ilyanglogis"
    }

    fn name() -> &'static str {
        "일양로지스"
    }

    fn validate(tracking_number: &str) -> bool {
        tracking_number.parse::<u64>().is_ok() && tracking_number.len() == 10
    }

    async fn track(tracking_number: &str) -> crate::structs::TrackingResult {
        if !Self::validate(tracking_number) {
            return Err(TrackingError::WrongTrackingNumber(
                "숫자 10자리".to_string(),
            ));
        }

        let url = format!(
            "https://www.ilyanglogis.com/functionality/tracking_result.asp?hawb_no={}",
            tracking_number
        );

        let body = reqwest::get(&url).await?.text().await?;
    
        if body.contains("해당 자료가 없습니다") {
            return Err(TrackingError::NotExistsTrackingNumber);
        }

        let document = Document::from(&body);

        let mut tracks: Vec<tracker::TrackingDetail> = vec![];

        for element in document.select("#popContainer > div > table > tbody > tr").iter() {
            let datetime = Seoul.datetime_from_str(
                &format!(
                    "{} {}",
                    element.select("td:nth-child(1)").text(),
                    element.select("td:nth-child(2)").text()
                ),
                "%Y-%m-%d %H:%M",
            )?;

            tracks.push(tracker::TrackingDetail {
                time: datetime.format("%Y-%m-%d %H:%M:%S").to_string(),
                message: None,
                status: Some(element.select("td:nth-child(3)").text().to_string()),
                location: Some(element.select("td:nth-child(4)").text().to_string()),
                live_tracking_url: None,
            });
        }

        Ok(tracker::TrackingInfo {
            id: Self::id().to_string(),
            name: Self::name().to_string(),
            url: url.to_string(),
            tracking_number: document
                .select("#popContainer > div > dl > dd:nth-child(2)")
                .text()
                .to_string(),
            is_delivered: document
                .select("#popContainer > div > dl > dd:nth-child(10) > strong")
                .text()
                .contains("배송완료"),
            sender: Some(
                document
                    .select("#popContainer > div > dl > dd:nth-child(4)")
                    .text()
                    .trim()
                    .to_string(),
            ),
            receiver: Some(
                document
                    .select("#popContainer > div > dl > dd:nth-child(6)")
                    .text()
                    .trim()
                    .to_string(),
            ),
            product: None,
            tracks,
        })
    }
}
