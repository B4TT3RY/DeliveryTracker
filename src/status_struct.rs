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
    pub fn to_type(courier_type: CourierType, status: String) -> Self {
        match courier_type {
            CourierType::CJLogistics(_) => {
                match status.as_str() {
                    "상품인수" => Self::Shipped,
                    "상품이동중" | "배달지도착" => Self::InTransit,
                    "배달출발" => Self::OutForDelivery,
                    "배달완료" => Self::Delivered,
                    _ => Self::Unknown,
                }
            },
            CourierType::EPost(_) => {
                Self::Shipped
            },
            CourierType::ILogen(_) => {
                Self::Shipped
            },
            CourierType::Lotte(_) => {
                Self::Shipped
            },
            CourierType::Hanjin(_) => {
                Self::Shipped
            },
            CourierType::GSPostbox(_) => {
                Self::Shipped
            },
            CourierType::CUPost(_) => {
                Self::Shipped
            },
        }
    }
}