use anyhow::Result;
use async_trait::async_trait;

use crate::delivery_status::DeliveryStatus;

#[async_trait]
pub trait Courier {
    fn get_url() -> &'static str;
    fn get_name() -> &'static str;
    async fn track(tracking_number: String) -> Result<DeliveryStatus>;
}
