mod args;
mod client;
mod messages;
mod utils;

use args::Args;
use clap::Parser;
use client::Client;
use colored::Colorize;
use log::{debug, info};
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
