use async_trait::async_trait;
use chrono::TimeZone;
use chrono_tz::Asia::Seoul;
use nipper::Document;
use regex::Regex;

use crate::{
    structs::{Courier, TrackingError},
    tracker,
};

pub struct EpostEMS {}

#[async_trait]
impl Courier for EpostEMS {
    fn id() -> &'static str {
        "kr.epostems"
    }

    fn name() -> &'static str {
        "우체국 EMS"
    }

    fn validate(tracking_number: &str) -> bool {
        let regex = Regex::new(r"\w{2}\d{9}\w{2}").unwrap();
        regex.is_match(tracking_number)
    }

    async fn track(tracking_number: &str) -> crate::structs::TrackingResult {
        if !Self::validate(tracking_number) {
            return Err(TrackingError::WrongTrackingNumber(
                "영문 2자리, 숫자 9자리, 영문 2자리".to_string(),
            ));
        }

        let url = format!(
            "https://service.epost.go.kr/trace.RetrieveEmsRigiTraceList.comm?POST_CODE={}",
            tracking_number
        );

        let body = reqwest::get(&url).await?.text().await?;

        if body.contains("배달정보를 찾지 못했습니다") {
            return Err(TrackingError::NotExistsTrackingNumber);
        }

        let document = Document::from(&body);

        let mut tracks: Vec<tracker::TrackingDetail> = vec![];

        for element in document
            .select("#print > table.table_col.detail_off.ma_t_5 > tbody > tr")
            .iter()
        {
            let datetime = Seoul.datetime_from_str(
                &element.select("td:nth-child(1)").text(),
                "%Y.%m.%d %H:%M",
            )?;

            tracks.push(tracker::TrackingDetail {
                time: datetime.format("%Y-%m-%d %H:%M:%S").to_string(),
                message: None,
                status: Some(element.select("td:nth-child(2)").text().trim().to_string()),
                location: Some(element.select("td:nth-child(3)").text().trim().to_string()),
                live_tracking_url: None,
            });
        }

        Ok(tracker::TrackingInfo {
            id: Self::id().to_string(),
            name: Self::name().to_string(),
            url: url.to_string(),
            tracking_number: document
                .select("#print > table > tbody > tr > th")
                .text()
                .to_string(),
            is_delivered: document
                .select("#print > table > tbody > tr > td:nth-child(5)")
                .text()
                .contains("배달완료"),
            sender: None,
            receiver: None,
            product: None,
            tracks,
        })
    }
}
