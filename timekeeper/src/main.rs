mod args;
mod client;
mod messages;
mod utils;

use args::Args;
use clap::Parser;
use client::Client;
use colored::Colorize;
use log::{debug, info, warn};
use messages::{Heat, RequestListOpenHeats, RequestStartList, ResponseListOpenHeats, ResponseStartList};
use std::{io::Result, thread};

fn main() -> Result<()> {
    env_logger::builder().init();
    let args = Args::parse();

    let mut client = Client::new(args.host, args.port)?;
    let open_heats = get_open_heats(&mut client)?;
    debug!("Open heats: {:#?}", open_heats);

    info!("Receiving events ...");
    thread::spawn(move || loop {
        let received = client.receive_line().unwrap();
        if !received.is_empty() {
            debug!("Received: \"{}\"", utils::print_whitespaces(&received).bold());
            parse_event(&received);
        }
    })
    .join()
    .unwrap();

    Ok(())
} // the stream is closed here

fn parse_event(event: &str) {
    let parts: Vec<&str> = event.split_whitespace().collect();
    if parts.len() != 4 {
        warn!("Invalid event format: {}", event);
        return;
    }

    let action = parts[0];
    let heat_number: usize = match parts[1].parse() {
        Ok(number) => number,
        Err(_) => {
            warn!("Invalid heat number: {}", parts[1]);
            return;
        }
    };
    let heat_id: usize = match parts[2].parse() {
        Ok(id) => id,
        Err(_) => {
            warn!("Invalid heat ID: {}", parts[2]);
            return;
        }
    };
    let status: u8 = match parts[3].parse() {
        Ok(id) => id,
        Err(_) => {
            warn!("Invalid status: {}", parts[3]);
            return;
        }
    };

    match action {
        "!OPEN+" => {
            debug!("Opening heat: {}, id: {}, status: {}", heat_number, heat_id, status);
            // Handle opening heat logic here
        }
        "!OPEN-" => {
            debug!("Closing heat: {}, id: {}, status: {}", heat_number, heat_id, status);
            // Handle closing heat logic here
        }
        _ => {
            debug!("Unknown action: {}", action);
        }
    }
}

fn get_open_heats(client: &mut Client) -> Result<Vec<Heat>> {
    client.write(&RequestListOpenHeats::new().to_string())?;
    let response = client.receive_all()?;
    let mut open_heats = ResponseListOpenHeats::new(&response);

    for heat in open_heats.heats.iter_mut() {
        client.write(&RequestStartList::new(heat.id).to_string())?;
        let response = client.receive_all()?;
        let start_list = ResponseStartList::new(response);
        heat.boats = Some(start_list.boats);
    }

    Ok(open_heats.heats)
}
