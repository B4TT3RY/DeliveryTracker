use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackingStatus {
    pub time: String,
    pub location: String,
    pub status: String,
    pub message: Option<String>,
}
