use couriers::kr::{cjlogistics::Cjlogistics, epost::Epost};
use structs::Courier;
use tonic::{Response, Status};

use crate::tracker::tracker_server::Tracker;

mod couriers;
mod structs;

pub mod tracker {
    tonic::include_proto!("tracker");
}

#[derive(Default)]
pub struct DeliveryTracker {}

#[tonic::async_trait]
impl Tracker for DeliveryTracker {
    async fn track(
        &self,
        request: tonic::Request<tracker::TrackingRequest>,
    ) -> Result<tonic::Response<tracker::TrackingResponse>, tonic::Status> {
        let tracking_request = request.into_inner();
        let courier_id = tracking_request.courier_id.as_str();
        let tracking_number = tracking_request.tracking_number.as_str();
        let result = match courier_id {
            "kr.cjlogistics" => Cjlogistics::track(tracking_number).await,
            "kr.epost" => Epost::track(tracking_number).await,
            _ => {
                return Err(Status::invalid_argument("Not supported courier"));
            }
        };
        match result {
            Ok(info) => Ok(Response::new(tracker::TrackingResponse {
                status: 0,
                message: None,
                tracking_info: Some(info),
            })),
            Err(err) => {
                use structs::TrackingError::*;
                let (status, message) = match err {
                    RequestFailed(err) => (1, err),
                    WrongTrackingNumber(err) => (2, err),
                    NotExistsTrackingNumber => (3, String::new()),
                    ParsingError(err) => (4, err),
                };
                Ok(Response::new(tracker::TrackingResponse {
                    status,
                    message: Some(message.to_string()),
                    tracking_info: None,
                }))
            }
        }
    }
}
