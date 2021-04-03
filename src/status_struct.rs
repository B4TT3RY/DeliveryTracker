use juniper::{GraphQLObject, GraphQLEnum};
use serde::{Deserialize, Serialize};

use crate::couriers::courier::CourierType;

#[derive(Debug, Serialize, Deserialize, GraphQLObject)]
pub struct DeliveryStatus {
    pub id: String,
    pub name: String,
    pub tracking_number: String,
    pub sender: Option<String>,
    pub receiver: Option<String>,
    pub product: Option<String>,
    pub tracks: Option<Vec<TrackingStatus>>,
}

#[derive(Debug, Serialize, Deserialize, GraphQLObject)]
pub struct TrackingStatus {
    pub state: StateType,
    pub time: String,
    pub location: String,
    pub status: String,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, GraphQLEnum)]
pub enum StateType {
    Shipped,
    InTransit,
    OutForDelivery,
    Delivered,
    Unknown,
}

impl StateType {
    pub fn to_type(courier_type: CourierType, status: &str) -> Self {
        match courier_type {
            CourierType::CJLogistics(_) => {
                match status {
                    "상품인수" => Self::Shipped,
                    "상품이동중" | "배달지도착" => Self::InTransit,
                    "배달출발" => Self::OutForDelivery,
                    "배달완료" => Self::Delivered,
                    _ => Self::Unknown,
                }
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
            },
            CourierType::ILogen(_) => {
                match status {
                    "집하완료" => Self::Shipped,
                    "터미널입고" | "터미널출고" | "배송입고" => Self::InTransit,
                    "배송출고" => Self::OutForDelivery,
                    "배송완료" => Self::Delivered,
                    _ => Self::Unknown
                }
            },
            CourierType::Lotte(_) => {
                match status {
                    "상품접수" => Self::Shipped,
                    "상품 이동중" => Self::InTransit,
                    "배송 출발" => Self::OutForDelivery,
                    "배달 완료" => Self::Delivered,
                    _ => Self::Unknown
                }
            },
            CourierType::Hanjin(_) => {
                match status {
                    "접수" => Self::Shipped,
                    "이동중" | "도착" | "배송준비중" => Self::InTransit,
                    "배송출발" => Self::OutForDelivery,
                    "배송완료" => Self::Delivered,
                    _ => Self::Unknown
                }
            },
            CourierType::GSPostbox(_) => {
                if status == "점포접수" {
                    Self::Shipped
                } else if status.contains("배송기사") || status.contains("입고") || status.contains("출고") {
                    Self::InTransit
                } else if status == "점포도착" {
                    Self::OutForDelivery
                } else if status == "고객전달" {
                    Self::Delivered
                } else {
                    Self::Unknown
                }
            },
            CourierType::CUPost(_) => {
                if status.contains("점포접수") {
                    Self::Shipped
                } else if status.contains("입고") || status.contains("출고") {
                    Self::InTransit
                } else if status.contains("도착") {
                    Self::OutForDelivery
                } else if status.contains("수령") {
                    Self::Delivered
                } else {
                    Self::Unknown
                }
            },
        }
    }
}