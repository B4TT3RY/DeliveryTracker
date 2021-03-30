use serde::{Deserialize, Serialize};

use crate::tracking_status::TrackingStatus;

#[derive(Debug, Serialize, Deserialize)]
pub struct DeliveryStatus {
    pub id: String,
    pub name: String,
    pub tracking_number: String,
    pub sender: String,
    pub receiver: String,
    pub product: Option<String>,
    pub tracks: Vec<TrackingStatus>,
}
