use anyhow::{anyhow, Result};
use async_trait::async_trait;

use crate::delivery_status::DeliveryStatus;

use super::{cjlogistics::CJLogistics, epost::EPost, ilogen::ILogen, lotte::Lotte};

#[async_trait]
pub trait Courier {
    fn get_url() -> &'static str;
    fn get_id() -> &'static str;
    fn get_name() -> &'static str;
    async fn validate(&self) -> Result<&Self>;
    async fn track(&self) -> Result<DeliveryStatus>;
}

pub enum CourierType {
    CJLogistics(CJLogistics),
    EPost(EPost),
    ILogen(ILogen),
    Lotte(Lotte),
}

impl CourierType {
    pub fn get_courier(id: String, tracking_number: String) -> Result<CourierType> {
        match id.as_str() {
            "kr.cjlogistics" => Ok(CourierType::CJLogistics(CJLogistics { tracking_number })),
            "kr.epost" => Ok(CourierType::EPost(EPost { tracking_number })),
            "kr.ilogen" => Ok(CourierType::ILogen(ILogen { tracking_number })),
            "kr.lotte" => Ok(CourierType::Lotte(Lotte { tracking_number })),
            _ => Err(anyhow!("해당 택배사가 존재하지 않습니다."))
        }
    }
    
    pub async fn track(id: String, tracking_number: String) -> Result<DeliveryStatus> {
        match CourierType::get_courier(id, tracking_number)? {
            CourierType::CJLogistics(courier) => courier.validate().await?.track().await,
            CourierType::EPost(courier) => courier.validate().await?.track().await,
            CourierType::ILogen(courier) => courier.validate().await?.track().await,
            CourierType::Lotte(courier) => courier.validate().await?.track().await,
        }
    }
}
