use async_trait::async_trait;

use crate::tracker;

#[async_trait]
pub trait Courier {
    fn id() -> &'static str;
    fn name() -> &'static str;
    fn validate(tracking_number: &str) -> bool;
    async fn track(tracking_number: &str) -> TrackingResult;
}

pub type TrackingResult = Result<tracker::TrackingInfo, TrackingError>;

#[derive(Debug)]
pub enum TrackingError {
    RequestFailed(String),
    WrongTrackingNumber(String),
    NotExistsTrackingNumber,
    ParsingError(String),
}

impl From<reqwest::Error> for TrackingError {
    fn from(error: reqwest::Error) -> Self {
        TrackingError::RequestFailed(error.to_string())
    }
}

impl From<reqwest::header::ToStrError> for TrackingError {
    fn from(error: reqwest::header::ToStrError) -> Self {
        TrackingError::RequestFailed(error.to_string())
    }
}

impl From<regex::Error> for TrackingError {
    fn from(error: regex::Error) -> Self {
        TrackingError::ParsingError(error.to_string())
    }
}

impl From<chrono::ParseError> for TrackingError {
    fn from(error: chrono::ParseError) -> Self {
        TrackingError::ParsingError(error.to_string())
    }
}

impl From<serde_json::Error> for TrackingError {
    fn from(error: serde_json::Error) -> Self {
        TrackingError::ParsingError(error.to_string())
    }
}