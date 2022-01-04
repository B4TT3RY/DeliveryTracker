use std::env;

use bot::tracker::{tracker_client::TrackerClient, SearchRequest, SupportCouriersRequest};
use telbot_hyper::{
    types::{
        markup::ParseMode,
        message::{EditMessageText, SendMessage},
    },
    Api,
};

use crate::{
    dialogue::{
        Dialogue, DialogueAnswerKind, DialogueStage, ReceivedTrackingNumberState,
        SelectedCourierState, TypeKind,
    },
    telegram::{
        create_courier_keyboard, create_search_result_keyboard, create_simple_tracking_message,
        escape,
    },
};

struct S(DialogueStage, DialogueAnswerKind);

pub async fn handle_dialogue(api: &Api, stage: DialogueStage, answer: DialogueAnswerKind) {
    use DialogueAnswerKind::*;
    use DialogueStage::*;

    match S(stage, answer) {
        S(Start(state), Message(_)) => {
            let send_message = match state.kind {
                TypeKind::Search => SendMessage::new(
                    state.user_id,
                    escape("💬 조회할 운송장 번호를 입력해 주세요."),
                )
                .with_parse_mode(ParseMode::MarkdownV2),
                TypeKind::Track => SendMessage::new(
                    state.user_id,
                    escape("💬 추적을 시작할 운송장 번호를 입력해 주세요."),
                )
                .with_parse_mode(ParseMode::MarkdownV2),
            };

            api.send_json(&send_message).await.unwrap();

            Dialogue::next(
                state.user_id,
                DialogueStage::ReceivedTrackingNumber(ReceivedTrackingNumberState {
                    kind: state.kind,
                    user_id: state.user_id,
                    tracking_number: None,
                }),
            );
        }
        S(ReceivedTrackingNumber(state), Message(message)) => {
            let tracking_number = if let Some(tracking_number) = state.tracking_number {
                tracking_number
            } else {
                message
            };

            let mut client =
                TrackerClient::connect(env::var("GRPC_ADDR").expect("env GRPC_ADDR is not set."))
                    .await
                    .unwrap();
            let request = tonic::Request::new(SupportCouriersRequest {
                tracking_number: tracking_number.clone(),
            });

            if let Ok(response) = client.get_support_couriers(request).await {
                let response = response.into_inner();
                if response.couriers.is_empty() {
                    let send_message = SendMessage::new(
                        state.user_id,
                        escape(
                            "⚠️ 지원하는 택배사가 없어요.\n\
                        운송장 번호를 다시 확인하시거나 관리자에게 문의해 주세요.",
                        ),
                    )
                    .with_parse_mode(ParseMode::MarkdownV2);

                    api.send_json(&send_message).await.unwrap();

                    Dialogue::exit(state.user_id);
                    return;
                }

                let send_message = match state.kind {
                    TypeKind::Search => SendMessage::new(
                        state.user_id,
                        escape("🚚 운송장을 조회할 택배사를 선택해 주세요."),
                    ),
                    TypeKind::Track => SendMessage::new(
                        state.user_id,
                        escape("🚚 운송장을 추적할 택배사를 선택해 주세요."),
                    ),
                }
                .with_parse_mode(ParseMode::MarkdownV2)
                .with_reply_markup(create_courier_keyboard(response));

                let send_message = api.send_json(&send_message).await.unwrap();

                Dialogue::next(
                    state.user_id,
                    DialogueStage::SelectedCourier(SelectedCourierState {
                        kind: state.kind,
                        user_id: state.user_id,
                        tracking_number,
                        message_id: send_message.message_id,
                    }),
                );
            } else {
                let send_message = SendMessage::new(
                    state.user_id,
                    escape("⚠️ 서버에 문제가 있어요. 나중에 다시 시도해 주세요."),
                )
                .with_parse_mode(ParseMode::MarkdownV2);

                api.send_json(&send_message).await.unwrap();

                Dialogue::exit(state.user_id);
            }
        }
        S(SelectedCourier(state), CallbackQuery(query)) => {
            let mut client =
                TrackerClient::connect(env::var("GRPC_ADDR").expect("env GRPC_ADDR is not set."))
                    .await
                    .unwrap();
            let request = tonic::Request::new(SearchRequest {
                tracking_number: state.tracking_number,
                courier_id: query,
            });

            let (text, keyboard) = if let Ok(response) = client.search(request).await {
                let response = response.into_inner();
                (
                    create_simple_tracking_message(&response),
                    if let Some(info) = response.tracking_info {
                        Some(create_search_result_keyboard(info.url, !info.is_delivered, info.tracking_number))
                    } else {
                        None
                    },
                )
            } else {
                (escape("⚠️ 운송장 정보가 없어요."), None)
            };

            let mut edit_message_text = EditMessageText::new(state.user_id, state.message_id, text)
                .with_parse_mode(ParseMode::MarkdownV2);
            edit_message_text.reply_markup = keyboard;

            api.send_json(&edit_message_text).await.unwrap();

            Dialogue::exit(state.user_id);
        }
        _ => {}
    };
}
