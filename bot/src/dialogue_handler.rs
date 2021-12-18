use std::env;

use bot::tracker::{tracker_client::TrackerClient, SupportCouriersRequest, TrackingRequest};
use telbot_hyper::{
    types::{markup::ParseMode, message::{SendMessage, SendChatAction, ChatActionKind}},
    Api,
};

use crate::{
    dialogue::{Dialogue, DialogueStage, ReceivedTrackingNumberState, SelectedCourierState},
    telegram::{escape, create_simple_tracking_message},
};

pub async fn handle_dialogue(api: &Api, stage: DialogueStage, answer: &str) {
    match stage {
        DialogueStage::Start(state) => {
            let send_message = SendMessage::new(
                state.user_id,
                escape("💬 조회할 운송장 번호를 입력해주세요."),
            )
            .with_parse_mode(ParseMode::MarkdownV2);
            api.send_json(&send_message).await.unwrap();

            Dialogue::next(
                state.user_id,
                DialogueStage::ReceivedTrackingNumber(ReceivedTrackingNumberState {
                    user_id: state.user_id,
                    tracking_number: None,
                }),
            );
        }
        DialogueStage::ReceivedTrackingNumber(state) => {
            let tracking_number = if let Some(tracking_number) = state.tracking_number {
                tracking_number
            } else {
                answer.to_string()
            };

            match api.send_json(&SendChatAction::new(state.user_id, ChatActionKind::Typing)).await {
                Ok(_) => {},
                Err(err) => log::error!("SendChatAction: {:?}", err),
            }

            let mut client =
                TrackerClient::connect(env::var("GRPC_ADDR").expect("env GRPC_ADDR is not set."))
                    .await
                    .unwrap();
            let request = tonic::Request::new(SupportCouriersRequest {
                tracking_number: tracking_number.clone(),
            });

            if let Ok(response) = client.get_support_couriers(request).await {
                if response.into_inner().couriers.is_empty() {
                    let send_message = SendMessage::new(
                        state.user_id,
                        escape(
                            "⚠️ 지원하는 택배사가 없어요.\n\
                        운송장 번호를 다시 확인하시거나 관리자에게 문의해주세요.",
                        ),
                    )
                    .with_parse_mode(ParseMode::MarkdownV2);
                    api.send_json(&send_message).await.unwrap();
                    Dialogue::exit(state.user_id);
                    return;
                }

                let send_message = SendMessage::new(
                    state.user_id,
                    escape("🚚 운송장 번호를 조회할 택배사를 선택해주세요."),
                )
                .with_parse_mode(ParseMode::MarkdownV2);
                api.send_json(&send_message).await.unwrap();

                Dialogue::next(
                    state.user_id,
                    DialogueStage::SelectedCourier(SelectedCourierState {
                        user_id: state.user_id,
                        tracking_number,
                    }),
                );
            } else {
                let send_message = SendMessage::new(
                    state.user_id,
                    escape("⚠️ 서버에 문제가 있어요. 나중에 다시 시도해주세요."),
                )
                .with_parse_mode(ParseMode::MarkdownV2);
                api.send_json(&send_message).await.unwrap();
                Dialogue::exit(state.user_id);
            }
        }
        DialogueStage::SelectedCourier(state) => {
            match api.send_json(&SendChatAction::new(state.user_id, ChatActionKind::Typing)).await {
                Ok(_) => {},
                Err(err) => log::error!("SendChatAction: {:?}", err),
            }
            
            let mut client =
                TrackerClient::connect(env::var("GRPC_ADDR").expect("env GRPC_ADDR is not set."))
                    .await
                    .unwrap();
            let request = tonic::Request::new(TrackingRequest {
                tracking_number: state.tracking_number,
                courier_id: answer.to_string(),
            });
            if let Ok(response) = client.track(request).await {
                let send_message = SendMessage::new(
                    state.user_id,
                    create_simple_tracking_message(response.into_inner()),
                )
                .with_parse_mode(ParseMode::MarkdownV2);
                api.send_json(&send_message).await.unwrap();
            } else {
                let send_message =
                    SendMessage::new(state.user_id, escape("⚠️ 운송장 정보가 존재하지 않습니다."))
                        .with_parse_mode(ParseMode::MarkdownV2);
                api.send_json(&send_message).await.unwrap();
            }
            Dialogue::exit(state.user_id);
        }
    };
}
