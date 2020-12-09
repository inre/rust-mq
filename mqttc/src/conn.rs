use mqtt3::{MqttRead, MqttWrite};
use std::io::{self, Read, Write};
use std::net::Shutdown;
use std::time::Duration;
use netopt::{NetworkStream};

pub struct Connection {
    stream: NetworkStream
}

impl Connection {
    pub fn new(stream: NetworkStream) -> io::Result<Connection> {
        Ok(Connection {
            stream: stream
        })
    }

    pub fn set_read_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        self.stream.set_read_timeout(dur)
    }

    pub fn terminate(&self) -> io::Result<()> {
        self.stream.shutdown(Shutdown::Both)
    }
}

impl Write for Connection {
    fn write(&mut self, msg: &[u8]) -> io::Result<usize> {
        self.stream.write(msg)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }
}

impl Read for Connection {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buf)
    }
}

impl MqttRead for Connection {}
impl MqttWrite for Connection {}
