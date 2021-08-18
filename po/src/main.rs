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

use polonium::Notification;
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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opts: Opts = Opts::from_args();

    let n = Notification::new(&opts.token, &opts.user, &opts.message);
    let res = n.send().await?;
    if opts.verbose {
        println!("{:?}", res);
    }

    Ok(())
}
