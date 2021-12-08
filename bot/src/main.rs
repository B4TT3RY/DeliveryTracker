#![warn(clippy::all)]

use command::Command;
use dialogue::Dialogue;
use teloxide::{Bot, prelude::*, types::ParseMode, adaptors::DefaultParseMode, dispatching::dialogue::InMemStorageError};
use tokio::runtime::Runtime;
use tokio_stream::wrappers::UnboundedReceiverStream;

mod command;
mod dialogue;

pub type BotType = DefaultParseMode<AutoSend<Bot>>;
type In = DialogueWithCx<BotType, Message, Dialogue, InMemStorageError>;

fn main() {
    teloxide::enable_logging!();
    Runtime::new().unwrap().block_on(run());
}

async fn run() {
    log::info!("Starting DeliveryTrackerBot...");

    let bot = Bot::from_env().auto_send().parse_mode(ParseMode::MarkdownV2);
    let bot_name = "DeliveryTracker".to_string();

    Dispatcher::new(bot)
        .messages_handler(move |rx: DispatcherHandlerRx<BotType, Message>| {
            UnboundedReceiverStream::new(rx).commands::<Command, String>(bot_name).for_each_concurrent(
                None,
                move |(cx, cmd)| {
                    async move {
                        Command::handler(cx, cmd).await.log_on_error().await;
                    }
                },
            )
        })
        // .messages_handler(DialogueDispatcher::new(
        //     move |DialogueWithCx { cx, dialogue }: In| {
        //         async move {
        //             let dialogue = dialogue.expect("std::convert::Infallible");
        //             Dialogue::handler(cx, dialogue).await.expect("Something wrong with Dialog!")
        //         }
        //     }
        // ))
        .setup_ctrlc_handler()
        .dispatch()
        .await;
}
