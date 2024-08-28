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
struct Args {
    #[arg(long)]
    host: String,
    #[arg(long)]
    port: String,
}

struct Client {
    stream: TcpStream,
}

impl Client {
    fn new() -> Self {
        info!("Parsing command line arguments");
        let args = Args::parse();

        info!("Connecting to {}:{}", args.host, args.port);
        let stream = TcpStream::connect(format!("{}:{}", args.host, args.port)).unwrap();
        stream.set_nodelay(true).unwrap();
        Client { stream }
    }

    fn send_cmd(&mut self, cmd: &str) -> Result<()> {
        info!("Writing command: \"{}\"", cmd);
        let result = self.stream.write(cmd.as_bytes())?;
        self.stream.flush()?;
        info!("Written {} bytes", result);
        Ok(())
    }

    fn receive(&mut self) -> Result<String> {
        info!("Receiving ...");
        let mut line = String::with_capacity(512);
        self.stream.read_to_string(&mut line)?;
        Ok(line)
    }
}

fn main() -> Result<()> {
    env_logger::builder().filter_level(LevelFilter::Info).init();

    let mut client = Client::new();
    client.send_cmd("?STARTLIST nr=50\r\n")?;

    let received = client.receive()?;
    info!("Received:\"{}\"", received);

    Ok(())
} // the stream is closed here
