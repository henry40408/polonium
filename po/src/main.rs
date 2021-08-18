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

//! Po is a command line application based on Polonium

use polonium::{Attachment, Monospace, Notification, HTML};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(about, author)]
struct Opts {
    /// your application's API token <https://pushover.net/api#identifiers>
    #[structopt(short, long, env = "PUSHOVER_TOKEN")]
    token: String,
    /// the user / group key (not e-mail address) of your user (or you) <https://pushover.net/api#identifiers>
    #[structopt(short, long, env = "PUSHOVER_USER")]
    user: String,
    /// your message <https://pushover.net/api#messages>
    #[structopt(short, long)]
    message: String,
    /// verbose
    #[structopt(short, long)]
    verbose: bool,
    /// To enable HTML formatting <https://pushover.net/api#html>
    #[structopt(long)]
    html: bool,
    /// To enable monospace messages <https://pushover.net/api#html>
    #[structopt(long)]
    monospace: bool,
    /// your user's device name to send the message directly to that device, rather than all of the user's devices <https://pushover.net/api#identifiers>
    #[structopt(long)]
    device: Option<String>,
    /// your message's title, otherwise your app's name is used <https://pushover.net/api#messages>
    #[structopt(long)]
    title: Option<String>,
    /// a Unix timestamp of your message's date and time to display to the user, rather than the time your message is received by our API <https://pushover.net/api#timestamp>
    #[structopt(long)]
    timestamp: Option<u64>,
    /// attach file as notification attachment
    #[structopt(short, long)]
    file: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opts: Opts = Opts::from_args();

    let mut notification = Notification::new(&opts.token, &opts.user, &opts.message);

    // set extra options
    if let Some(ref d) = opts.device {
        notification.request.device = Some(d.into());
    }
    if let Some(ref t) = opts.title {
        notification.request.title = Some(t.into());
    }
    if let Some(ref t) = opts.timestamp {
        notification.request.timestamp = Some(*t);
    }

    if opts.html {
        notification.request.html = Some(HTML::Enabled);
        if opts.monospace {
            notification.request.monospace = Some(Monospace::Enabled);
        }
    }

    // send request with file as attachment
    let attachment;
    if let Some(p) = &opts.file {
        attachment = Attachment::from_path(p).await?;
        notification.attach(&attachment);
    }

    // send request
    let res = notification.send().await?;
    if opts.verbose {
        println!("{:?}", res);
    }

    Ok(())
}
