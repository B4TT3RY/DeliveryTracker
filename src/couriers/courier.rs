use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Courier {
    fn get_url() -> &'static str;
    async fn track() -> Result<()>;
}