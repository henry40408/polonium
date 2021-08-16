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

/// Request to Pushover
/// ref: https://pushover.net/api#messages
struct Request<'a> {
    /// Required. API token
    token: &'a str,
    /// Required. User key
    user: &'a str,
    /// Required. Message
    message: &'a str,
    /// Optional. Device
    device: Option<&'a str>,
    /// Optional. Title
    title: Option<&'a str>,
    /// Optional. Render as HTML?
    html: Option<HTML>,
    /// Optional. Render with monospace font?
    monospace: Option<Monospace>,
    /// Optional. Message timestamp
    timestamp: Option<u64>,
    /// Optional. Priority
    priority: Option<Priority>,
    /// Optional. URL
    url: Option<&'a str>,
    /// Optional. URL title
    url_title: Option<&'a str>,
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
    filename: &'a str,
    mime_type: &'a str,
    content: &'a [u8],
}

struct Notification<'a> {
    request: &'a Request<'a>,
    attachment: &'a Attachment<'a>,
}

struct Response<'a> {
    status_code: u8,
    request: &'a str,
    errors: &'a [&'a str],
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
