use async_trait::async_trait;
use nipper::Document;
use serde_json::Value;

use crate::structs::{Courier, TrackingDetail, TrackingError, TrackingInfo, TrackingResult};

pub struct Cjlogistics {}

impl Cjlogistics {
    async fn get_csrf(url: &str) -> Result<(String, String), reqwest::Error> {
        let response = reqwest::Client::builder()
            .cookie_store(true)
            .build()?
            .get(url)
            .send()
            .await?;
        let cookies = response
            .cookies()
            .map(|c| format!("{}={}", c.name(), c.value()))
            .collect::<Vec<_>>()
            .join(";");
        let document = Document::from(&response.text().await?);
        let csrf = document.select("input[name='_csrf']").attr("value").unwrap().to_string();
        Ok((csrf, cookies))
    }
}

#[async_trait]
impl Courier for Cjlogistics {
    fn id() -> &'static str {
        "kr.cjlogistics"
    }

    fn name() -> &'static str {
        "CJ대한통운"
    }

    fn validate(tracking_number: &str) -> bool {
        tracking_number.parse::<u64>().is_ok()
            && (tracking_number.len() == 10 || tracking_number.len() == 12)
    }

    async fn track(tracking_number: &str) -> TrackingResult {
        if !Self::validate(tracking_number) {
            return Err(TrackingError::WrongTrackingNumber);
        }
        let client = reqwest::Client::new();
        let url = "https://www.cjlogistics.com/ko/tool/parcel/tracking";

        let (csrf, cookies) = Self::get_csrf(url).await?;

        let json: Value = client
            .post("https://www.cjlogistics.com/ko/tool/parcel/tracking-detail")
            .header("Cookie", cookies)
            .form(&[("paramInvcNo", tracking_number), ("_csrf", &csrf)])
            .send()
            .await?
            .json()
            .await?;

        println!("{}", json);

        if json["parcelResultMap"]["resultList"].as_array().unwrap().is_empty() {
            return Err(TrackingError::NotExistsTrackingNumber);
        }

        let mut tracks: Vec<TrackingDetail> = vec![];

        for element in json["parcelDetailResultMap"]["resultList"].as_array().unwrap() {
            tracks.push(TrackingDetail {
                time: element["dTime"].as_str().unwrap().to_string(),
                message: Some(element["crgNm"].as_str().unwrap().replace(".(", ". (").to_string()),
                status: Some(element["scanNm"].as_str().unwrap().to_string()),
                location: Some(element["regBranNm"].as_str().unwrap().to_string()),
                live_tracking_url: None,
            });
        }

        let detail = &json["parcelResultMap"]["resultList"][0];

        Ok(TrackingInfo {
            id: Self::id().to_string(),
            name: Self::name().to_string(),
            url: url.to_string(),
            tracking_number: detail["invcNo"].as_str().unwrap().to_string(),
            is_delivered: detail["nsDlvNm"].as_str().unwrap() == "91",
            sender: Some(detail["sendrNm"].as_str().unwrap().to_string()),
            receiver: Some(detail["rcvrNm"].as_str().unwrap().to_string()),
            product: Some(detail["itemNm"].as_str().unwrap().to_string()),
            tracks,
        })
    }
}
