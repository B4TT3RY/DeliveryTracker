use couriers::{kr::*, us::*};
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
            // "cn.cainiao" => Cainiao::track(tracking_number).await,
            "kr.chunil" => Chunil::track(tracking_number).await,
            "kr.cjlogistics" => Cjlogistics::track(tracking_number).await,
            "kr.cupost" => Cupost::track(tracking_number).await,
            "kr.daesin" => Daesin::track(tracking_number).await,
            "kr.epost" => Epost::track(tracking_number).await,
            "kr.epostems" => EpostEMS::track(tracking_number).await,
            "kr.gspostbox" => Gspostbox::track(tracking_number).await,
            "kr.hanjin" => Hanjin::track(tracking_number).await,
            "kr.ilyanglogis" => IlyangLogis::track(tracking_number).await,
            "kr.kyoungdong" => Kyoungdong::track(tracking_number).await,
            "kr.logen" => Logen::track(tracking_number).await,
            "kr.lotte" => Lotte::track(tracking_number).await,
            "us.warpex" => Warpex::track(tracking_number).await,
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

    async fn get_support_couriers(
        &self,
        request: tonic::Request<tracker::SupportCouriersRequest>,
    ) -> Result<tonic::Response<tracker::SupportCouriersResponse>, tonic::Status> {
        let mut couriers: Vec<tracker::SupportCouriersDetail> = Vec::new();
        let tracking_number = request.into_inner().tracking_number;

        // if Cainiao::validate(&tracking_number) {
        //     couriers.push(tracker::SupportCouriersDetail {
        //         id: Cainiao::id().to_string(),
        //         name: Cainiao::name().to_string(),
        //     });
        // }

        if Cjlogistics::validate(&tracking_number) {
            couriers.push(tracker::SupportCouriersDetail {
                id: Cjlogistics::id().to_string(),
                name: Cjlogistics::name().to_string(),
            });
        }
        if Epost::validate(&tracking_number) {
            couriers.push(tracker::SupportCouriersDetail {
                id: Epost::id().to_string(),
                name: Epost::name().to_string(),
            });
        }
        if EpostEMS::validate(&tracking_number) {
            couriers.push(tracker::SupportCouriersDetail {
                id: EpostEMS::id().to_string(),
                name: EpostEMS::name().to_string(),
            });
        }
        if Hanjin::validate(&tracking_number) {
            couriers.push(tracker::SupportCouriersDetail {
                id: Hanjin::id().to_string(),
                name: Hanjin::name().to_string(),
            });
        }
        if Logen::validate(&tracking_number) {
            couriers.push(tracker::SupportCouriersDetail {
                id: Logen::id().to_string(),
                name: Logen::name().to_string(),
            });
        }
        if Lotte::validate(&tracking_number) {
            couriers.push(tracker::SupportCouriersDetail {
                id: Lotte::id().to_string(),
                name: Lotte::name().to_string(),
            });
        }
        if Cupost::validate(&tracking_number) {
            couriers.push(tracker::SupportCouriersDetail {
                id: Cupost::id().to_string(),
                name: Cupost::name().to_string(),
            });
        }
        if Gspostbox::validate(&tracking_number) {
            couriers.push(tracker::SupportCouriersDetail {
                id: Gspostbox::id().to_string(),
                name: Gspostbox::name().to_string(),
            });
        }
        

        if Chunil::validate(&tracking_number) {
            couriers.push(tracker::SupportCouriersDetail {
                id: Chunil::id().to_string(),
                name: Chunil::name().to_string(),
            });
        }
        if Daesin::validate(&tracking_number) {
            couriers.push(tracker::SupportCouriersDetail {
                id: Daesin::id().to_string(),
                name: Daesin::name().to_string(),
            });
        }
        if IlyangLogis::validate(&tracking_number) {
            couriers.push(tracker::SupportCouriersDetail {
                id: IlyangLogis::id().to_string(),
                name: IlyangLogis::name().to_string(),
            });
        }
        if Kyoungdong::validate(&tracking_number) {
            couriers.push(tracker::SupportCouriersDetail {
                id: Kyoungdong::id().to_string(),
                name: Kyoungdong::name().to_string(),
            });
        }

        if Warpex::validate(&tracking_number) {
            couriers.push(tracker::SupportCouriersDetail {
                id: Warpex::id().to_string(),
                name: Warpex::name().to_string(),
            });
        }

        Ok(Response::new(tracker::SupportCouriersResponse { couriers }))
    }
}
