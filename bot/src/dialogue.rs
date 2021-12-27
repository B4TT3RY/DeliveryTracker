use std::{collections::HashMap, sync::RwLock};

use once_cell::sync::Lazy;

static GLOBAL_DATA: Lazy<RwLock<HashMap<i64, DialogueStage>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub struct Dialogue;

#[derive(Clone, Debug)]
pub enum DialogueStage {
    Start(StartState),
    ReceivedTrackingNumber(ReceivedTrackingNumberState),
    SelectedCourier(SelectedCourierState),
}

#[derive(Clone, Debug)]
pub enum DialogueAnswerKind {
    Message(String),
    CallbackQuery(String),
}

#[derive(Clone, Debug)]
pub enum TypeKind {
    Search,
    Track,
}

#[derive(Clone, Debug)]
pub struct StartState {
    pub kind: TypeKind,
    pub user_id: i64,
}

#[derive(Clone, Debug)]
pub struct ReceivedTrackingNumberState {
    pub kind: TypeKind,
    pub user_id: i64,
    pub tracking_number: Option<String>,
}

#[derive(Clone, Debug)]
pub struct SelectedCourierState {
    pub kind: TypeKind,
    pub user_id: i64,
    pub tracking_number: String,
    pub message_id: i64,
}

impl Dialogue {
    pub fn get(user_id: i64) -> Option<DialogueStage> {
        if let Ok(map) = GLOBAL_DATA.read() {
            map.get(&user_id).map(|stage| stage.clone())
        } else {
            None
        }
    }

    pub fn next(user_id: i64, dialogue_stage: DialogueStage) {
        if let Ok(mut map) = GLOBAL_DATA.write() {
            map.insert(user_id, dialogue_stage);
        }
    }

    pub fn exit(user_id: i64) -> bool {
        if let Ok(mut map) = GLOBAL_DATA.write() {
            map.remove(&user_id).is_some()
        } else {
            false
        }
    }
}
