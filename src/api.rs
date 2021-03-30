use anyhow::Result;
use tide::Request;

use crate::couriers::{cjlogistics::CJLogistics, courier::Courier, epost::EPost};

pub async fn start_api_server() -> Result<()> {
    let mut app = tide::new();
    app.at("/:courier/:tracking_number").get(|req: Request<()>| async move {
        let courier = req.param("courier").unwrap().parse().unwrap_or(String::new());
        let tracking_number = req.param("tracking_number").unwrap().parse().unwrap_or(String::new());

        let delivery_status = match courier.as_str() {
            "kr.cjlogistics" => CJLogistics::track(tracking_number),
            "kr.epost" => EPost::track(tracking_number),
            _ => CJLogistics::track(tracking_number),
        }
        .await
        .unwrap();
        
        Ok(serde_json::to_string(&delivery_status).unwrap())
    });

    app.listen("0.0.0.0:8083").await?;
    Ok(())
}