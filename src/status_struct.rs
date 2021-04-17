use async_graphql::{Enum, SimpleObject};

use crate::couriers::courier::CourierType;

#[derive(SimpleObject)]
pub struct DeliveryStatus {
    /// 택배사 ID
    pub id: String,
    /// 택배사 이름
    pub name: String,
    /// 운송장 번호
    pub tracking_number: String,
    /// 보내는 사람
    pub sender: Option<String>,
    /// 받는 사람
    pub receiver: Option<String>,
    /// 상품 이름
    pub product: Option<String>,
    /// 처리 단계
    pub tracks: Option<Vec<TrackingStatus>>,
}

#[derive(SimpleObject)]
pub struct TrackingStatus {
    /// 현재 단계
    pub state: StateType,
    /// 처리 시각
    pub time: String,
    /// 처리 위치
    pub location: Option<String>,
    /// 현재 단계
    pub status: String,
    /// 상태 메세지
    pub message: Option<String>,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum StateType {
    /// 택배 정보 접수
    InformationReceived,

    /// 택배 집하
    AtPickup,

    /// 택배 이동중
    InTransit,

    /// 배송 출발
    OutForDelivery,

    /// 배송 완료
    Delivered,

    /// 알 수 없음
    Unknown,
}

impl StateType {
    pub fn to_type(courier_type: CourierType, status: &str) -> Self {
        match courier_type {
            CourierType::CJLogistics(_) => match status {
                "상품인수" => Self::AtPickup,
                "상품이동중" | "배달지도착" => Self::InTransit,
                "배달출발" => Self::OutForDelivery,
                "배달완료" => Self::Delivered,
                _ => Self::Unknown,
            },
            CourierType::EPost(_) => {
                if status == "접수" {
                    Self::InformationReceived
                } else if status == "발송" || status == "도착" {
                    Self::InTransit
                } else if status.contains("배달준비") {
                    Self::OutForDelivery
                } else if status.contains("배달완료") {
                    Self::Delivered
                } else {
                    Self::Unknown
                }
            }
            CourierType::ILogen(_) => match status {
                "집하완료" => Self::AtPickup,
                "터미널입고" | "터미널출고" | "배송입고" => Self::InTransit,
                "배송출고" => Self::OutForDelivery,
                "배송완료" => Self::Delivered,
                _ => Self::Unknown,
            },
            CourierType::Lotte(_) => match status {
                "상품접수" => Self::AtPickup,
                "상품 이동중" => Self::InTransit,
                "배송 출발" => Self::OutForDelivery,
                "배달 완료" => Self::Delivered,
                _ => Self::Unknown,
            },
            CourierType::Hanjin(_) => match status {
                "접수" => Self::InformationReceived,
                "입고" => Self::AtPickup,
                "이동중" | "도착" | "배송준비중" => Self::InTransit,
                "배송출발" => Self::OutForDelivery,
                "배송완료" => Self::Delivered,
                _ => Self::Unknown,
            },
            CourierType::GSPostbox(_) => {
                if status == "점포접수" {
                    Self::InformationReceived
                } else if status.contains("인수") {
                    Self::AtPickup
                } else if status.contains("입고")
                    || status.contains("출고")
                    || status.contains("인계")
                {
                    Self::InTransit
                } else if status == "점포도착" {
                    Self::OutForDelivery
                } else if status == "고객전달" {
                    Self::Delivered
                } else {
                    Self::Unknown
                }
            }
            CourierType::CUPost(_) => {
                if status.contains("점포접수") {
                    Self::AtPickup
                } else if status.contains("입고") || status.contains("출고") {
                    Self::InTransit
                } else if status.contains("도착") {
                    Self::OutForDelivery
                } else if status.contains("수령") {
                    Self::Delivered
                } else {
                    Self::Unknown
                }
            }
            CourierType::Cainiao(_) => {
                Self::Unknown
            }
        }
    }
}
