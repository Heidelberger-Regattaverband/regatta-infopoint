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
use log::{debug, info};
use messages::{
    EventHeatChanged, Heat, RequestListOpenHeats, RequestStartList, ResponseListOpenHeats, ResponseStartList,
};
use std::thread;

fn main() -> Result<(), MessageErr> {
    env_logger::builder().init();
    let args = Args::parse();

    let mut client = Client::new(args.host, args.port).map_err(MessageErr::IoError)?;
    let open_heats = read_open_heats(&mut client)?;
    debug!("Open heats: {:#?}", open_heats);

    info!("Receiving events ...");
    thread::spawn(move || loop {
        let received = client.receive_line().unwrap();
        if !received.is_empty() {
            debug!("Received: \"{}\"", utils::print_whitespaces(&received).bold());
            let event_opt = EventHeatChanged::parse(&received);
            if let Some(mut event) = event_opt {
                read_start_list(&mut client, &mut event.heat).unwrap();
            }
        }
    })
    .join()
    .unwrap();

    Ok(())
} // the stream is closed here

fn read_open_heats(client: &mut Client) -> Result<Vec<Heat>, MessageErr> {
    client
        .write(&RequestListOpenHeats::new().to_string())
        .map_err(MessageErr::IoError)?;
    let response = client.receive_all().map_err(MessageErr::IoError)?;
    let mut open_heats = ResponseListOpenHeats::parse(&response).unwrap();

    for heat in open_heats.heats.iter_mut() {
        read_start_list(client, heat)?;
    }

    Ok(open_heats.heats)
}

fn read_start_list(client: &mut Client, heat: &mut Heat) -> Result<(), MessageErr> {
    client
        .write(&RequestStartList::new(heat.id).to_string())
        .map_err(MessageErr::IoError)?;
    let response = client.receive_all().map_err(MessageErr::IoError)?;
    let start_list = ResponseStartList::parse(response)?;
    heat.boats = Some(start_list.boats);
    Ok(())
}
