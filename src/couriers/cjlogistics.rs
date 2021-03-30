use anyhow::Result;
use async_trait::async_trait;
use scraper::{Html, Selector};

use crate::{
    couriers::courier::Courier, delivery_status::DeliveryStatus, get_html_string,
    tracking_status::TrackingStatus,
};

pub struct CJLogistics {}

#[async_trait]
impl Courier for CJLogistics {
    fn get_url() -> &'static str {
        "https://www.doortodoor.co.kr/parcel/doortodoor.do"
    }

    fn get_name() -> &'static str {
        "CJ대한통운"
    }

    async fn track(tracking_number: String) -> Result<DeliveryStatus> {
        let response = surf::post(CJLogistics::get_url())
            .body(format!("fsp_action=PARC_ACT_002&fsp_cmd=retrieveInvNoACT2&invc_no={}", tracking_number))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Referer", "https://www.doortodoor.co.kr/parcel/pa_004.jsp")
            .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_6) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0.3 Safari/605.1.15")
            .recv_string()
            .await
            .unwrap();
        let document = Html::parse_document(&response);

        let tracking_number = get_html_string!(document, ".last_b:nth-child(1)");
        let sender = get_html_string!(document, ".last_b:nth-child(2)");
        let receiver = get_html_string!(document, ".last_b:nth-child(3)");
        let product = get_html_string!(document, ".last_b:nth-child(4)");

        let last_track = {
            let selector = Selector::parse("#tabContents > ul > li.first.focus > div > div:nth-child(2) > div > table > tbody > tr:last-child").unwrap();
            let parent = document.select(&selector).next().unwrap();
            TrackingStatus {
                time: get_html_string!(parent, ".last_b:nth-child(2)"),
                location: get_html_string!(parent, ".last_b:nth-child(4)"),
                status: get_html_string!(parent, ".last_b:nth-child(1)"),
                message: get_html_string!(parent, ".last_b:nth-child(3)"),
            }
        };

        Ok(DeliveryStatus {
            tracking_number,
            sender,
            receiver,
            product,
            last_track,
        })
    }
}
