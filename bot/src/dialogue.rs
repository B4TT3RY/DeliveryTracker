pub enum DialogueStage {
    Start(StartState),
    ReceivedTrackingNumber(ReceivedTrackingNumberState),
    SelectedCourier(SelectedCourierState),
}

pub struct StartState;
pub struct ReceivedTrackingNumberState {
    tracking_number: String,
}
pub struct SelectedCourierState {
    tracking_number: String,
    courier_id: String,
}