use surf::{Body, StatusCode, http::mime};
use tide::{Request, Response};
use juniper::{EmptyMutation, EmptySubscription, RootNode, graphql_object, http::GraphQLRequest};
use lazy_static::lazy_static;

use crate::{couriers::courier::CourierType, delivery_status::DeliveryStatus};

struct Delivery;

impl Delivery {
    
}

struct QueryRoot;

#[graphql_object]
impl QueryRoot {
    async fn track(id: String, tracking_number: String) -> Option<DeliveryStatus> {
        let result = CourierType::track(id, tracking_number).await;
        if let Err(_) = result {
            return None;
        }
        return result.ok();
    }
}

type Schema = RootNode<'static, QueryRoot, EmptyMutation, EmptySubscription>;
lazy_static! {
    static ref SCHEMA: Schema = Schema::new(QueryRoot {}, EmptyMutation::new(), EmptySubscription::new());
}

pub async fn handle_graphql(mut request: Request<()>) -> tide::Result {
    let query: GraphQLRequest = request.body_json().await?;
    let response = query.execute(&SCHEMA, request.state()).await;
    let status = if response.is_ok() {
        StatusCode::Ok
    } else {
        StatusCode::BadRequest
    };

    Ok(Response::builder(status)
        .body(Body::from_json(&response)?)
        .build())
}

pub async fn handle_playground(_: Request<()>) -> tide::Result<impl Into<Response>> {
    Ok(
        Response::builder(200)
            .body(juniper::http::playground::playground_source("/graphql", None))
            .content_type(mime::HTML)
    )
}