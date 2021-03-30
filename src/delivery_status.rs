use crate::tracking_status::TrackingStatus;

#[derive(Debug)]
pub struct DeliveryStatus {
    pub tracking_number: String,
    pub sender: String,
    pub receiver: String,
    pub product: String,
    pub last_track: TrackingStatus,
}
