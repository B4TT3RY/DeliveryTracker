#![warn(clippy::all)]

use std::env;

use telbot_reqwest::Api;
use telbot_types::{update::{GetUpdates, UpdateKind}, markup::ParseMode};
use tokio::runtime::Runtime;

mod command_handler;
mod command;
mod dialogue;
mod telegram;

fn main() {
    Runtime::new().unwrap().block_on(run());
}

async fn run() {
    let api = Api::new(env::var("BOT_TOKEN").expect("env BOT_TOKEN is not set."));

    let mut offset = 0u32;

    loop {
        let get_updates = GetUpdates::new().with_offset(offset as i32).with_timeout(1);
        let updates = api.send_json(&get_updates).await.unwrap();
        for update in updates {
            if let UpdateKind::Message { message } = update.kind {
                if let Some(text) = message.text() {
                    if text.starts_with("/") {
                        command_handler::handle_command(&api, &message, text).await;
                    } else {
                        let reply = &message.reply_text(text).with_parse_mode(ParseMode::MarkdownV2);
                        api.send_json(reply)
                            .await
                            .expect("Failed to send message");
                    }
                }
            }
            offset = offset.max(update.update_id + 1);
        }
    }
}
