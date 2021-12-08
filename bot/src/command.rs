use std::error::Error;

use teloxide::{utils::command::BotCommand, prelude::*, types::Message};


#[derive(BotCommand)]
#[command(rename = "lowercase")]
pub enum Command {
    #[command(description = "off")]
    Start,
    #[command(description = "도움말을 볼 수 있어요.")]
    Help,
    #[command(description = "운송장 번호로 택배를 조회할 수 있어요.")]
    Search(String),
    #[command(description = "운송장 번호로 택배를 추적할 수 있어요.")]
    Track(String),
    #[command(description = "현재 추적중인 운송장을 관리할 수 있어요.")]
    List,
}

impl Command {
    pub async fn handler(
        cx: UpdateWithCx<crate::BotType, Message>,
        command: Command,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        match command {
            Command::Start | Command::Help => {
                cx.answer(Command::descriptions()).await?
            },
            Command::Search(tracking_number) => {
                cx.answer(format!("{:?}", tracking_number)).await?
            },
            Command::Track(tracking_number) => {
                cx.answer(format!("{:?}", tracking_number)).await?
            },
            Command::List => {
                cx.answer("").await?
            },
        };
        Ok(())
    }
}