#![warn(clippy::all)]

use std::env;

use actix_web::{get, post, web, App, HttpServer, Responder};
use dialogue::{Dialogue, DialogueAnswerKind};
use dotenv::dotenv;
use telbot_hyper::{
    types::{
        update::{Update, UpdateKind},
    },
    Api,
};

mod command;
mod command_handler;
mod dialogue;
mod dialogue_handler;
mod telegram;

#[post("/tg_webhook")]
async fn tg_webhook(update: web::Json<Update>) -> impl Responder {
    let api = Api::new(env::var("BOT_TOKEN").expect("env BOT_TOKEN is not set."));

    if let UpdateKind::Message { message } = &update.kind {
        if let Some(text) = message.kind.text() {
            if text.starts_with("/") {
                command_handler::handle_command(&api, &message, text).await;
            } else if let Some(stage) = Dialogue::get(message.chat.id) {
                dialogue_handler::handle_dialogue(&api, stage, DialogueAnswerKind::Message(text.to_string())).await;
            } else {
                let reply = &message
                    .reply_text(text);
                api.send_json(reply).await.expect("Failed to send message");
            }
        }
    } else if let UpdateKind::CallbackQuery { callback_query } = &update.kind {
        if let Some(message) = &callback_query.message {
            if let Some(stage) = Dialogue::get(message.chat.id) {
                let answer = callback_query.data.as_ref().and_then(|str| Some(str.to_string())).unwrap_or_default();
                dialogue_handler::handle_dialogue(&api, stage, DialogueAnswerKind::CallbackQuery(answer)).await;
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
    dotenv().ok();
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
    let port = env::var("PORT").expect("env PORT is not set.");
    HttpServer::new(|| {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .service(tg_webhook)
            .service(health)
    })
    .workers(2)
    .bind(format!("0.0.0.0:{}", port))
    .expect(&format!("Couldn't bind to port {}", port))
    .run()
    .await
    .unwrap()
}
