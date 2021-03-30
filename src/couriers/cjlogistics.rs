use anyhow::Result;
use async_trait::async_trait;
use scraper::{Html, Selector};

use crate::{couriers::courier::Courier, delivery_status::DeliveryStatus, tracking_status::TrackingStatus};

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

        let tracking_number = {
            let selector = Selector::parse(".last_b:nth-child(1)").unwrap();
            document.select(&selector)
                .next()
                .unwrap()
                .text()
                .collect::<String>()
                .trim()
                .to_string()
        };

        let sender = {
            let selector = Selector::parse(".last_b:nth-child(2)").unwrap();
            document.select(&selector)
                .next()
                .unwrap()
                .text()
                .collect::<String>()
                .trim()
                .to_string()
        };

        let receiver = {
            let selector = Selector::parse(".last_b:nth-child(3)").unwrap();
            document.select(&selector)
                .next()
                .unwrap()
                .text()
                .collect::<String>()
                .trim()
                .to_string()
        };

        let product = {
            let selector = Selector::parse(".last_b:nth-child(4)").unwrap();
            document.select(&selector)
                .next()
                .unwrap()
                .text()
                .collect::<String>()
                .trim()
                .to_string()
        };

        let last_track = {
            let selector = Selector::parse("#tabContents > ul > li.first.focus > div > div:nth-child(2) > div > table > tbody > tr:last-child").unwrap();
            let parent = document.select(&selector)
                .next()
                .unwrap();
            TrackingStatus {
                time: {
                    let selector = Selector::parse(".last_b:nth-child(2)").unwrap();
                    parent.select(&selector)
                        .next()
                        .unwrap()
                        .text()
                        .collect::<String>()
                        .trim()
                        .to_string()
                },
                location: {
                    let selector = Selector::parse(".last_b:nth-child(4)").unwrap();
                    parent.select(&selector)
                        .next()
                        .unwrap()
                        .text()
                        .collect::<String>()
                        .trim()
                        .to_string()
                },
                status: {
                    let selector = Selector::parse(".last_b:nth-child(1)").unwrap();
                    parent.select(&selector)
                        .next()
                        .unwrap()
                        .text()
                        .collect::<String>()
                        .trim()
                        .to_string()
                },
                message: {
                    let selector = Selector::parse(".last_b:nth-child(3)").unwrap();
                    parent.select(&selector)
                        .next()
                        .unwrap()
                        .text()
                        .collect::<String>()
                        .trim()
                        .to_string()
                },
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
