use async_trait::async_trait;
use chrono::TimeZone;
use chrono_tz::Asia::Seoul;
use nipper::Document;

use crate::{
    structs::{Courier, TrackingError},
    tracker,
};

pub struct Hanjin {}

#[async_trait]
impl Courier for Hanjin {
    fn id() -> &'static str {
        "kr.hanjin"
    }

    fn name() -> &'static str {
        "한진택배"
    }

    fn validate(tracking_number: &str) -> bool {
        tracking_number.parse::<u64>().is_ok()
            && (tracking_number.len() == 12 || tracking_number.len() == 14)
    }

    async fn track(tracking_number: &str) -> crate::structs::TrackingResult {
        if !Self::validate(tracking_number) {
            return Err(TrackingError::WrongTrackingNumber(
                "숫자 12자리 또는 숫자 14자리".to_string(),
            ));
        }

        let url = format!(
            "https://www.hanjin.co.kr/kor/CMS/DeliveryMgr/WaybillResult.do?mCode=MN038&schLang=KR&wblnum={}",
            tracking_number
        );

        let body = reqwest::Client::new()
            .get(&url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.0.4664.45 Safari/537.36")
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
            .select("#delivery-wr > div > div.waybill-tbl > table > tbody > tr")
            .iter()
        {
            let datetime = Seoul.datetime_from_str(
                &format!(
                    "{} {}",
                    element.select("td.w-date").text(),
                    element.select("td.w-time").text()
                ),
                "%Y-%m-%d %H:%M",
            )?;

            tracks.push(tracker::TrackingDetail {
                time: datetime.format("%Y-%m-%d %H:%M:%S").to_string(),
                message: None,
                status: Some(element.select(".stateDesc").text().replace("  ", " ")),
                location: Some(element.select("td.w-org").text().to_string()),
                live_tracking_url: None,
            });
        }

        Ok(tracker::TrackingInfo {
            id: Self::id().to_string(),
            name: Self::name().to_string(),
            url: url.to_string(),
            tracking_number: document
                .select(".songjang-num > .num")
                .text()
                .to_string(),
            is_delivered: document.select("li.on > span.num").text().contains("STEP6"),
            sender: Some(
                document
                    .select("td[data-label='보내는 분']")
                    .text()
                    .to_string(),
            ),
            receiver: Some(
                document
                    .select("td[data-label='받는 분']")
                    .text()
                    .to_string(),
            ),
            product: Some(
                document
                    .select("td[data-label='상품명']")
                    .text()
                    .to_string()
            ),
            tracks,
        })
    }
}
