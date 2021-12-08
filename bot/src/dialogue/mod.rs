mod states;

use derive_more::From;
use teloxide::{macros::Transition, prelude::*, types::Message};

use self::states::{StartState, ReceiveTrackingNumber};

#[derive(Transition, Clone, From)]
pub enum Dialogue {
    Start(StartState),
    ReceiveTrackingNumber(ReceiveTrackingNumber),
}

impl Default for Dialogue {
    fn default() -> Self {
        Self::Start(StartState)
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