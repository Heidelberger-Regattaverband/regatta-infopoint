use clap::{command, Parser};
use log::{info, LevelFilter};
use std::{
    io::{Read, Result, Write},
    net::TcpStream,
};

#[derive(Parser)]
#[command(name = "TimeKeeper")]
#[command(version = "0.1.0")]
#[command(about = "A Timekeeper for Aquarius", long_about = None)]
struct Cli {
    #[arg(long)]
    host: String,
    #[arg(long)]
    port: String,
}

fn main() -> Result<()> {
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        // .format_target(false)
        // .format_timestamp(None)
        .init();

    info!("Parsing command line arguments");
    let args = Cli::parse();

    info!("Connecting to: {}:{}", args.host, args.port);
    let mut stream = TcpStream::connect(format!("{}:{}", args.host, args.port))?;

    let cmd = "?OPEN\r\n";
    info!("Writing command: \"{}\"", cmd);
    let result = stream.write(cmd.as_bytes())?;
    info!("Written {} bytes", result);

    info!("Reading response ...");
    let mut buf = Vec::with_capacity(128);
    let read = stream.read(&mut buf)?;
    let response = String::from_utf8(buf).unwrap();
    info!("Read {} bytes: \"{}\"", read, response);

    Ok(())
} // the stream is closed here
