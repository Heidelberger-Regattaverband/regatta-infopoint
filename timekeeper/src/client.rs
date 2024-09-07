use crate::Args;
use clap::Parser;
use log::info;
use std::{
    io::{BufRead, BufReader, BufWriter, Error, ErrorKind, Result, Write},
    net::TcpStream,
};

pub(crate) struct Client {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
}

impl Client {
    pub(crate) fn new() -> Self {
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

    pub(crate) fn send_cmd(&mut self, cmd: &str) -> Result<()> {
        info!("Writing command: \"{}\"", cmd);
        let result = self.writer.write(cmd.as_bytes())?;
        self.writer.flush()?;
        info!("Written {} bytes", result);
        Ok(())
    }

    pub(crate) fn receive(&mut self) -> Result<String> {
        let mut line = String::with_capacity(512);
        let count = self.reader.read_line(&mut line)?;
        info!("Read {} bytes", count);
        if count == 0 {
            Err(Error::new(ErrorKind::Other, "No data received"))
        } else {
            Ok(line)
        }
    }
}
