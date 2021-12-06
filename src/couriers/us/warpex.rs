use async_trait::async_trait;
use chrono::TimeZone;
use chrono_tz::Asia::Seoul;
use nipper::Document;
use regex::Regex;

use crate::{
    structs::{Courier, TrackingError},
    tracker,
};

pub struct Warpex {}

#[async_trait]
impl Courier for Warpex {
    fn id() -> &'static str {
        "us.warpex"
    }

    fn name() -> &'static str {
        "WarpEX"
    }

    fn validate(tracking_number: &str) -> bool {
        tracking_number.parse::<u64>().is_ok() && tracking_number.len() == 12
    }

    async fn track(tracking_number: &str) -> crate::structs::TrackingResult {
        if !Self::validate(tracking_number) {
            return Err(TrackingError::WrongTrackingNumber("숫자 12자리".to_string()));
        }

        let url = format!("https://packing.warpex.com/api/warpexTrack?wbl={}", tracking_number);

        let body = reqwest::Client::new()
            .get(&url)
            .send()
            .await?
            .text()
            .await?;

        if body.contains("조회된 데이터가 없습니다") {
            return Err(TrackingError::NotExistsTrackingNumber);
        }

        let document = Document::from(&body);

        let mut tracks: Vec<tracker::TrackingDetail> = vec![];
        let message_regex = Regex::new(r"\s+").unwrap();

        for element in document
            .select("#history > ul > li")
            .iter()
        {
            let datetime = Seoul.datetime_from_str(
                &element.select(".date").text(),
                "%Y-%m-%d %p %I:%M:%S",
            )?;

            tracks.push(tracker::TrackingDetail {
                time: datetime.format("%Y-%m-%d %H:%M:%S").to_string(),
                message: Some(
                    message_regex.replace_all(
                    &element
                            .select(".txt")
                            .text()
                            .to_string(),
                    " "
                    )
                    .trim()
                    .to_string()
                ),
                status: None,
                location: None,
                live_tracking_url: None,
            });
        }

        let sender_regex = Regex::new(r"[\n|\t|\s]+").unwrap();
        let sender = document.select("div.Tdate > div > p:nth-child(1) > span").html();
        let (_, sender) = sender.split_once("<br>").unwrap();
        let sender = sender_regex.replace_all(sender, " ");

        let receiver = document.select("div.Tdate > div > p:nth-child(2) > span").html();
        let (_, receiver) = receiver.split_once("<br>").unwrap();

        Ok(tracker::TrackingInfo {
            id: Self::id().to_string(),
            name: Self::name().to_string(),
            url: url.to_string(),
            tracking_number: tracking_number.to_string(),
            is_delivered: document
                .select("body > section > section > div:nth-child(2) > div.step > p > img")
                .attr("src")
                .unwrap()
                .contains("step5"),
            sender: Some(sender.replace("</span>", "").trim().to_string()),
            receiver: Some(receiver.replace("</span>", "").trim().to_string()),
            product: None,
            tracks,
        })
    }
}
