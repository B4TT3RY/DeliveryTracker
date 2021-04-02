use juniper::GraphQLObject;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, GraphQLObject)]
pub struct DeliveryStatus {
    pub id: String,
    pub name: String,
    pub tracking_number: String,
    pub sender: Option<String>,
    pub receiver: Option<String>,
    pub product: Option<String>,
    pub tracks: Option<Vec<TrackingStatus>>,
}

#[derive(Debug, Serialize, Deserialize, GraphQLObject)]
pub struct TrackingStatus {
    pub time: String,
    pub location: String,
    pub status: String,
    pub message: Option<String>,
}
