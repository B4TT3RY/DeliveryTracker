pub mod states;

use derive_more::From;
use serde::{Serialize, Deserialize};
use teloxide::{macros::Transition, prelude::*, types::Message};

use self::states::{StartState, ReceiveTrackingNumberState, NothingState};

#[derive(Transition, Clone, From, Serialize, Deserialize)]
pub enum Dialogue {
    Nothing(NothingState),
    Start(StartState),
    ReceiveTrackingNumber(ReceiveTrackingNumberState),
}

impl Default for Dialogue {
    fn default() -> Self {
        Self::Nothing(NothingState)
    }
}

impl Dialogue {
    pub async fn handler(
        cx: UpdateWithCx<crate::BotType, Message>,
        dialogue: Dialogue
    ) -> TransitionOut<Dialogue> {
        match cx.update.text().map(ToOwned::to_owned) {
            Some(ans) => dialogue.react(cx, ans).await,
            None => next(dialogue),
        }
    }
}