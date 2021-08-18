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

use polonium::{Attachment, Notification, HTML};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(about, author)]
struct Opts {
    /// Pushover API token
    #[structopt(short, long, env = "PUSHOVER_TOKEN")]
    token: String,
    /// Pushover user key
    #[structopt(short, long, env = "PUSHOVER_USER")]
    user: String,
    /// message
    #[structopt(short, long)]
    message: String,
    /// verbose
    #[structopt(short, long)]
    verbose: bool,
    /// render as HTML?
    #[structopt(long)]
    html: bool,
    /// attach file
    #[structopt(short, long)]
    file: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opts: Opts = Opts::from_args();

    let mut notification = Notification::new(&opts.token, &opts.user, &opts.message);

    // set extra options
    if opts.html {
        notification.request.html = Some(HTML::Enabled);
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
