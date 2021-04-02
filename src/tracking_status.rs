use serde::{Deserialize, Serialize};
use juniper::GraphQLObject;

#[derive(Debug, Serialize, Deserialize, GraphQLObject)]
pub struct TrackingStatus {
    pub time: String,
    pub location: String,
    pub status: String,
    pub message: Option<String>,
}
