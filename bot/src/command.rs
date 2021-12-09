use std::error::Error;

use teloxide::{utils::command::BotCommand, prelude::*, types::Message};

use crate::{dialogue::{Dialogue, states::{NothingState, StartState, ReceiveTrackingNumberState}}, telegram};


#[derive(BotCommand, Debug)]
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
        cx: &UpdateWithCx<crate::BotType, Message>,
        command: Command,
    ) -> Result<Dialogue, Box<dyn Error + Send + Sync>> {
        match command {
            Command::Start | Command::Help => {
                cx.answer(telegram::escape(Command::descriptions())).await?;
                Ok(Dialogue::Nothing(NothingState))
            },
            Command::Search(tracking_number) => {
                if tracking_number.is_empty() {
                    Ok(Dialogue::Start(StartState))
                } else {
                    Ok(Dialogue::ReceiveTrackingNumber(ReceiveTrackingNumberState { tracking_number }))
                }
            },
            Command::Track(tracking_number) => {
                if tracking_number.is_empty() {
                    Ok(Dialogue::Start(StartState))
                } else {
                    Ok(Dialogue::ReceiveTrackingNumber(ReceiveTrackingNumberState { tracking_number }))
                }
            },
            Command::List => {
                cx.answer("").await?;
                Ok(Dialogue::Nothing(NothingState))
            },
        }
    }
}