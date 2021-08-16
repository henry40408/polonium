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
#[derive(strum::ToString)]
enum HTML {
    #[strum(serialize = "0")]
    None,
    #[strum(serialize = "1")]
    Enabled,
}

/// Render with monospace
/// ref: https://pushover.net/api#html
#[derive(strum::ToString)]
enum Monospace {
    #[strum(serialize = "0")]
    None,
    #[strum(serialize = "1")]
    Enabled,
}

/// Priority
/// ref: https://pushover.net/api#priority
#[derive(strum::ToString)]
enum Priority {
    #[strum(serialize = "0")]
    Normal,
    #[strum(serialize = "-2")]
    Lowest,
    #[strum(serialize = "-1")]
    Low,
    #[strum(serialize = "1")]
    High,
    #[strum(serialize = "2")]
    Emergency,
}

/// Sound
/// ref: https://pushover.net/api#sounds
#[derive(strum::ToString)]
#[strum(serialize_all = "lowercase")]
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
        let form = Self::append_part(form, "title", self.request.title.as_ref());
        let form = Self::append_part(form, "html", self.request.html.as_ref());
        let form = Self::append_part(form, "monospace", self.request.monospace.as_ref());
        let form = Self::append_part(form, "timestamp", self.request.timestamp.as_ref());
        let form = Self::append_part(form, "priority", self.request.priority.as_ref());
        let form = Self::append_part(form, "url", self.request.url.as_ref());
        let form = Self::append_part(form, "url_title", self.request.url_title.as_ref());
        let form = Self::append_part(form, "sound", self.request.sound.as_ref());

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
    use mockito::mock;

    use crate::{Monospace, Notification, NotificationError, Priority, Sound, HTML};

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

    #[test]
    fn test_html() {
        assert_eq!("0", HTML::None.to_string());
        assert_eq!("1", HTML::Enabled.to_string());
    }

    #[test]
    fn test_monospace() {
        assert_eq!("0", Monospace::None.to_string());
        assert_eq!("1", Monospace::Enabled.to_string());
    }

    #[test]
    fn test_priority() {
        assert_eq!("-2", Priority::Lowest.to_string());
        assert_eq!("-1", Priority::Low.to_string());
        assert_eq!("0", Priority::Normal.to_string());
        assert_eq!("1", Priority::High.to_string());
        assert_eq!("2", Priority::Emergency.to_string());
    }

    #[test]
    fn test_sound() {
        assert_eq!("pushover", Sound::Pushover.to_string());
        assert_eq!("bike", Sound::Bike.to_string());
        assert_eq!("bugle", Sound::Bugle.to_string());
        assert_eq!("cashregister", Sound::CashRegister.to_string());
        assert_eq!("classical", Sound::Classical.to_string());
        assert_eq!("cosmic", Sound::Cosmic.to_string());
        assert_eq!("falling", Sound::Falling.to_string());
        assert_eq!("gamelan", Sound::GameLan.to_string());
        assert_eq!("incoming", Sound::Incoming.to_string());
        assert_eq!("intermission", Sound::Intermission.to_string());
        assert_eq!("magic", Sound::Magic.to_string());
        assert_eq!("mechanical", Sound::Mechanical.to_string());
        assert_eq!("pianobar", Sound::PianoBar.to_string());
        assert_eq!("siren", Sound::Siren.to_string());
        assert_eq!("spacealarm", Sound::SpaceAlarm.to_string());
        assert_eq!("tugboat", Sound::Tugboat.to_string());
        assert_eq!("alien", Sound::Alien.to_string());
        assert_eq!("climb", Sound::Climb.to_string());
        assert_eq!("persistent", Sound::Persistent.to_string());
        assert_eq!("echo", Sound::Echo.to_string());
        assert_eq!("updown", Sound::UpDown.to_string());
        assert_eq!("vibrate", Sound::Vibrate.to_string());
        assert_eq!("none", Sound::None.to_string());
    }
}
