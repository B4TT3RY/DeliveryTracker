use anyhow::{anyhow, Result};
use async_trait::async_trait;
use nipper::Document;
use regex::Regex;

use crate::{
    couriers::courier::{Courier, CourierType},
    get_html_string,
    status_struct::{DeliveryStatus, StateType, TrackingStatus},
};

pub struct ILogen {
    pub tracking_number: String,
}

#[async_trait]
impl Courier for ILogen {
    fn get_url() -> &'static str {
        "https://www.ilogen.com/web/personal/trace/"
    }

    fn get_id() -> &'static str {
        "kr.ilogen"
    }

    fn get_name() -> &'static str {
        "로젠택배"
    }

    async fn validate(&self) -> Result<&Self> {
        if !Regex::new(r#"^(\d{11})$"#)?.is_match(&self.tracking_number) {
            return Err(anyhow!("운송장번호 11자리를 입력해주세요."));
        }
        Ok(self)
    }

    async fn track(&self) -> Result<DeliveryStatus> {
        let response = surf::get(format!("{}{}", Self::get_url(), &self.tracking_number))
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.90 Safari/537.36")
            .recv_string()
            .await
            .map_err(|err| anyhow!(err))?;
        let document = Document::from(&response);

        if document.select(".empty").exists() {
            return Err(anyhow!(
                "{} {} 운송장 번호로 조회된 결과가 없습니다.",
                Self::get_name(),
                &self.tracking_number
            ));
        }

        let product = get_html_string!(
            document,
            "table.horizon.pdInfo > tbody > tr:nth-child(1) > td:nth-child(4)"
        );
        let sender = get_html_string!(
            document,
            "table.horizon.pdInfo > tbody > tr:nth-child(4) > td:nth-child(2)"
        );
        let receiver = get_html_string!(
            document,
            "table.horizon.pdInfo > tbody > tr:nth-child(4) > td:nth-child(4)"
        );

        let mut tracks: Vec<TrackingStatus> = Vec::new();
        for element in document.select("table.data.tkInfo > tbody > tr").iter() {
            let status = get_html_string!(element, "td:nth-child(3)");
            tracks.push(TrackingStatus {
                state: StateType::to_type(
                    CourierType::get_courier(Self::get_id().to_string(), None)?,
                    &status,
                ),
                time: get_html_string!(element, "td:nth-child(1)"),
                location: Some(get_html_string!(element, "td:nth-child(2)")),
                status: status.clone(),
                message: if status == "배송출고" {
                    Some(
                        format!(
                            "{} ({} 배송 예정)",
                            get_html_string!(element, "td:nth-child(4)"),
                            get_html_string!(element, "td:nth-child(6)").replace("배송", ""),
                        )
                        .to_string(),
                    )
                } else {
                    Some(get_html_string!(element, "td:nth-child(4)"))
                },
            });
        }

        Ok(DeliveryStatus {
            id: Self::get_id().to_string(),
            name: Self::get_name().to_string(),
            tracking_number: self.tracking_number.clone(),
            sender: Some(sender),
            receiver: Some(receiver),
            product: Some(product),
            tracks: Some(tracks),
        })
    }
}
