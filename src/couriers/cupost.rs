use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use regex::Regex;
use scraper::{Html, Selector};

use crate::{
    couriers::courier::{Courier, CourierType},
    get_html_string,
    status_struct::{DeliveryStatus, TrackingStatus, StateType},
};

pub struct CUPost {
    pub tracking_number: String,
}

#[async_trait]
impl Courier for CUPost {
    fn get_url() -> &'static str {
        "https://www.cupost.co.kr/postbox/delivery/localResult.cupost"
    }

    fn get_id() -> &'static str {
        "kr.cupost"
    }

    fn get_name() -> &'static str {
        "CU 편의점택배"
    }

    async fn validate(&self) -> Result<&Self> {
        if !Regex::new(r#"^(\d{1,12})$"#)?.is_match(&self.tracking_number) {
            return Err(anyhow!("운송장번호를 최대 12자리까지 입력해주세요."));
        }
        Ok(self)
    }

    async fn track(&self) -> Result<DeliveryStatus> {
        let response = surf::post(Self::get_url())
            .body(format!("invoice_no={}", self.tracking_number))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.90 Safari/537.36")
            .header("Referer", "https://www.cupost.co.kr/postbox/delivery/localResult.cupost")
            .recv_string()
            .await
            .map_err(|err| anyhow!(err))?;

        if response.contains("<iframe") {
            let cj = CourierType::track("kr.cjlogistics".to_string(), self.tracking_number.clone())
                .await;
            if let Err(err) = cj {
                return Err(err);
            }
            let mut cj = cj?;
            cj.id = Self::get_id().to_string();
            cj.name = format!("{} (CJ대한통운 국내택배)", Self::get_name()).to_string();
            return Ok(cj);
        }

        let document: Html = Html::parse_document(&response);

        if document
            .select(&Selector::parse(".ac").unwrap())
            .next()
            .is_some()
        {
            return Err(anyhow!(
                "{} {} 운송장 번호로 조회된 결과가 없습니다.",
                Self::get_name(),
                &self.tracking_number
            ));
        }

        let tracking_number = get_html_string!(
            document,
            "#gotoMainContents > table:nth-child(5) > tbody > tr:nth-child(1) > td:nth-child(2)"
        );
        let product = get_html_string!(
            document,
            "#gotoMainContents > table:nth-child(5) > tbody > tr:nth-child(1) > td:nth-child(4)"
        );
        let sender = format!(
            "{} ({})",
            get_html_string!(document, "#gotoMainContents > table:nth-child(5) > tbody > tr:nth-child(3) > td:nth-child(2)"),
            get_html_string!(document, "#gotoMainContents > table:nth-child(5) > tbody > tr:nth-child(4) > td")
        );
        let receiver = format!(
            "{} ({})",
            get_html_string!(document, "#gotoMainContents > table:nth-child(5) > tbody > tr:nth-child(3) > td:nth-child(4)"),
            get_html_string!(document, "#gotoMainContents > table:nth-child(5) > tbody > tr:nth-child(5) > td:nth-child(2)")
        );

        let regex = Regex::new(r#"^(\d{4}.\d{2}.\d{2})(.|\n)*(\d{2}:\d{2})$"#)?;
        let mut tracks: Vec<TrackingStatus> = Vec::new();
        let selector =
            Selector::parse("#gotoMainContents > table:nth-child(10) > tbody > tr").unwrap();
        for element in document.select(&selector) {
            let status = get_html_string!(element, "td:nth-child(3)");
            let time = get_html_string!(element, "td:nth-child(1)");
            let cap = regex.captures(&time).unwrap();
            let time = format!(
                "{} {}",
                cap.get(1).map_or("", |m| m.as_str()),
                cap.get(3).map_or("", |m| m.as_str())
            );
            tracks.push(TrackingStatus {
                state: StateType::to_type(CourierType::get_courier(Self::get_id().to_string(), None)?, &status),
                time,
                location: get_html_string!(element, "td:nth-child(2)"),
                status,
                message: None,
            });
        }

        Ok(DeliveryStatus {
            id: Self::get_id().to_string(),
            name: format!("{} (CU끼리택배)", Self::get_name()).to_string(),
            tracking_number,
            sender: Some(sender),
            receiver: Some(receiver),
            product: Some(product),
            tracks: Some(tracks),
        })
    }
}
