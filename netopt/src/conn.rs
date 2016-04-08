use mqtt3::{MqttRead, MqttWrite};
use std::io::{self, Read, Write};
use std::net::Shutdown;
use std::time::Duration;
use {NetworkStream, NetworkReader, NetworkWriter};

pub struct Connection {
    stream: NetworkStream,
    reader: NetworkReader,
    writer: NetworkWriter
}

impl Connection {
    pub fn new(stream: &NetworkStream) -> io::Result<Connection> {
        Ok(Connection {
            stream: try!(stream.try_clone()),
            reader: NetworkReader::new(try!(stream.try_clone())),
            writer: NetworkWriter::new(try!(stream.try_clone()))
        })
    }

    pub fn set_read_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        self.stream.set_read_timeout(dur)
    }

    pub fn terminate(&self) -> io::Result<()> {
        self.stream.shutdown(Shutdown::Both)
    }

    pub fn split(self) -> (NetworkReader, NetworkWriter) {
        (self.reader, self.writer)
    }
}

impl Write for Connection {
    fn write(&mut self, msg: &[u8]) -> io::Result<usize> {
        self.writer.write(msg)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

impl Read for Connection {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reader.read(buf)
    }
}

impl MqttRead for Connection {}
impl MqttWrite for Connection {}
