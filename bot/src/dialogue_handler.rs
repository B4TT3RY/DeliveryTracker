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
                    escape("๐ฌ ์กฐํํ  ์ด์ก์ฅ ๋ฒํธ๋ฅผ ์๋ ฅํด ์ฃผ์ธ์."),
                )
                .with_parse_mode(ParseMode::MarkdownV2),
                TypeKind::Track => SendMessage::new(
                    state.user_id,
                    escape("๐ฌ ์ถ์ ์ ์์ํ  ์ด์ก์ฅ ๋ฒํธ๋ฅผ ์๋ ฅํด ์ฃผ์ธ์."),
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
                            "โ ๏ธ ์ง์ํ๋ ํ๋ฐฐ์ฌ๊ฐ ์์ด์.\n\
                        ์ด์ก์ฅ ๋ฒํธ๋ฅผ ๋ค์ ํ์ธํ์๊ฑฐ๋ ๊ด๋ฆฌ์์๊ฒ ๋ฌธ์ํด ์ฃผ์ธ์.",
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
                        escape("๐ ์ด์ก์ฅ์ ์กฐํํ  ํ๋ฐฐ์ฌ๋ฅผ ์ ํํด ์ฃผ์ธ์."),
                    ),
                    TypeKind::Track => SendMessage::new(
                        state.user_id,
                        escape("๐ ์ด์ก์ฅ์ ์ถ์ ํ  ํ๋ฐฐ์ฌ๋ฅผ ์ ํํด ์ฃผ์ธ์."),
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
                    escape("โ ๏ธ ์๋ฒ์ ๋ฌธ์ ๊ฐ ์์ด์. ๋์ค์ ๋ค์ ์๋ํด ์ฃผ์ธ์."),
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
                (escape("โ ๏ธ ์ด์ก์ฅ ์ ๋ณด๊ฐ ์์ด์."), None)
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
