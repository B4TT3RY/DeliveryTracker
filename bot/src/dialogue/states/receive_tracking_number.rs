use teloxide::{teloxide, prelude::*};

use crate::dialogue::Dialogue;

#[derive(Clone)]
pub struct ReceiveTrackingNumberState {
    pub tracking_number: String,
}

#[teloxide(subtransition)]
async fn receive_tracking_number(
    state: ReceiveTrackingNumberState,
    cx: TransitionIn<crate::BotType>,
    ans: String,
) -> TransitionOut<Dialogue> {
    cx.answer(format!("{}를 입력하셨습니다\n택배사를 골라주세용", if state.tracking_number.is_empty() { ans } else { state.tracking_number })).await?;
    exit()
}