use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    Error, Object, Result, SimpleObject,
};
use strum::IntoEnumIterator;
use surf::{http::mime, Body, StatusCode};
use tide::{Request, Response};

use crate::couriers::{Courier, CourierKind, DeliveryStatus};

#[derive(SimpleObject)]
struct CourierInfo {
    id: &'static str,
    name: &'static str,
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn track(
        &self,
        #[graphql(desc = "택배사 ID")] id: String,
        #[graphql(desc = "추적할 운송장 번호")] tracking_number: String,
    ) -> Result<DeliveryStatus> {
        Courier::new(id, Some(tracking_number))?
            .track()
            .await
            .map_err(|err| Error::from(err))
    }

    async fn couriers(&self) -> Vec<CourierInfo> {
        let mut result = Vec::new();
        for kind in CourierKind::iter() {
            let courier = Courier::new_with_kind(kind, None);
            result.push(CourierInfo {
                id: courier.get_id(),
                name: courier.get_name(),
            });
        }

        result
    }

    async fn available_couriers(
        &self,
        #[graphql(desc = "운송장 번호")] tracking_number: String,
    ) -> Vec<CourierInfo> {
        let mut result = Vec::new();
        for kind in CourierKind::iter() {
            let courier = Courier::new_with_kind(kind, Some(tracking_number.to_string()));
            if courier.validate().is_err() {
                continue;
            }
            result.push(CourierInfo {
                id: courier.get_id(),
                name: courier.get_name(),
            });
        }

        result
    }
}

pub async fn handle_playground(_: Request<()>) -> tide::Result<impl Into<Response>> {
    Ok(Response::builder(StatusCode::Ok)
        .body(Body::from_string(playground_source(
            GraphQLPlaygroundConfig::new("/graphql"),
        )))
        .content_type(mime::HTML)
        .build())
}
