use async_trait::async_trait;
use chrono::TimeZone;
use chrono_tz::Asia::Seoul;
use nipper::Document;
use regex::Regex;

use crate::{
    structs::{Courier, TrackingError},
    tracker,
};

pub struct Epost {}

#[async_trait]
impl Courier for Epost {
    fn id() -> &'static str {
        "kr.epost"
    }

    fn name() -> &'static str {
        "우체국택배"
    }

    fn validate(tracking_number: &str) -> bool {
        tracking_number.parse::<u64>().is_ok() && tracking_number.len() == 13
    }

    async fn track(tracking_number: &str) -> crate::structs::TrackingResult {
        if !Self::validate(tracking_number) {
            return Err(TrackingError::WrongTrackingNumber(
                "숫자 13자리".to_string(),
            ));
        }

        let url = format!(
            "https://service.epost.go.kr/trace.RetrieveDomRigiTraceList.comm?sid1={}",
            tracking_number
        );

        let body = reqwest::get(&url).await?.text().await?;

        if body.contains("배달정보를 찾지 못했습니다") {
            return Err(TrackingError::NotExistsTrackingNumber);
        }

        let document = Document::from(&body);

        let mut tracks: Vec<tracker::TrackingDetail> = vec![];
        let space_regex = Regex::new(r"\s+")?;

        for element in document.select("#processTable > tbody > tr").iter() {
            let status = element
                .select("td:nth-child(4)")
                .text()
                .trim()
                .replace(&['\n', '\t'][..], "");
            let status = space_regex.replace_all(&status, " ").to_string();
            let (status, _) = status
                .trim()
                .split_once(" (")
                .unwrap_or_else(|| (&status, ""));

            let location = element.select("td:nth-child(3)").text();
            let (location, _) = location
                .trim()
                .split_once('\n')
                .unwrap_or_else(|| (location.trim(), ""));

            let datetime = Seoul.datetime_from_str(
                &format!(
                    "{} {}",
                    element.select("td:nth-child(1)").text(),
                    element.select("td:nth-child(2)").text()
                ),
                "%Y.%m.%d %H:%M",
            )?;

            tracks.push(tracker::TrackingDetail {
                time: datetime.format("%Y-%m-%d %H:%M:%S").to_string(),
                message: None,
                status: Some(status.to_string()),
                location: Some(location.to_string()),
                live_tracking_url: None,
            });
        }

        let sender_regex = Regex::new("<td>(.+)<br>")?;
        let sender_html = document
            .select("#print > table > tbody > tr > td:nth-child(2)")
            .html()
            .to_string();
        let sender = sender_regex
            .captures(&sender_html)
            .unwrap()
            .get(1)
            .map_or("", |c| c.as_str());

        Ok(tracker::TrackingInfo {
            id: Self::id().to_string(),
            name: Self::name().to_string(),
            url: url.to_string(),
            tracking_number: document
                .select("#print > table > tbody > tr > th")
                .text()
                .to_string(),
            is_delivered: document
                .select("#print > table > tbody > tr > td:nth-child(6)")
                .text()
                .contains("배달완료"),
            sender: Some(sender.to_string()),
            receiver: Some(
                document
                    .select("#print > table > tbody > tr > td:nth-child(3)")
                    .text()
                    .trim()
                    .to_string(),
            ),
            product: None,
            tracks,
        })
    }
}
