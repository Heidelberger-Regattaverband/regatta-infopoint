use colored::Colorize;
use log::{debug, info};
use std::{
    io::{BufRead, BufReader, BufWriter, Result, Write},
    net::TcpStream,
};

/// A client to connect to the Aquarius server.
pub(crate) struct Client {
    /// A buffered reader to read from the server.
    reader: BufReader<TcpStream>,

    /// A buffered writer to write to the server.
    writer: BufWriter<TcpStream>,
}

impl Client {
    pub(crate) fn new(host: String, port: String) -> Result<Self> {
        info!("Connecting to {}:{}", host.bold(), port.bold());
        let stream = TcpStream::connect(format!("{}:{}", host, port)).unwrap();
        stream.set_nodelay(true).unwrap();
        let write_stream = stream.try_clone().unwrap();
        let reader = BufReader::new(stream);
        let writer = BufWriter::new(write_stream);
        Ok(Client { reader, writer })
    }

    pub(crate) fn write(&mut self, cmd: &str) -> Result<usize> {
        info!("Writing command: \"{}\"", cmd.bold());
        let count = self.writer.write(cmd.as_bytes())?;
        self.writer.flush()?;
        debug!("Written {} bytes", count.to_string().bold());
        Ok(count)
    }

    pub(crate) fn receive_line(&mut self) -> Result<String> {
        let mut line = String::new();
        let count = self.reader.read_line(&mut line)?;
        debug!("Received {} bytes", count.to_string().bold());
        Ok(line.trim_end().to_string())
    }

    pub(crate) fn receive_all(&mut self) -> Result<String> {
        let mut all = String::new();
        loop {
            let count = self.reader.read_line(&mut all)?;
            debug!("Received {} bytes", count);
            if count <= 2 {
                break;
            }
        }
        info!("Received message: \"{}\"", all.bold());
        Ok(all.trim_end().to_string())
    }
}
