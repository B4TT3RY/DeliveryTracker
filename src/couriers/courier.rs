use anyhow::Result;
use async_trait::async_trait;

use crate::delivery_status::DeliveryStatus;

#[async_trait]
pub trait Courier {
    fn get_url() -> &'static str;
    async fn track(self, tracking_number: String) -> Result<DeliveryStatus>;
}
