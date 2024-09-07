use log::{debug, info};
use std::{
    io::{BufRead, BufReader, BufWriter, Error, ErrorKind, Result, Write},
    net::TcpStream,
};

pub(crate) struct Client {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
}

impl Client {
    pub(crate) fn new(host: String, port: String) -> Self {
        info!("Connecting to {}:{}", host, port);
        let stream = TcpStream::connect(format!("{}:{}", host, port)).unwrap();
        stream.set_nodelay(true).unwrap();
        let wstream = stream.try_clone().unwrap();
        let reader = BufReader::new(stream);
        let writer = BufWriter::new(wstream);
        Client { reader, writer }
    }

    pub(crate) fn write(&mut self, cmd: &str) -> Result<()> {
        info!("Writing command: \"{}\"", cmd);
        let count = self.writer.write(cmd.as_bytes())?;
        self.writer.flush()?;
        debug!("Written {} bytes", count);
        Ok(())
    }

    pub(crate) fn receive(&mut self) -> Result<String> {
        let mut line = String::with_capacity(512);
        let count = self.reader.read_line(&mut line)?;
        debug!("Read {} bytes", count);
        if count == 0 {
            Err(Error::new(ErrorKind::Other, "No data received"))
        } else {
            Ok(line)
        }
    }
}
