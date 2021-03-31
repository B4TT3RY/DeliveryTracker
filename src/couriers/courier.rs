use anyhow::{anyhow, Result};
use async_trait::async_trait;

use crate::delivery_status::DeliveryStatus;

use super::{cjlogistics::CJLogistics, epost::EPost};

pub enum CourierType {
    CJLogistics(CJLogistics),
    EPost(EPost),
}

impl CourierType {
    pub fn get_courier_by_id(id: String) -> Result<CourierType> {
        match id.as_str() {
            "kr.cjlogistics" => Ok(CourierType::CJLogistics(CJLogistics)),
            "kr.epost" => Ok(CourierType::EPost(EPost)),
            _ => Err(anyhow!("해당 택배사가 존재하지 않습니다."))
        }
    }
    
    pub async fn track(id: String, tracking_number: String) -> Result<DeliveryStatus> {
        match CourierType::get_courier_by_id(id)? {
            CourierType::CJLogistics(courier) => courier.track(tracking_number).await,
            CourierType::EPost(courier) => courier.track(tracking_number).await,
        }
    }
}

#[async_trait]
pub trait Courier {
    fn get_url() -> &'static str;
    fn get_id() -> &'static str;
    fn get_name() -> &'static str;
    async fn track(&self, tracking_number: String) -> Result<DeliveryStatus>;
}
