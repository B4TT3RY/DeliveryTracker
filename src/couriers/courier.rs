use anyhow::{anyhow, Result};
use async_trait::async_trait;

use crate::status_struct::DeliveryStatus;

use super::{
    cainiao::Cainiao, cjlogistics::CJLogistics, cupost::CUPost, epost::EPost, gspostbox::GSPostbox,
    hanjin::Hanjin, ilogen::ILogen, lotte::Lotte,
};

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
    Hanjin(Hanjin),
    GSPostbox(GSPostbox),
    CUPost(CUPost),
    Cainiao(Cainiao),
}

impl CourierType {
    pub fn get_courier(id: String, tracking_number: Option<String>) -> Result<CourierType> {
        let tracking_number = tracking_number.unwrap_or(String::new());
        match id.as_str() {
            "kr.cjlogistics" => Ok(CourierType::CJLogistics(CJLogistics { tracking_number })),
            "kr.epost" => Ok(CourierType::EPost(EPost { tracking_number })),
            "kr.ilogen" => Ok(CourierType::ILogen(ILogen { tracking_number })),
            "kr.lotte" => Ok(CourierType::Lotte(Lotte { tracking_number })),
            "kr.hanjin" => Ok(CourierType::Hanjin(Hanjin { tracking_number })),
            "kr.gspostbox" => Ok(CourierType::GSPostbox(GSPostbox { tracking_number })),
            "kr.cupost" => Ok(CourierType::CUPost(CUPost { tracking_number })),
            "cn.cainiao" => Ok(CourierType::Cainiao(Cainiao { tracking_number })),
            _ => Err(anyhow!("해당 택배사가 존재하지 않습니다.")),
        }
    }

    pub async fn track(id: String, tracking_number: String) -> Result<DeliveryStatus> {
        match CourierType::get_courier(id, Some(tracking_number))? {
            CourierType::CJLogistics(courier) => courier.validate().await?.track().await,
            CourierType::EPost(courier) => courier.validate().await?.track().await,
            CourierType::ILogen(courier) => courier.validate().await?.track().await,
            CourierType::Lotte(courier) => courier.validate().await?.track().await,
            CourierType::Hanjin(courier) => courier.validate().await?.track().await,
            CourierType::GSPostbox(courier) => courier.validate().await?.track().await,
            CourierType::CUPost(courier) => courier.validate().await?.track().await,
            CourierType::Cainiao(courier) => courier.validate().await?.track().await,
        }
    }
}
