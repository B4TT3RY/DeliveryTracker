use serde::{Serialize, Deserialize};
use teloxide::{teloxide, prelude::*};

use crate::dialogue::Dialogue;

#[derive(Clone, Serialize, Deserialize)]
pub struct NothingState;

#[teloxide(subtransition)]
async fn nothing(
    _state: NothingState,
    _cx: TransitionIn<crate::BotType>,
    _ans: String,
) -> TransitionOut<Dialogue> {
    exit()
}
