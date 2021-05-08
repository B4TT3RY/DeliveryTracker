use anyhow::{anyhow, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::Value;

use super::{Courier, DeliveryStatus, StateType, TrackingStatus};

pub const URL: &str = "https://www.fedex.com/trackingCal/track";
pub const ID: &str = "us.fedex";
pub const NAME: &str = "Fedex";

static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^(\d{12})$"#).unwrap());

pub fn validate(courier: &Courier) -> Result<()> {
    if !REGEX.is_match(&courier.tracking_number) {
        Err(anyhow!("운송장번호 12자리를 입력해주세요."))
    } else {
        Ok(())
    }
}

pub fn state_from(status: &str) -> StateType {
    use StateType::*;
    match status {
        "OC" => InformationReceived,
        "PU" => AtPickup,
        "DP" => InTransitSend,
        "AR" | "CC" => InTransitReceived,
        "IT" => InTransit,
        // "배달지도착" => ReadyForDelivery,
        "OD" => OutForDelivery,
        "DL" => Delivered,
        _ => Unknown,
    }
}

pub async fn track(courier: &Courier) -> Result<DeliveryStatus> {
    let form = [
        ("action", "trackpackages"),
        ("format", "json"),
        ("data", &format!(r#"
            {{
                "TrackPackagesRequest":{{
                    "appDeviceType":"DESKTOP",
                    "appType":"WTRK",
                    "processingParameters":{{}},
                    "uniqueKey":"",
                    "supportCurrentLocation":true,
                    "supportHTML":true,
                    "trackingInfoList":[
                        {{
                            "trackNumberInfo":{{
                                "trackingNumber":"{}",
                                "trackingQualifier":null,
                                "trackingCarrier":null
                            }}
                        }}
                    ]
                }}
            }}"#,
            courier.tracking_number)
        ),
        ("locale", "ko_KR"),
        ("version", "1"),
    ];
    let response = surf::post(URL)
        .body(form.iter().map(|(key, value)| format!("&{}={}", key, value)).collect::<String>())
        .header("Content-Type", "application/x-www-form-urlencoded; charset=UTF-8")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/89.0.4389.90 Safari/537.36")
        .recv_string()
        .await
        .map_err(|err| anyhow!(err))?;

    let json: Value = serde_json::from_str(&response)?;

    if !json["TrackPackagesResponse"]["packageList"][0]["errorList"][0]["code"].as_str().unwrap().is_empty() {
        return Ok(
            DeliveryStatus {
                id: ID.to_string(),
                name: NAME.to_string(),
                tracking_number: None,
                sender: None,
                receiver: None,
                product: None,
                tracks: None,
            }
        );
    }

    let mut tracks: Vec<TrackingStatus> = Vec::new();

    for scan in 
        json
        ["TrackPackagesResponse"]
        ["packageList"]
        [0]
        ["scanEventList"]
        .as_array()
        .unwrap()
    {
        let state = scan["statusCD"].as_str().unwrap();
        let status = scan["status"].as_str().unwrap().to_string();

        tracks.push(TrackingStatus {
            state: state_from(state),
            time: format!("{} {}", scan["date"].as_str().unwrap(), scan["time"].as_str().unwrap()),
            location: Some(scan["scanLocation"].as_str().unwrap().to_string()),
            status: format!("[{}] {}", state, status),
            message: None,
        });
    }

    tracks.reverse();
    // tracks.sort_by_key(|k| StateType::get_priority(k.state));

    Ok(DeliveryStatus {
        id: ID.to_string(),
        name: format!(
            "{} ({})",
            json["TrackPackagesResponse"]["packageList"][0]["trackingCarrierDesc"].as_str().unwrap(),
            json["TrackPackagesResponse"]["packageList"][0]["serviceDesc"].as_str().unwrap(),
        ),
        tracking_number: Some(courier.tracking_number.to_string()),
        sender: Some(format!(
            "{}, {} {}",
            json["TrackPackagesResponse"]["packageList"][0]["shipperCity"].as_str().unwrap(),
            json["TrackPackagesResponse"]["packageList"][0]["shipperStateCD"].as_str().unwrap(),
            json["TrackPackagesResponse"]["packageList"][0]["shipperCntryCD"].as_str().unwrap(),
        )),
        receiver: Some(format!(
            "{}, {} {}",
            json["TrackPackagesResponse"]["packageList"][0]["recipientCity"].as_str().unwrap(),
            json["TrackPackagesResponse"]["packageList"][0]["recipientStateCD"].as_str().unwrap(),
            json["TrackPackagesResponse"]["packageList"][0]["recipientCntryCD"].as_str().unwrap(),
        )),
        product: None,
        tracks: Some(tracks),
    })
}
