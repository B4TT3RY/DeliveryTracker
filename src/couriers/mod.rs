use anyhow::{anyhow, Result};

pub mod cainiao;
pub mod cjlogistics;
pub mod cupost;
pub mod epost;
pub mod gspostbox;
pub mod hanjin;
pub mod ilogen;
pub mod lotte;

pub struct Courier {
    pub tracking_number: String,
    pub kind: CourierKind,
}

macro_rules! define_couriers {
    ($($module:ident :: $name:ident),+) => {
        pub enum CourierKind {
            $($name),+
        }

        impl Courier {
            pub fn new(id: String, tracking_number: Option<String>) -> Result<Self> {
                let kind = match id.as_str() {
                    $($module::ID => Some(CourierKind::$name),)+
                    _ => None
                }.ok_or_else(|| anyhow!("해당 택배사가 존재하지 않습니다."))?;

                Ok(Self {
                    tracking_number: tracking_number.unwrap_or_default(),
                    kind,
                })
            }

            pub async fn track(&self) -> Result<DeliveryStatus> {
                match self.kind {
                    $(
                        CourierKind::$name => {
                            $module::validate(self)?;
                            $module::track(self).await
                        }
                    )+
                }
            }
        }
    }
}

define_couriers! {
    cjlogistics::CJLogistics,
    cupost::CUPost,
    epost::EPost,
    gspostbox::GSPostbox,
    hanjin::Hanjin
}

use async_graphql::{Enum, SimpleObject};

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
