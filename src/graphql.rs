use async_graphql::{Error, Object, Result, http::{playground_source, GraphQLPlaygroundConfig}};
use surf::{http::mime, Body, StatusCode};
use tide::{Request, Response};

use crate::{couriers::courier::CourierType, status_struct::DeliveryStatus};

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn track(
        &self,
        #[graphql(desc = "택배사 ID")] id: String,
        #[graphql(desc = "추적할 운송장 번호")] tracking_number: String
    ) -> Result<DeliveryStatus> {
        CourierType::track(id, tracking_number)
            .await
            .map_err(|err| Error::from(err))
    }
}

pub async fn handle_playground(_: Request<()>) -> tide::Result<impl Into<Response>> {
    Ok(
        Response::builder(StatusCode::Ok)
            .body(Body::from_string(playground_source(
                GraphQLPlaygroundConfig::new("/graphql"),
            )))
            .content_type(mime::HTML)
            .build()
    )
}
