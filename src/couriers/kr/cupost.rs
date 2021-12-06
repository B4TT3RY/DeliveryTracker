use async_trait::async_trait;
use chrono::TimeZone;
use chrono_tz::Asia::Seoul;
use nipper::Document;

use crate::{
    structs::{Courier, TrackingError},
    tracker,
};

use super::cjlogistics::Cjlogistics;

pub struct Cupost {}

#[async_trait]
impl Courier for Cupost {
    fn id() -> &'static str {
        "kr.cupost"
    }

    fn name() -> &'static str {
        "CU 편의점택배"
    }

    fn validate(tracking_number: &str) -> bool {
        tracking_number.parse::<u64>().is_ok() && tracking_number.len() >= 10 && tracking_number.len() <= 12
    }

    async fn track(tracking_number: &str) -> crate::structs::TrackingResult {
        if !Self::validate(tracking_number) {
            return Err(TrackingError::WrongTrackingNumber(
                "숫자 10자리 또는 11자리 또는 12자리".to_string(),
            ));
        }
        
        let body = reqwest::Client::new()
            .post("https://www.cupost.co.kr/postbox/delivery/localResult.cupost")
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.0.4664.45 Safari/537.36")
            .form(&[("invoice_no", tracking_number)])
            .send()
            .await?
            .text()
            .await?;
        
        if body.contains("<iframe") {
            let mut cj = Cjlogistics::track(tracking_number).await?;
            cj.id = Self::id().to_string();
            cj.name = format!("{} (국내택배)", Self::name());
            return Ok(cj);
        }

        if body.contains("조회하신 내용이 없습니다") {
            return Err(TrackingError::NotExistsTrackingNumber);
        }

        let document = Document::from(&body);

        let mut tracks: Vec<tracker::TrackingDetail> = vec![];

        for element in document
            .select("#gotoMainContents > table:nth-child(10) > tbody > tr")
            .iter()
        {
            let datetime = Seoul.datetime_from_str(
                element.select("td:nth-child(1)").text().trim(),
                "%Y.%m.%d %H:%M",
            )?;

            tracks.push(tracker::TrackingDetail {
                time: datetime.format("%Y-%m-%d %H:%M:%S").to_string(),
                message: Some(element.select("td:nth-child(3)").text().trim().to_string()),
                status: None,
                location: Some(element.select("td:nth-child(2)").text().trim().to_string()),
                live_tracking_url: None,
            });
        }

        Ok(tracker::TrackingInfo {
            id: Self::id().to_string(),
            name: Self::name().to_string(),
            url: "https://www.cupost.co.kr/postbox/delivery/local.cupost".to_string(),
            tracking_number: document
                .select("#gotoMainContents > table:nth-child(5) > tbody > tr:nth-child(1) > td:nth-child(2)")
                .text()
                .to_string(),
            is_delivered: document
                .select("#local_result > tbody > tr:nth-child(1) > td:nth-child(9) > img")
                .attr("src")
                .unwrap()
                .contains("step5_on"),
            sender: Some(
                document
                    .select("#gotoMainContents > table:nth-child(5) > tbody > tr:nth-child(3) > td:nth-child(2)")
                    .text()
                    .trim()
                    .to_string(),
            ),
            receiver: Some(
                document
                    .select("#gotoMainContents > table:nth-child(5) > tbody > tr:nth-child(3) > td:nth-child(4)")
                    .text()
                    .trim()
                    .to_string(),
            ),
            product: Some(
                document
                    .select("#gotoMainContents > table:nth-child(5) > tbody > tr:nth-child(1) > td:nth-child(4)")
                    .text()
                    .to_string()
            ),
            tracks,
        })
    }
}
