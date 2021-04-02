use serde::{Deserialize, Serialize};
use juniper::GraphQLObject;

use crate::tracking_status::TrackingStatus;

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
