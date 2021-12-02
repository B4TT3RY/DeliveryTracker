use async_trait::async_trait;

#[async_trait]
pub trait Courier {
    fn id() -> &'static str;
    fn name() -> &'static str;
    fn validate(tracking_number: &str) -> bool;
    async fn track(tracking_number: &str) -> TrackingResult;
}

pub type TrackingResult = Result<TrackingInfo, TrackingError>;

#[derive(Debug)]
pub struct TrackingInfo {
    pub id: String,
    pub name: String,
    pub url: String,
    pub tracking_number: String,
    pub is_delivered: bool,
    pub sender: Option<String>,
    pub receiver: Option<String>,
    pub product: Option<String>,
    pub tracks: Vec<TrackingDetail>,
}

#[derive(Debug)]
pub struct TrackingDetail {
    pub time: String,
    pub message: Option<String>,
    pub status: Option<String>,
    pub location: Option<String>,
    pub live_tracking_url: Option<String>,
}

#[derive(Debug)]
pub enum TrackingError {
    RequestFailed(String),
    WrongTrackingNumber,
    NotExistsTrackingNumber,
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
