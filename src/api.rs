use anyhow::Result;
use tide::Request;

use crate::couriers::{cjlogistics::CJLogistics, courier::Courier, epost::EPost};

pub async fn start_api_server() -> Result<()> {
    let mut app = tide::new();
    app.at("/:courier/:tracking_number").get(|req: Request<()>| async move {
        let tracking_number = req.param("tracking_number").map(|s| s.to_string()).unwrap_or(String::new());

        let delivery_status = match req.param("courier") {
            Ok("kr.cjlogistics") => CJLogistics::track(tracking_number),
            Ok("kr.epost") => EPost::track(tracking_number),
            _ => CJLogistics::track(tracking_number),
        }
        .await
        .unwrap();
        
        Ok(serde_json::to_string(&delivery_status).unwrap())
    });

    app.listen("0.0.0.0:8083").await?;
    Ok(())
}