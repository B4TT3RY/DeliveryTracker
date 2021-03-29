use crate::tracking_status::TrackingStatus;

pub struct DeliveryStatus {
    pub tracking_number: String,
    pub sender: String,
    pub receiver: String,
    pub product: String,
    pub tracks: Vec<TrackingStatus>,
}
