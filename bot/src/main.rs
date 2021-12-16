#![warn(clippy::all)]

use std::env;

use actix_web::{HttpServer, App, Responder, get, web, post};
use dialogue::Dialogue;
use telbot_hyper::{Api, types::{update::{Update, UpdateKind}, markup::ParseMode}};

mod command_handler;
mod command;
mod dialogue_handler;
mod dialogue;
mod telegram;

#[post("/tg_webhook")]
async fn tg_webhook(update: web::Json<Update>) -> impl Responder {
    let api = Api::new(env::var("BOT_TOKEN").expect("env BOT_TOKEN is not set."));
    
    if let UpdateKind::Message { message } = &update.kind {
        if let Some(text) = message.kind.text() {
            if text.starts_with("/") {
                command_handler::handle_command(&api, &message, text).await;
            } else if let Some(stage) = Dialogue::get(message.chat.id) {
                dialogue_handler::handle_dialogue(&api, stage, text).await;
            } else {
                let reply = &message.reply_text(text).with_parse_mode(ParseMode::MarkdownV2);
                api.send_json(reply)
                    .await
                    .expect("Failed to send message");
            }
        }
    }

    ""
}

#[get("/health")]
async fn health() -> impl Responder {
    "OK"
}

fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    actix_web::rt::System::with_tokio_rt(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(2)
            .thread_name("main-tokio")
            .build()
            .unwrap()
    })
    .block_on(run())
}

async fn run() {
    HttpServer::new(|| {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .service(tg_webhook)
            .service(health)
    })
    .workers(2)
    .bind("0.0.0.0:8080")
    .expect("Couldn't bind to port 8080")
    .run()
    .await
    .unwrap()
}
