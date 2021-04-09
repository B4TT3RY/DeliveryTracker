use juniper::{GraphQLEnum, GraphQLObject};
use serde::{Deserialize, Serialize};

use crate::couriers::courier::CourierType;

#[derive(Debug, Serialize, Deserialize, GraphQLObject)]
pub struct DeliveryStatus {
    #[graphql(description = "택배사 ID")]
    pub id: String,
    #[graphql(description = "택배사 이름")]
    pub name: String,
    #[graphql(description = "운송장 번호")]
    pub tracking_number: String,
    #[graphql(description = "보내는 사람")]
    pub sender: Option<String>,
    #[graphql(description = "받는 사람")]
    pub receiver: Option<String>,
    #[graphql(description = "상품 이름")]
    pub product: Option<String>,
    #[graphql(description = "처리 단계")]
    pub tracks: Option<Vec<TrackingStatus>>,
}

#[derive(Debug, Serialize, Deserialize, GraphQLObject)]
pub struct TrackingStatus {
    #[graphql(description = "현재 단계")]
    pub state: StateType,
    #[graphql(description = "처리 시각")]
    pub time: String,
    #[graphql(description = "처리 위치")]
    pub location: String,
    #[graphql(description = "현재 단계")]
    pub status: String,
    #[graphql(description = "상태 메세지")]
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, GraphQLEnum)]
pub enum StateType {
    #[graphql(description = "택배 정보 접수")]
    InformationReceived,
    #[graphql(description = "택배 집하")]
    AtPickup,
    #[graphql(description = "택배 이동중")]
    InTransit,
    #[graphql(description = "배송 출발")]
    OutForDelivery,
    #[graphql(description = "배송 완료")]
    Delivered,
    #[graphql(description = "알 수 없음")]
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
                if status == "발송" || status == "도착" {
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
        }
    }
}
