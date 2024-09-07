mod client;

use clap::{command, Parser};
use client::Client;
use log::{info, LevelFilter};
use std::{io::Result, thread};

#[derive(Parser)]
#[command(name = "TimeKeeper")]
#[command(version = "0.1.0")]
#[command(about = "A Timekeeper for Aquarius", long_about = None)]
struct Args {
    #[arg(long)]
    host: String,
    #[arg(long)]
    port: String,
}

fn main() -> Result<()> {
    env_logger::builder().filter_level(LevelFilter::Info).init();
    let args = Args::parse();

    let mut client = Client::new(args.host, args.port);
    client.write("?OPEN\n")?;

    info!("Receiving ...");
    thread::spawn(move || loop {
        let received = client.receive().unwrap();
        if !received.is_empty() {
            info!("Received:\"{}\"", received);
        }
    })
    .join()
    .unwrap();

    Ok(())
} // the stream is closed here
