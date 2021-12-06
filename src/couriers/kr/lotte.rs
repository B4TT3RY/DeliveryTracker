use async_trait::async_trait;
use chrono::TimeZone;
use chrono_tz::Asia::Seoul;
use nipper::Document;

use crate::{
    structs::{Courier, TrackingError},
    tracker,
};

pub struct Lotte {}

#[async_trait]
impl Courier for Lotte {
    fn id() -> &'static str {
        "kr.lotte"
    }

    fn name() -> &'static str {
        "롯데택배"
    }

    fn validate(tracking_number: &str) -> bool {
        tracking_number.parse::<u64>().is_ok()
            && (tracking_number.len() == 10
                || tracking_number.len() == 12
                || tracking_number.len() == 13)
    }

    async fn track(tracking_number: &str) -> crate::structs::TrackingResult {
        if !Self::validate(tracking_number) {
            return Err(TrackingError::WrongTrackingNumber(
                "숫자 10자리 또는 숫자 12자리 또는 숫자 13자리".to_string(),
            ));
        }

        let body = reqwest::Client::new()
            .post("https://www.lotteglogis.com/home/reservation/tracking/linkView")
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.0.4664.45 Safari/537.36")
            .form(&[("InvNo", tracking_number)])
            .send()
            .await?
            .text()
            .await?;

        if body.contains("운송장이 등록되지 않았거나") {
            return Err(TrackingError::NotExistsTrackingNumber);
        }

        let document = Document::from(&body);

        let mut tracks: Vec<tracker::TrackingDetail> = vec![];

        for element in document
            .select("#contents > div > div.contArea > table:nth-child(4) > tbody > tr")
            .iter()
        {
            if element.select("td:nth-child(2)").text().contains("--:--") {
                continue;
            }

            let datetime = Seoul
                .datetime_from_str(&element.select("td:nth-child(2)").text(), "%Y-%m-%d %H:%M")?;

            tracks.push(tracker::TrackingDetail {
                time: datetime.format("%Y-%m-%d %H:%M:%S").to_string(),
                message: Some(
                    element
                        .select("td:nth-child(4)")
                        .text()
                        .replace("  ", " ")
                        .replace(".(", ". ("),
                ),
                status: Some(element.select("td:nth-child(1)").text().to_string()),
                location: Some(element.select("td:nth-child(3)").text().trim().to_string()),
                live_tracking_url: None,
            });
        }

        tracks.reverse();

        Ok(tracker::TrackingInfo {
            id: Self::id().to_string(),
            name: Self::name().to_string(),
            url: "https://www.lotteglogis.com/home/main".to_string(),
            tracking_number: document
                .select("#contents > div > div.contArea > table.tblH.mt60 > tbody > tr > td:nth-child(1)")
                .text()
                .to_string(),
            is_delivered: document
                .select("#contents > div > div.contArea > table.tblH.mt60 > tbody > tr > td:nth-child(4)")
                .text()
                .contains("배달완료"),
            sender: Some(
                document
                    .select("#contents > div > div.contArea > table.tblH.mt60 > tbody > tr > td:nth-child(2)")
                    .text()
                    .to_string()
            ),
            receiver: Some(
                document
                    .select("#contents > div > div.contArea > table.tblH.mt60 > tbody > tr > td:nth-child(3)")
                    .text()
                    .to_string()
            ),
            product: None,
            tracks,
        })
    }
}
