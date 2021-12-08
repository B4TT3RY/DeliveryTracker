use teloxide::{teloxide, prelude::*};

use crate::dialogue::Dialogue;

#[derive(Clone)]
pub struct ReceiveTrackingNumber;

#[teloxide(subtransition)]
async fn receive_tracking_number(
    _state: ReceiveTrackingNumber,
    cx: TransitionIn<crate::BotType>,
    ans: String,
) -> TransitionOut<Dialogue> {
    cx.answer(format!("{}를 입력하셨습니다\n택배사를 골라주세용", ans)).await?;
    exit()
}