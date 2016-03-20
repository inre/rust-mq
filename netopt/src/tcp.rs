use std::net::{TcpListener, TcpStream, SocketAddr, ToSocketAddrs, Shutdown, SocketAddrV4, Ipv4Addr};
use std::io::{self, Read, Write, BufReader, BufWriter};
use std::time::Duration;
use std::os::unix::io::AsRawFd;
use std::fmt;

use mqtt3::{MqttRead, MqttWrite};
use ssl::{SslContext, SslStream};
use mock::MockStream;
use openssl::ssl::IntoSsl;

use NetworkStream::{
    Tcp,
    Ssl,
    Mock
};

pub struct NetworkOptions {
    ssl: Option<SslContext>,
    mock: Option<NetworkStream>
}

impl NetworkOptions {
    pub fn new() -> NetworkOptions {
        NetworkOptions {
            ssl: None,
            mock: None
        }
    }

    pub fn attach(&mut self, stream: NetworkStream) -> &mut NetworkOptions {
        self.mock = Some(stream); self
    }

    pub fn tls(&mut self, ssl: SslContext) -> &mut NetworkOptions {
        self.ssl = Some(ssl); self
    }

    pub fn bind<A: ToSocketAddrs>(&self, addr: A) -> io::Result<NetworkListener> {
        Ok(NetworkListener {
            tcp: try!(TcpListener::bind(addr)),
            ssl: match self.ssl {
                Some(ref ssl) => Some(ssl.clone()),
                None => None
            }
        })
    }

    pub fn connect<A: ToSocketAddrs>(&self, addr: A) -> io::Result<NetworkStream> {
        if let Some(ref stream) = self.mock {
            return Ok(try!(stream.try_clone()));
        };

        let stream = try!(TcpStream::connect(addr));
        match self.ssl {
            Some(ref ssl) => Ok(NetworkStream::Ssl(try!(ssl.connect(stream)))),
            None => Ok(NetworkStream::Tcp(stream))
        }
    }
}

pub struct NetworkListener {
    tcp: TcpListener,
    ssl: Option<SslContext>,
}

impl NetworkListener {
    pub fn accept(&mut self) -> io::Result<(NetworkStream, SocketAddr)> {
        let (stream, addr) = try!(self.tcp.accept());
        match self.ssl {
            Some(ref ssl) => {
                match ssl.accept(stream) {
                    Ok(ssl_stream) => Ok((NetworkStream::Ssl(ssl_stream), addr)),
                    Err(e) => Err(e)
                }
            },
            None => Ok((NetworkStream::Tcp(stream), addr))
        }
    }
}

pub enum NetworkStream {
    Tcp(TcpStream),
    Ssl(SslStream),
    Mock(MockStream)
}

impl NetworkStream {
    pub fn try_clone(&self) -> io::Result<NetworkStream> {
        match *self {
            Tcp(ref s) => Ok(Tcp(try!(s.try_clone()))),
            Ssl(ref s) => Ok(Ssl(try!(s.try_clone()))),
            Mock(ref s) => Ok(Mock(s.clone()))
        }
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        match *self {
            Tcp(ref s) => s.peer_addr(),
            Ssl(ref s) => s.get_ref().peer_addr(),
            Mock(ref s) => Ok(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127,0,0,1), 80)))
        }
    }

    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        match *self {
            Tcp(ref s) => s.shutdown(how),
            Ssl(ref s) => s.get_ref().shutdown(how),
            Mock(_) => Ok(())
        }
    }

    pub fn set_read_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        match *self {
            Tcp(ref s) => s.set_read_timeout(dur),
            Ssl(ref s) => s.get_ref().set_read_timeout(dur),
            Mock(_) => Ok(())
        }
    }

    pub fn set_write_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        match *self {
            Tcp(ref s) => s.set_write_timeout(dur),
            Ssl(ref s) => s.get_ref().set_write_timeout(dur),
            Mock(ref s) => Ok(())
        }
    }
}

impl Read for NetworkStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match *self {
            Tcp(ref mut s) => s.read(buf),
            Ssl(ref mut s) => s.read(buf),
            Mock(ref mut s) => s.read(buf)
        }
    }
}

impl Write for NetworkStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match *self {
            Tcp(ref mut s) => s.write(buf),
            Ssl(ref mut s) => s.write(buf),
            Mock(ref mut s) => s.write(buf)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match *self {
            Tcp(ref mut s) => s.flush(),
            Ssl(ref mut s) => s.flush(),
            Mock(ref mut s) => s.flush()
        }
    }
}

pub type NetworkReader = BufReader<NetworkStream>;
pub type NetworkWriter = BufWriter<NetworkStream>;

impl MqttRead for NetworkStream {}
impl MqttWrite for NetworkStream {}

#[cfg(test)]
mod test {
    use std::net::{TcpListener, TcpStream, Shutdown};
    use std::io::{self, Read, Write};
    use std::thread;
    use super::{NetworkOptions, NetworkListener, NetworkStream};
    use mock::MockStream;

    #[test]
    fn tcp_server_client_test() {
        let mut listener = NetworkOptions::new().bind("127.0.0.1:8432").unwrap();

        thread::spawn(|| {
            let mut client = NetworkOptions::new().connect("127.0.0.1:8432").unwrap();
            client.write(&[0, 1, 2, 3, 4, 5]).unwrap();
            client.flush().unwrap();
            client.shutdown(Shutdown::Both).unwrap();
        });

        let (mut stream, addr) = listener.accept().unwrap();
        let mut req = Vec::new();
        stream.read_to_end(&mut req).unwrap();
        assert_eq!(req, vec![0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn tcp_attach_test() {
        let stream = NetworkStream::Mock(MockStream::with_vec(vec![0xFE, 0xFD]));
        let mut options = NetworkOptions::new();
        options.attach(stream);
        let mut client = options.connect("127.0.0.1:80").unwrap();
        let mut buf = Vec::new();
        client.read_to_end(&mut buf).unwrap();
        assert_eq!(buf, vec![0xFE, 0xFD]);
    }
}
