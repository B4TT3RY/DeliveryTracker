use anyhow::Result;
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use tide::Redirect;

use crate::graphql::{QueryRoot, handle_playground};

pub async fn start_api_server() -> Result<()> {
    let mut app = tide::new();
    app.with(tide_compress::CompressMiddleware::new());

    let schema: Schema<QueryRoot, EmptyMutation, EmptySubscription>
        = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();
    app.at("/graphql").post(async_graphql_tide::endpoint(schema));

    app.at("/").get(Redirect::new("/playground"));
    app.at("/playground").get(handle_playground);

    app.listen("0.0.0.0:8083").await?;
    Ok(())
}
