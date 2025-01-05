mod args;
mod client;
mod error;
mod messages;
mod utils;

use args::Args;
use clap::Parser;
use client::Client;
use colored::Colorize;
use error::MessageErr;
use log::{debug, info, warn};
use messages::EventHeatChanged;
use std::thread;

fn main() -> Result<(), MessageErr> {
    env_logger::builder().init();
    let args = Args::parse();

    let mut client = Client::new(args.host, args.port, args.timeout).map_err(MessageErr::IoError)?;
    let open_heats = client.read_open_heats()?;
    debug!("Open heats: {:#?}", open_heats);

    info!("Receiving events ...");
    thread::spawn(move || loop {
        let received = client.receive_line().unwrap();
        if !received.is_empty() {
            debug!("Received: \"{}\"", utils::print_whitespaces(&received).bold());
            let event_opt = EventHeatChanged::parse(&received);
            match event_opt {
                Ok(mut event) => {
                    if event.opened {
                        client.read_start_list(&mut event.heat).unwrap();
                    }
                }
                Err(err) => handle_error(err),
            }
        }
    })
    .join()
    .unwrap();

    Ok(())
} // the stream is closed here

fn handle_error(err: MessageErr) {
    match err {
        MessageErr::ParseError(parse_err) => {
            warn!("Error parsing number: {}", parse_err);
        }
        MessageErr::IoError(io_err) => {
            warn!("I/O error: {}", io_err);
        }
        MessageErr::InvalidMessage(message) => {
            warn!("Invalid message: {}", message);
        }
    }
}
