use log::{debug, info};
use std::{
    io::{BufRead, BufReader, BufWriter, Error, ErrorKind, Result, Write},
    net::TcpStream,
};

/// A client to connect to the Aquarius server.
pub(crate) struct Client {
    /// A reader to read from the server.
    reader: BufReader<TcpStream>,

    /// A writer to write to the server.
    writer: BufWriter<TcpStream>,
}

impl Client {
    pub(crate) fn new(host: String, port: String) -> Result<Self> {
        info!("Connecting to {}:{}", host, port);
        let stream = TcpStream::connect(format!("{}:{}", host, port)).unwrap();
        stream.set_nodelay(true).unwrap();
        let wstream = stream.try_clone().unwrap();
        let reader = BufReader::new(stream);
        let writer = BufWriter::new(wstream);
        Ok(Client { reader, writer })
    }

    pub(crate) fn write(&mut self, cmd: &str) -> Result<()> {
        info!("Writing command: \"{}\"", cmd);
        let count = self.writer.write(cmd.as_bytes())?;
        self.writer.flush()?;
        debug!("{} bytes written", count);
        Ok(())
    }

    pub(crate) fn receive(&mut self) -> Result<String> {
        let mut line = String::new();
        let count = self.reader.read_line(&mut line)?;
        debug!("{} bytes received", count);
        if count == 0 {
            Err(Error::new(ErrorKind::Other, "No data received"))
        } else {
            Ok(line.trim_end().to_string())
        }
    }
}
