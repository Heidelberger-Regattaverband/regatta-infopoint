mod args;
mod client;
mod error;
mod messages;
mod utils;

use args::Args;
use clap::Parser;
use client::Client;
use error::MessageErr;
use log::debug;

fn main() -> Result<(), MessageErr> {
    env_logger::builder().init();
    let args = Args::parse();

    let mut client = Client::new(args.host, args.port, args.timeout).map_err(MessageErr::IoError)?;
    let open_heats = client.read_open_heats()?;
    debug!("Open heats: {:#?}", open_heats);

    client
        .start_receiving_events()
        .map_err(MessageErr::IoError)?
        .join()
        .unwrap();

    Ok(())
} // the stream is closed here
