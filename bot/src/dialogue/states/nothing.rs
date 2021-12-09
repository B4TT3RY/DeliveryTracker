use teloxide::{teloxide, prelude::*};

use crate::dialogue::Dialogue;

#[derive(Clone)]
pub struct NothingState;

#[teloxide(subtransition)]
async fn start(
    _state: NothingState,
    _cx: TransitionIn<crate::BotType>,
    _ans: String,
) -> TransitionOut<Dialogue> {
    exit()
}