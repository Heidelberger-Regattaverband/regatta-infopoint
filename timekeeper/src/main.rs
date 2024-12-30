mod args;
mod client;
mod messages;

use args::Args;
use clap::Parser;
use client::Client;
use colored::Colorize;
use log::{info, LevelFilter};
use messages::{RequestListOpenHeats, RequestStartList, ResponseListOpenHeats, ResponseStartList};
use std::{io::Result, thread};

fn main() -> Result<()> {
    env_logger::builder().filter_level(LevelFilter::Info).init();
    let args = Args::parse();

    let mut client = Client::new(args.host, args.port)?;

    client.write(&RequestListOpenHeats::new().to_string())?;
    let response = client.receive_all()?;
    let open_heats = ResponseListOpenHeats::new(&response);

    for mut heat in open_heats.heats {
        client.write(&RequestStartList::new(heat.id).to_string())?;
        let response = client.receive_all()?;
        let start_list = ResponseStartList::new(&response);
        heat.boats = Some(start_list.boats);
    }

    info!("Receiving ...");
    thread::spawn(move || loop {
        let received = client.receive_line().unwrap();
        if !received.is_empty() {
            info!("Received: \"{}\"", received.bold());
        }
    })
    .join()
    .unwrap();

    Ok(())
} // the stream is closed here
