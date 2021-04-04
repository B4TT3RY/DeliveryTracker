use anyhow::Result;

use crate::graphql::{handle_graphql, handle_playground};

pub async fn start_api_server() -> Result<()> {
    let mut app = tide::new();

    app.at("/graphql").post(handle_graphql);
    app.at("/playground").get(handle_playground);

    app.listen("0.0.0.0:8083").await?;
    Ok(())
}
