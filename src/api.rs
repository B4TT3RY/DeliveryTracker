use anyhow::Result;
use tide::Request;

use crate::couriers::courier::CourierType;

pub async fn start_api_server() -> Result<()> {
    let mut app = tide::new();
    app.at("/:courier/:tracking_number").get(|req: Request<()>| async move {
        let id = req.param("courier").map(|s| s.to_string()).unwrap_or(String::new());
        let tracking_number = req.param("tracking_number").map(|s| s.to_string()).unwrap_or(String::new());
        let delivery_status = CourierType::track(id, tracking_number).await?;
        
        serde_json::to_string(&delivery_status).map_err(|err| tide::Error::new(500, err))
    });

    app.listen("0.0.0.0:8083").await?;
    Ok(())
}