mod args;
mod client;
mod messages;

use args::Args;
use clap::Parser;
use client::Client;
use log::{info, LevelFilter};
use messages::RequestOpenHeats;
use std::{io::Result, thread};

fn main() -> Result<()> {
    env_logger::builder().filter_level(LevelFilter::Info).init();
    let args = Args::parse();

    let mut client = Client::new(args.host, args.port)?;
    client.write(&RequestOpenHeats::new().to_string())?;

    info!("Receiving ...");
    thread::spawn(move || loop {
        let received = client.receive().unwrap();
        if !received.is_empty() {
            info!("Received: \"{}\"", received);
        }
    })
    .join()
    .unwrap();

    Ok(())
} // the stream is closed here
