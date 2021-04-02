use anyhow::Result;
use serde_json::json;
use tide::{Request, Response};

use crate::{
    couriers::courier::CourierType,
    graphql::{handle_graphql, handle_playground},
};

pub async fn start_api_server() -> Result<()> {
    let mut app = tide::new();
    app.at("/:courier/:tracking_number")
        .get(|req: Request<()>| async move {
            let mut response = Response::new(200);
            response.insert_header("Content-Type", "application/json; charset=utf-8");

            let id = req
                .param("courier")
                .map(|s| s.to_string())
                .unwrap_or(String::new());
            let tracking_number = req
                .param("tracking_number")
                .map(|s| s.to_string())
                .unwrap_or(String::new());
            let delivery_status = CourierType::track(id, tracking_number).await;
            if let Err(err) = delivery_status {
                response.set_status(500);
                response.set_body(json!({
                    "message": err.to_string()
                }));
                return Ok(response);
            }

            let body = serde_json::to_string_pretty(&delivery_status?);
            if let Err(err) = body {
                response.set_status(500);
                response.set_body(json!({
                    "message": err.to_string()
                }));
                return Ok(response);
            }

            response.set_body(body?);
            Ok(response)
        });

    app.at("/graphql").post(handle_graphql);
    app.at("/playground").get(handle_playground);

    app.listen("0.0.0.0:8083").await?;
    Ok(())
}
