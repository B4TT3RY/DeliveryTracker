use telbot_hyper::{Api, types::{message::{SendMessage, Message}, markup::ParseMode}};

use crate::{command::Command, telegram::{self, escape}, dialogue::{Dialogue, DialogueStage, StartState, SelectedCourierState}, dialogue_handler};

pub async fn handle_command(api: &Api, message: &Message, text: &str) {
    let command = Command::new(text);
    let mut args = command.args();
    match command.label {
        "/start" | "/help" => {
            let help_message = telegram::escape(
                "/help - 도움말을 볼 수 있어요.\n\
                /search - 운송장 번호로 택배를 조회할 수 있어요.\n\
                /track - 운송장 번호로 택배를 추적할 수 있어요.\n\
                /list - 현재 추적중인 운송장을 관리할 수 있어요.\n\
                /cancel - 대화를 취소할 수 있어요."
            );
            api.send_json(
                &SendMessage::new(message.chat.id, help_message)
                    .with_parse_mode(ParseMode::MarkdownV2)
                )
                .await
                .expect("Failed to send help message");
        }
        "/search" => {
            let stage = if let Some(tracking_number) = args.next() {
                DialogueStage::SelectedCourier(SelectedCourierState {
                    user_id: message.chat.id,
                    tracking_number: tracking_number.to_string(),
                })
            } else {
                DialogueStage::Start(StartState {
                    user_id: message.chat.id,
                })
            };

            Dialogue::next(message.chat.id, stage.clone());
            dialogue_handler::handle_dialogue(api, stage, "").await;
        }
        "/cancel" => {
            if Dialogue::exit(message.chat.id) {
                api.send_json(
                    &SendMessage::new(message.chat.id, escape("❌ 취소되었어요."))
                        .with_parse_mode(ParseMode::MarkdownV2)
                    )
                    .await
                    .expect("Failed to send cancel message");
            }
        }
        _ => {}
    };
}