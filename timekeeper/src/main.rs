mod args;
mod client;
mod error;
mod messages;
mod utils;

use args::Args;
use clap::Parser;
use client::{Client, HeatEventReceiver};
use error::MessageErr;
use log::{debug, info};
use std::sync::{Arc, Mutex};

struct EventReceiver {}

impl HeatEventReceiver for EventReceiver {
    fn on_event(&mut self, event: &messages::EventHeatChanged) {
        info!("Received event: {:?}", &event);
    }
}

fn main() -> Result<(), MessageErr> {
    env_logger::builder().init();
    let args = Args::parse();

    let mut client = Client::connect(args.host, args.port, args.timeout).map_err(MessageErr::IoError)?;
    let open_heats = client.read_open_heats()?;
    debug!("Open heats: {:#?}", open_heats);

    let receiver = Arc::new(Mutex::new(EventReceiver {}));

    client
        .start_receiving_events(receiver)
        .map_err(MessageErr::IoError)?
        .join()
        .unwrap();

    Ok(())
} // the stream is closed here
