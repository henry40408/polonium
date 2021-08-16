#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

//! Polonium is Pushover API wrapper with attachment support in Rust 2018 edition

use std::borrow::Cow;

use reqwest::multipart;
use serde::Deserialize;
use thiserror::Error;

/// Request to Pushover
/// ref: https://pushover.net/api#messages
#[derive(Default)]
struct Request<'a> {
    /// Required. API token
    token: Cow<'a, str>,
    /// Required. User key
    user: Cow<'a, str>,
    /// Required. Message
    message: Cow<'a, str>,
    /// Optional. Device
    device: Option<Cow<'a, str>>,
    /// Optional. Title
    title: Option<Cow<'a, str>>,
    /// Optional. Render as HTML?
    html: Option<HTML>,
    /// Optional. Render with monospace font?
    monospace: Option<Monospace>,
    /// Optional. Message timestamp
    timestamp: Option<u64>,
    /// Optional. Priority
    priority: Option<Priority>,
    /// Optional. URL
    url: Option<Cow<'a, str>>,
    /// Optional. URL title
    url_title: Option<Cow<'a, str>>,
    /// Optional. Sound
    sound: Option<Sound>,
}

/// Render in HTML
/// ref: https://pushover.net/api#html
enum HTML {
    None,
    Enabled,
}

/// Render with monospace
/// ref: https://pushover.net/api#html
enum Monospace {
    None,
    Enabled,
}

/// Priority
/// ref: https://pushover.net/api#priority
enum Priority {
    Normal,
    Lowest,
    Low,
    High,
    Emergency,
}

/// Sound
/// ref: https://pushover.net/api#sounds
enum Sound {
    /// pushover - Pushover (default)
    Pushover,
    /// bike - Bike
    Bike,
    /// bugle - Bugle
    Bugle,
    /// cashregister - Cash Register
    CashRegister,
    /// classical - Classical
    Classical,
    /// cosmic - Cosmic
    Cosmic,
    /// falling - Falling
    Falling,
    /// gamelan - Gamelan
    GameLan,
    /// incoming - Incoming
    Incoming,
    /// intermission - Intermission
    Intermission,
    /// magic - Magic
    Magic,
    /// mechanical - Mechanical
    Mechanical,
    /// pianobar - Piano Bar
    PianoBar,
    /// siren - Siren
    Siren,
    /// spacealarm - Space Alarm
    SpaceAlarm,
    /// tugboat - Tug Boat
    Tugboat,
    /// alien - Alien Alarm (long)
    Alien,
    /// climb - Climb (long)
    Climb,
    /// persistent - Persistent (long)
    Persistent,
    /// echo - Pushover Echo (long)
    Echo,
    /// updown - Up Down (long)
    UpDown,
    /// vibrate - Vibrate Only
    Vibrate,
    /// none - None (silent)
    None,
}

struct Attachment<'a> {
    filename: Cow<'a, str>,
    mime_type: Cow<'a, str>,
    content: &'a [u8],
}

#[derive(Default)]
struct Notification<'a> {
    request: Request<'a>,
    attachment: Option<&'a Attachment<'a>>,
}

#[cfg(test)]
fn server_url() -> String {
    mockito::server_url()
}

#[cfg(not(test))]
fn server_url() -> String {
    "https://api.pushover.net".to_string()
}

#[derive(Error, Debug)]
enum NotificationError {
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("deserialization error: {0}")]
    DeserializeError(#[from] serde_json::Error),
    #[error("unknown")]
    Unknown,
}

impl<'a> Notification<'a> {
    fn new(token: &'a str, user: &'a str, message: &'a str) -> Self {
        Self {
            request: Request {
                token: Cow::from(token),
                user: Cow::from(user),
                message: Cow::from(message),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    async fn send(&self) -> Result<Response, NotificationError> {
        let form = multipart::Form::new()
            .text("token", self.request.token.to_string())
            .text("user", self.request.user.to_string())
            .text("message", self.request.message.to_string());

        let form = Self::append_part(form, "device", self.request.device.as_ref());

        let uri = format!("{0}/1/messages.json", server_url());
        let client = reqwest::Client::new();
        let body = client
            .post(&uri)
            .multipart(form)
            .send()
            .await?
            .text()
            .await?;
        match serde_json::from_str(&body) {
            Ok(r) => Ok(r),
            Err(e) => Err(NotificationError::DeserializeError(e)),
        }
    }

    fn append_part<T: ToString>(
        form: multipart::Form,
        name: &'static str,
        value: Option<&T>,
    ) -> multipart::Form {
        if let Some(v) = value {
            form.text(name, v.to_string())
        } else {
            form
        }
    }
}

#[derive(Deserialize)]
struct Response {
    status: u8,
    request: String,
    errors: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use mockito::{mock, Mock};

    use crate::{Notification, NotificationError};

    #[test]
    fn test_new() {
        build_notification();
    }

    #[tokio::test]
    async fn test_send() -> Result<(), NotificationError> {
        let _m = mock("POST", "/1/messages.json")
            .with_status(200)
            .with_body(r#"{"status":1,"request":"647d2300-702c-4b38-8b2f-d56326ae460b"}"#)
            .create();
        let n = build_notification();
        let res = n.send().await?;
        assert_eq!(1, res.status);
        assert_eq!("647d2300-702c-4b38-8b2f-d56326ae460b", res.request);
        assert!(res.errors.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn test_device() -> Result<(), NotificationError> {
        let _m = mock("POST", "/1/messages.json")
            .with_status(200)
            .with_body(r#"{"status":1,"request":"647d2300-702c-4b38-8b2f-d56326ae460b"}"#)
            .create();

        let mut n = build_notification();
        n.request.device = Some("device".into());

        let res = n.send().await?;
        assert_eq!(1, res.status);
        assert_eq!("647d2300-702c-4b38-8b2f-d56326ae460b", res.request);
        assert!(res.errors.is_none());

        Ok(())
    }

    fn build_notification<'a>() -> Notification<'a> {
        let user = "user";
        let token = "token";
        let message = "message";
        Notification::new(token, user, message)
    }
}
