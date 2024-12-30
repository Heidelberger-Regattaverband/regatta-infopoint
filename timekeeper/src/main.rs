mod args;
mod client;
mod messages;

use args::Args;
use clap::Parser;
use client::Client;
use log::{info, LevelFilter};
use messages::{RequestListOpenHeats, ResponseListOpenHeats};
use std::{io::Result, thread};

fn main() -> Result<()> {
    env_logger::builder().filter_level(LevelFilter::Info).init();
    let args = Args::parse();

    let mut client = Client::new(args.host, args.port)?;
    client.write(&RequestListOpenHeats::new().to_string())?;
    let response = client.receive_all()?;
    let open_heats = ResponseListOpenHeats::new(&response);

    for heat in open_heats.heats {
        info!("Heat: {}", heat.number);
    }

    info!("Receiving ...");
    thread::spawn(move || loop {
        let received = client.receive_line().unwrap();
        if !received.is_empty() {
            info!("Received: \"{}\"", received);
        }
    })
    .join()
    .unwrap();

    Ok(())
} // the stream is closed here
