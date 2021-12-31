use bot::tracker::{
    search_response::Status, SupportCouriersResponse, TrackingDetail, TrackingInfo,
    SearchResponse,
};
use chrono::TimeZone;
use chrono_tz::Asia::Seoul;
use telbot_hyper::types::markup::{
    InlineKeyboardButton, InlineKeyboardButtonKind, InlineKeyboardMarkup,
};

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
        product = if let Some(product) = &info.product {
            escape(format!(" ({})", product))
        } else {
            "".to_string()
        }
    )
}

fn create_detail_message(detail: &TrackingDetail) -> String {
    let datetime = Seoul
        .datetime_from_str(&detail.time, "%Y-%m-%d %H:%M:%S")
        .unwrap();

    if detail.message.is_some() && detail.status.is_some() && detail.location.is_some() {
        escape(format!(
            "{} ({}, {}): {}",
            detail.status(),
            datetime.format("%H시 %M분"),
            detail.location(),
            detail.message(),
        ))
    } else if detail.message.is_none() && detail.status.is_some() && detail.location.is_some() {
        escape(format!(
            "{}: [{}] {}",
            datetime.format("%H시 %M분"),
            detail.location(),
            detail.status(),
        ))
    } else if detail.message.is_some() && detail.status.is_none() && detail.location.is_some() {
        escape(format!(
            "{}: [{}] {}",
            datetime.format("%H시 %M분"),
            detail.location(),
            detail.message(),
        ))
    } else if detail.message.is_some() && detail.status.is_none() && detail.location.is_none() {
        escape(format!(
            "{}: {}",
            datetime.format("%H시 %M분"),
            detail.message(),
        ))
    } else {
        String::new()
    }
}

pub fn create_simple_tracking_message(response: SearchResponse) -> String {
    match response.status() {
        Status::Ok => {
            let info = response.tracking_info.unwrap();
            let header = create_info_header_message(&info);
            if info.tracks.len() == 0 {
                return header;
            }
            let last_detail = info.tracks.last().unwrap();

            let datetime = Seoul
                .datetime_from_str(&last_detail.time, "%Y-%m-%d %H:%M:%S")
                .unwrap();

            let detail_message = create_detail_message(last_detail);

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
        Status::RequestFailed => String::new(),
        Status::WrongTrackingNumber => String::new(),
        Status::NotExistsTrackingNumber => String::new(),
    }
}

pub fn create_courier_keyboard(support_couriers: SupportCouriersResponse) -> InlineKeyboardMarkup {
    let rows = support_couriers
        .couriers
        .iter()
        .map(|courier| InlineKeyboardButton {
            text: courier.name.clone(),
            kind: InlineKeyboardButtonKind::Callback {
                callback_data: courier.id.clone(),
            },
        })
        .collect::<Vec<InlineKeyboardButton>>()
        .chunks(2)
        .collect::<Vec<&[InlineKeyboardButton]>>()
        .iter()
        .map(|vec| vec.to_vec())
        .collect::<Vec<Vec<InlineKeyboardButton>>>();

    InlineKeyboardMarkup {
        inline_keyboard: rows,
    }
}
