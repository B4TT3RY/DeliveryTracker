#![warn(clippy::all)]

use command::Command;
use dialogue::Dialogue;
use teloxide::dispatching::dialogue::{SqliteStorage, Storage};
use teloxide::dispatching::dialogue::serializer::Json;
use teloxide::{Bot, prelude::*, types::ParseMode, adaptors::DefaultParseMode};
use teloxide::utils::command::BotCommand;
use tokio::runtime::Runtime;

mod command;
mod dialogue;
mod telegram;

pub type BotType = DefaultParseMode<AutoSend<Bot>>;
type StorageError = <SqliteStorage<Json> as Storage<Dialogue>>::Error;
type In = DialogueWithCx<BotType, Message, Dialogue, StorageError>;

fn main() {
    // teloxide::enable_logging!();
    pretty_env_logger::formatted_builder()
            .write_style(pretty_env_logger::env_logger::WriteStyle::Auto)
            .filter(Some("teloxide"), log::LevelFilter::Trace)
            .init();
    Runtime::new().unwrap().block_on(run());
}

async fn run() {
    log::info!("Starting DeliveryTrackerBot...");

    let bot = Bot::from_env().auto_send().parse_mode(ParseMode::MarkdownV2);

    Dispatcher::new(bot)
        .messages_handler(DialogueDispatcher::with_storage(
            |DialogueWithCx { cx, dialogue }: In| async move {
                    let mut dialogue = dialogue.expect("std::convert::Infallible");

                    let parse = Command::parse(cx.update.text().unwrap_or_default(), "DeliveryTracker");
                    if let Ok(command) = parse {
                        let response = Command::handler(&cx, command).await;
                        if let Ok(next_dialogue) = response {
                            dialogue = next_dialogue;
                        } else if let Err(error) = response {
                            log::error!("command handler: {}", error);
                        }
                    }
                    Dialogue::handler(cx, dialogue).await.expect("Something wrong with Dialog!")
                },
                SqliteStorage::open("db.sqlite", Json).await.unwrap(),
        ))
        .setup_ctrlc_handler()
        .dispatch()
        .await;
}
