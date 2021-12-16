use telbot_hyper::{Api, types::{message::SendMessage, markup::ParseMode}};

use crate::{dialogue::{DialogueStage, Dialogue, ReceivedTrackingNumberState, SelectedCourierState}, telegram::escape};

pub async fn handle_dialogue(api: &Api, stage: DialogueStage, answer: &str) {
    match stage {
        DialogueStage::Start(state) => {
            let send_message = SendMessage::new(state.user_id, escape("💬 조회할 운송장 번호를 입력해주세요."))
                .with_parse_mode(ParseMode::MarkdownV2);
            api.send_json(&send_message).await.unwrap();

            Dialogue::next(state.user_id, DialogueStage::ReceivedTrackingNumber(ReceivedTrackingNumberState {
                user_id: state.user_id,
            }));
        },
        DialogueStage::ReceivedTrackingNumber(state) => {
            let send_message = SendMessage::new(state.user_id, escape("🚚 운송장 번호를 조회할 택배사를 선택해주세요."))
                .with_parse_mode(ParseMode::MarkdownV2);
            api.send_json(&send_message).await.unwrap();

            Dialogue::next(state.user_id, DialogueStage::SelectedCourier(SelectedCourierState {
                user_id: state.user_id,
                tracking_number: answer.to_string(),
            }));
        },
        DialogueStage::SelectedCourier(state) => {
            let send_message = SendMessage::new(state.user_id, escape(format!("송장: {}, 택배사: {}", state.tracking_number, answer)))
                .with_parse_mode(ParseMode::MarkdownV2);
            api.send_json(&send_message).await.unwrap();
            Dialogue::exit(state.user_id);
        },
    };
}