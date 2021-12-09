use teloxide::{teloxide, prelude::*};

use crate::dialogue::Dialogue;

use super::receive_tracking_number::ReceiveTrackingNumberState;

#[derive(Clone)]
pub struct StartState;

#[teloxide(subtransition)]
async fn start(
    _state: StartState,
    cx: TransitionIn<crate::BotType>,
    _ans: String,
) -> TransitionOut<Dialogue> {
    cx.answer("송장번호를 입력해주세용").await?;
    next(ReceiveTrackingNumberState { tracking_number: String::new() })
}