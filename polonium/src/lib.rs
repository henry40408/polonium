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

mod attachment;

pub use attachment::{Attachment, AttachmentError};

/// Pushover API request <https://pushover.net/api#messages>
#[derive(Default, Debug)]
pub struct Request<'a> {
    token: Cow<'a, str>,
    user: Cow<'a, str>,
    message: Cow<'a, str>,
    /// Optional. Device
    pub device: Option<Cow<'a, str>>,
    /// Optional. Title
    pub title: Option<Cow<'a, str>>,
    /// Optional. Render as HTML?
    pub html: Option<HTML>,
    /// Optional. Render with monospace font?
    pub monospace: Option<Monospace>,
    /// Optional. Message timestamp
    pub timestamp: Option<u64>,
    /// Optional. Priority
    pub priority: Option<Priority>,
    /// Optional. URL
    pub url: Option<Cow<'a, str>>,
    /// Optional. URL title
    pub url_title: Option<Cow<'a, str>>,
    /// Optional. Sound
    pub sound: Option<Sound>,
}

/// Render in HTML <https://pushover.net/api#html>
#[derive(Clone, Copy, Debug, strum::ToString)]
pub enum HTML {
    /// Displayed in plain text
    #[strum(serialize = "0")]
    None,
    /// Displayed in HTML
    #[strum(serialize = "1")]
    Enabled,
}

/// Render with monospace <https://pushover.net/api#html>
#[derive(Clone, Copy, Debug, strum::ToString)]
pub enum Monospace {
    /// Displayed in normal font
    #[strum(serialize = "0")]
    None,
    /// Displayed in monospace font
    #[strum(serialize = "1")]
    Enabled,
}

/// Priority <https://pushover.net/api#priority>
#[derive(Clone, Copy, Debug, strum::ToString)]
pub enum Priority {
    /// Normal priority
    #[strum(serialize = "0")]
    Normal,
    /// Lowest priority
    #[strum(serialize = "-2")]
    Lowest,
    /// Low priority
    #[strum(serialize = "-1")]
    Low,
    /// High priority
    #[strum(serialize = "1")]
    High,
    /// Emergency priority
    #[strum(serialize = "2")]
    Emergency,
}

/// Sound <https://pushover.net/api#sounds>
#[derive(Clone, Copy, Debug, strum::ToString)]
#[strum(serialize_all = "lowercase")]
pub enum Sound {
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

/// Notification error
#[derive(Error, Debug)]
pub enum NotificationError {
    /// Error from [`reqwest`] crate
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    /// Error from [`serde_json`] crate
    #[error("deserialization error: {0}")]
    Deserialize(#[from] serde_json::Error),
    /// Wrapped [`crate::AttachmentError`]
    #[error("attachment error: {0}")]
    Attachment(#[from] AttachmentError),
}

/// Request wrapped with attachment
#[derive(Default, Debug)]
pub struct Notification<'a> {
    request: Request<'a>,
    attachment: Option<&'a Attachment>,
}

#[cfg(test)]
fn server_url() -> String {
    mockito::server_url()
}

#[cfg(not(test))]
fn server_url() -> String {
    "https://api.pushover.net".to_string()
}

impl<'a> Notification<'a> {
    /// Creates a [`Notification`]
    pub fn new(token: &'a str, user: &'a str, message: &'a str) -> Self {
        Self {
            request: Request {
                token: token.into(),
                user: user.into(),
                message: message.into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Attach an [`Attachment`]
    pub fn attach(&mut self, attachment: &'a Attachment) {
        self.attachment = Some(attachment);
    }

    /// Send [`Request`] to Pushover API
    pub async fn send(&'a self) -> Result<Response, NotificationError> {
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

        let form = if let Some(a) = self.attachment {
            let part = multipart::Part::bytes(a.content.clone())
                .file_name(a.filename.to_string())
                .mime_str(a.mime_type.as_str())?;
            form.part("attachment", part)
        } else {
            form
        };

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
            Err(e) => Err(NotificationError::Deserialize(e)),
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

/// Pushover API response <https://pushover.net/api#response>
#[derive(Debug, Deserialize)]
pub struct Response {
    /// Status, 1 if success
    pub status: u8,
    /// Randomly generated unique token associated with request
    pub request: String,
    /// Array of string if any error occurred
    pub errors: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use mockito::mock;

    use crate::attachment::Attachment;
    use crate::{server_url, Monospace, Notification, NotificationError, Priority, Sound, HTML};

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

    #[tokio::test]
    async fn test_attach_and_send() -> Result<(), NotificationError> {
        let _m = mock("POST", "/1/messages.json")
            .with_status(200)
            .with_body(r#"{"status":1,"request":"647d2300-702c-4b38-8b2f-d56326ae460b"}"#)
            .create();

        let mut n = build_notification();
        let a = Attachment::new("filename", "plain/text", &[]);
        n.attach(&a);

        let res = n.send().await?;
        assert_eq!(1, res.status);
        assert_eq!("647d2300-702c-4b38-8b2f-d56326ae460b", res.request);
        Ok(())
    }

    #[tokio::test]
    async fn test_attach_url_and_send() -> Result<(), NotificationError> {
        let _m = mock("POST", "/1/messages.json")
            .with_status(200)
            .with_body(r#"{"status":1,"request":"647d2300-702c-4b38-8b2f-d56326ae460b"}"#)
            .create();

        let _n = mock("GET", "/filename.png")
            .with_status(200)
            .with_body(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])
            .create();

        let mut n = build_notification();
        let u = format!("{}/filename.png", server_url());

        let a = Attachment::from_url(&u).await?;
        assert_eq!("filename.png", a.filename);
        assert_eq!("image/png", a.mime_type);
        assert!(a.content.len() > 0);

        n.attach(&a);

        let res = n.send().await?;
        assert_eq!(1, res.status);
        assert_eq!("647d2300-702c-4b38-8b2f-d56326ae460b", res.request);
        Ok(())
    }
}
