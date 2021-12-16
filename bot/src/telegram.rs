use bot::tracker::{TrackingResponse, tracking_response::Status, TrackingInfo};
use chrono::TimeZone;
use chrono_tz::Asia::Seoul;

pub fn escape<S>(input: S) -> String
where
    S: Into<String>,
{
    const ESCAPE: [char; 18] = [
        '_', '*', '[', ']', '(', ')', '~', '`', '>', '#', '+', '-', '=', '|', '{', '}', '.', '!',
    ];
    let mut output = String::new();
    for c in input.into().chars() {
        if ESCAPE.contains(&c) {
            output.push('\\');
        }
        output.push(c);
    }
    output
}

fn create_info_header_message(info: &TrackingInfo) -> String {
    format!(
        "📦 *{name}* {tracking_number}\n\
        {sender} ▶️ {receiver}{product}",
        name = info.name,
        tracking_number = info.tracking_number,
        sender = escape(info.sender.as_ref().unwrap_or(&"정보 없음".to_string())),
        receiver = escape(info.receiver.as_ref().unwrap_or(&"정보 없음".to_string())),
        product = 
            if let Some(product) = &info.product {
                escape(format!(
                    " ({})",
                    product
                ))
            } else {
                "".to_string()
            }
    )
}

pub fn create_simple_tracking_message(response: TrackingResponse) -> String {
    match response.status() {
        Status::Ok => {
            let info = response.tracking_info.unwrap();
            let header = create_info_header_message(&info);
            if info.tracks.len() == 0 {
                return header;
            }
            let last_info = info.tracks.last().unwrap();

            let datetime = Seoul
                .datetime_from_str(&last_info.time, "%Y-%m-%d %H:%M:%S").unwrap();

            let detail_message =
                if last_info.message.is_some() && last_info.status.is_some() && last_info.location.is_some() {
                    escape(format!(
                        "{} ({}, {}): {}",
                        last_info.status(),
                        datetime.format("%H시 %M분"),
                        last_info.location(),
                        last_info.message(),
                    ))
                } else if last_info.message.is_none() && last_info.status.is_some() && last_info.location.is_some() {
                    escape(format!(
                        "{}: [{}] {}",
                        datetime.format("%H시 %M분"),
                        last_info.location(),
                        last_info.status(),
                    ))
                } else if last_info.message.is_some() && last_info.status.is_none() && last_info.location.is_some() {
                    escape(format!(
                        "{}: [{}] {}",
                        datetime.format("%H시 %M분"),
                        last_info.location(),
                        last_info.message(),
                    ))
                } else if last_info.message.is_some() && last_info.status.is_none() && last_info.location.is_none() {
                    escape(format!(
                        "{}: {}",
                        datetime.format("%H시 %M분"),
                        last_info.message(),
                    ))
                } else {
                    String::new()
                };

            format!(
                "{}\n\
                \n\
                *{}*\n\
                {}",
                header,
                datetime.format("%Y년 %m월 %d일"),
                detail_message
            )
        }
        Status::RequestFailed =>{
            String::new()
        }
        Status::WrongTrackingNumber => {
            String::new()
        }
        Status::NotExistsTrackingNumber => {
            String::new()
        }
    }
}