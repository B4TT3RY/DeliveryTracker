use telbot_reqwest::Api;
use telbot_types::{message::{SendMessage, Message}, markup::ParseMode};

use crate::{command::Command, telegram};

pub async fn handle_command(api: &Api, message: &Message, text: &str) {
    let command = Command::new(text);
    let mut args = command.args();
    match command.label {
        "/start" | "/help" => {
            let help_message = telegram::escape(
                "/help - 도움말을 볼 수 있어요.\n\
                /search - 운송장 번호로 택배를 조회할 수 있어요.\n\
                /track - 운송장 번호로 택배를 추적할 수 있어요.\n\
                /list - 현재 추적중인 운송장을 관리할 수 있어요."
            );
            api.send_json(
                &SendMessage::new(message.chat.id, help_message)
                    .with_parse_mode(ParseMode::MarkdownV2)
                )
                .await
                .expect("Failed to help message");
        }
        "/search" => {
            if let Some(tracking_number) = args.next() {
                api.send_json(&message.reply_text(format!("운송장번호: {}", tracking_number))).await.unwrap();
            } else {
                api.send_json(&message.reply_text("운송장번호 미입력")).await.unwrap();
            }
        }
        _ => {}
    };
}