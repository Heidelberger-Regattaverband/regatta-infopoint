mod args;
mod client;
mod messages;

use args::Args;
use clap::Parser;
use client::Client;
use colored::Colorize;
use log::{info, LevelFilter};
use messages::{Heat, RequestListOpenHeats, RequestStartList, ResponseListOpenHeats, ResponseStartList};
use std::{io::Result, thread};

fn main() -> Result<()> {
    env_logger::builder().init();
    let args = Args::parse();

    let mut client = Client::new(args.host, args.port)?;
    let open_heats = get_open_heats(&mut client)?;
    info!("Open heats: {:#?}", open_heats);

    info!("Receiving events ...");
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
