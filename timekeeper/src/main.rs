mod args;
mod client;
mod messages;
mod utils;

use args::Args;
use clap::Parser;
use client::Client;
use colored::Colorize;
use log::{debug, info, warn};
use messages::{
    EventHeatChanged, Heat, RequestListOpenHeats, RequestStartList, ResponseListOpenHeats, ResponseStartList,
};
use std::{io::Result, thread};

fn main() -> Result<()> {
    env_logger::builder().init();
    let args = Args::parse();

    let mut client = Client::new(args.host, args.port)?;
    let open_heats = read_open_heats(&mut client)?;
    debug!("Open heats: {:#?}", open_heats);

    info!("Receiving events ...");
    thread::spawn(move || loop {
        let received = client.receive_line().unwrap();
        if !received.is_empty() {
            debug!("Received: \"{}\"", utils::print_whitespaces(&received).bold());
            let event_opt = parse_event(&received);
            if let Some(mut event) = event_opt {
                read_start_list(&mut client, &mut event.heat).unwrap();
            }
        }
    })
    .join()
    .unwrap();

    Ok(())
} // the stream is closed here

fn parse_event(event: &str) -> Option<EventHeatChanged> {
    let parts: Vec<&str> = event.split_whitespace().collect();
    if parts.len() != 4 {
        warn!("Invalid event format: {}", event);
        return None;
    }

    let action = parts[0];
    let number = match parts[1].parse() {
        Ok(number) => number,
        Err(_) => {
            warn!("Invalid heat number: {}", parts[1]);
            return None;
        }
    };
    let id = match parts[2].parse() {
        Ok(id) => id,
        Err(_) => {
            warn!("Invalid heat ID: {}", parts[2]);
            return None;
        }
    };
    let status = match parts[3].parse() {
        Ok(status) => status,
        Err(_) => {
            warn!("Invalid status: {}", parts[3]);
            return None;
        }
    };

    match action {
        "!OPEN+" => {
            debug!("Opening heat: {}, id: {}, status: {}", number, id, status);
            Some(EventHeatChanged::new(Heat::new(id, number, status), true))
        }
        "!OPEN-" => {
            debug!("Closing heat: {}, id: {}, status: {}", number, id, status);
            Some(EventHeatChanged::new(Heat::new(id, number, status), false))
        }
        _ => {
            debug!("Unknown action: {}", action);
            None
        }
    }
}

fn read_open_heats(client: &mut Client) -> Result<Vec<Heat>> {
    client.write(&RequestListOpenHeats::new().to_string())?;
    let response = client.receive_all()?;
    let mut open_heats = ResponseListOpenHeats::parse(&response);

    for heat in open_heats.heats.iter_mut() {
        read_start_list(client, heat)?;
    }

    Ok(open_heats.heats)
}

fn read_start_list(client: &mut Client, heat: &mut Heat) -> Result<()> {
    client.write(&RequestStartList::new(heat.id).to_string())?;
    let response = client.receive_all()?;
    let start_list = ResponseStartList::parse(response);
    heat.boats = Some(start_list.boats);
    Ok(())
}
