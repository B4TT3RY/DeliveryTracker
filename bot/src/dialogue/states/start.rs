use teloxide::{teloxide, prelude::*};

use crate::dialogue::Dialogue;

use super::receive_tracking_number::ReceiveTrackingNumber;

#[derive(Clone)]
pub struct StartState;

#[teloxide(subtransition)]
async fn start(
    _state: StartState,
    cx: TransitionIn<crate::BotType>,
    _ans: String,
) -> TransitionOut<Dialogue> {
    cx.answer("택배사를 입력해주세용").await?;
    next(ReceiveTrackingNumber)
}