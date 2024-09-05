use clap::{command, Parser};
use log::{info, LevelFilter};
use std::{
    io::{BufRead, BufReader, BufWriter, Result, Write},
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
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
}

impl Client {
    fn new() -> Self {
        info!("Parsing command line arguments");
        let args = Args::parse();

        info!("Connecting to {}:{}", args.host, args.port);
        let stream = TcpStream::connect(format!("{}:{}", args.host, args.port)).unwrap();
        stream.set_nodelay(true).unwrap();
        let wstream = stream.try_clone().unwrap();
        let reader = BufReader::new(stream);
        let writer = BufWriter::new(wstream);
        Client { reader, writer }
    }

    fn send_cmd(&mut self, cmd: &str) -> Result<()> {
        info!("Writing command: \"{}\"", cmd);
        let result = self.writer.write(cmd.as_bytes())?;
        self.writer.flush()?;
        info!("Written {} bytes", result);
        Ok(())
    }

    fn receive(&mut self) -> Result<String> {
        let mut line = String::with_capacity(512);
        self.reader.read_line(&mut line)?; //read_linto_string(&mut line)?;
        Ok(line)
    }
}

fn main() -> Result<()> {
    env_logger::builder().filter_level(LevelFilter::Info).init();

    let mut client = Client::new();
    client.send_cmd("?STARTLIST nr=50\r\n")?;

    info!("Receiving ...");
    loop {
        let received = client.receive()?;
        if !received.is_empty() {
            info!("Received:\"{}\"", received);
        }
    }
    //Ok(())
} // the stream is closed here
