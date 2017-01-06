use super::*;
use std::fmt;
use std::io::{self, Read, Write};
use std::net::{self, ToSocketAddrs, SocketAddr};
use std::time::Duration;

pub struct TcpStream(net::TcpStream);

impl TcpStream {
    pub fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let tcp_stream = try!(net::TcpStream::connect(addr));
        Ok(tcp_stream.into())
    }

    pub fn into_inner(self) -> net::TcpStream {
        self.0
    }
}

impl From<net::TcpStream> for TcpStream {
    fn from(tcp_stream: net::TcpStream) -> Self {
        TcpStream(tcp_stream)
    }
}

impl Read for TcpStream {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl Write for TcpStream {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

impl fmt::Debug for TcpStream {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Clone for TcpStream {
    fn clone(&self) -> Self {
        self.0.try_clone().unwrap().into()
    }
}

impl NetworkStream for TcpStream {
    #[inline]
    fn peer_addr(&mut self) -> io::Result<SocketAddr> {
        self.0.peer_addr()
    }

    #[inline]
    fn set_read_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        self.0.set_read_timeout(dur)
    }

    #[inline]
    fn set_write_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        self.0.set_write_timeout(dur)
    }

    #[inline]
    fn shutdown(&mut self, how: net::Shutdown) -> io::Result<()> {
        match self.0.shutdown(how) {
            Ok(_) => Ok(()),
            // see https://github.com/hyperium/hyper/issues/508
            Err(ref e) if e.kind() == io::ErrorKind::NotConnected => Ok(()),
            err => err,
        }
    }
}

pub struct TcpListener(net::TcpListener);

impl TcpListener {
    pub fn into_inner(self) -> net::TcpListener {
        self.0
    }

    pub fn bind<A: ToSocketAddrs>(addr: A) -> Result<Self> {
        let stream = try!(net::TcpListener::bind(addr));
        Ok(stream.into())
    }
}

impl From<net::TcpListener> for TcpListener {
    fn from(tcp_listener: net::TcpListener) -> Self {
        TcpListener(tcp_listener)
    }
}

impl Clone for TcpListener {
    fn clone(&self) -> Self {
        self.0.try_clone().unwrap().into()
    }
}

impl NetworkListener for TcpListener {
    type Stream = TcpStream;

    fn accept(&mut self) -> ::Result<(Self::Stream, SocketAddr)> {
        let (stream, addr) = try!(self.0.accept());
        Ok((stream.into(), addr))
    }

    fn local_addr(&mut self) -> io::Result<SocketAddr> {
        let local_addr = try!(self.0.local_addr());
        Ok(local_addr)
    }
}

/// A connector that will produce HttpStreams.
#[derive(Debug, Clone, Default)]
pub struct TcpConnector;

impl TcpConnector {
    pub fn new() -> Self {
        TcpConnector
    }
}

impl NetworkConnector for TcpConnector {
    type Stream = TcpStream;

    fn connect(&self, host_port: &HostAndPort) -> Result<Self::Stream> {
        Ok(try!(TcpStream::connect(host_port)))
    }
}

#[cfg(test)]
mod test {
    use std::net::Shutdown;
    use std::io::{Read, Write};
    use std::thread;
    use super::{TcpListener, TcpConnector, NetworkStream, NetworkConnector, NetworkListener};
    use url::{HostAndPort, Host};

    #[test]
    fn tcp_server_client_test() {
        let addr = HostAndPort { host: Host::parse("localhost").unwrap(), port: 8334 };
        let mut listener = TcpListener::bind(&addr).unwrap();

        thread::spawn(move || {
            let mut client = TcpConnector::new().connect(&addr).unwrap();
            client.write(&[0, 1, 2, 3, 4, 5]).unwrap();
            client.flush().unwrap();
            client.shutdown(Shutdown::Both).unwrap();
        });

        let (mut stream, _) = listener.accept().unwrap();
        let mut req = Vec::new();
        stream.read_to_end(&mut req).unwrap();
        assert_eq!(req, vec![0, 1, 2, 3, 4, 5]);
    }
}
