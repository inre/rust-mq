use super::{NetworkStream, NetworkConnector, Result, Error};
use super::tcp::TcpConnector;
use ::url::HostAndPort;
use ::openssl::{ssl};
use std::fmt;
use std::io::{self, Read, Write};
use std::net::{self, SocketAddr};
use std::time::Duration;
use openssl::ssl::{SslMethod};
use url::Host;

pub use openssl::ssl::Error as SslError;

pub struct SslStream<S>(ssl::SslStream<S>);

impl<S> SslStream<S> {
    pub fn from_ssl(ssl_stream: ssl::SslStream<S>) -> Self {
        SslStream(ssl_stream)
    }

    pub fn into_inner(self) -> ssl::SslStream<S> {
        self.0
    }
}

impl<S: Read + Write> Read for SslStream<S> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl<S: Read + Write> Write for SslStream<S> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

impl<S: fmt::Debug> fmt::Debug for SslStream<S> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<S: NetworkStream + 'static> NetworkStream for SslStream<S> {
    #[inline]
    fn peer_addr(&mut self) -> io::Result<SocketAddr> {
        self.0.get_mut().peer_addr()
    }

    #[inline]
    fn set_read_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        self.0.get_ref().set_read_timeout(dur)
    }

    #[inline]
    fn set_write_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        self.0.get_ref().set_write_timeout(dur)
    }

    #[inline]
    fn shutdown(&mut self, how: net::Shutdown) -> io::Result<()> {
        self.0.get_mut().shutdown(how)
    }
}

#[derive(Clone)]
pub struct SslConnector<C: NetworkConnector = TcpConnector> {
    base_connector: C,
    ssl_connector: ssl::SslConnector,
}

impl<C: NetworkConnector> SslConnector<C> {
    pub fn new_with_ssl_connector(base_connector: C, ssl_connector: ssl::SslConnector) -> Self {
        SslConnector {
            base_connector: base_connector,
            ssl_connector: ssl_connector,
        }
    }

    pub fn new(base_connector: C) -> Result<Self> {
        let connector = try!(ssl::SslConnector::builder(SslMethod::tls())).build();
        Ok(Self::new_with_ssl_connector(base_connector, connector))
    }
}

impl<C> NetworkConnector for SslConnector<C>
    where C: NetworkConnector + 'static
{
    type Stream = SslStream<C::Stream>;

    fn connect(&self, host_port: &HostAndPort) -> Result<Self::Stream> {
        let stream = try!(self.base_connector.connect(host_port));
        match host_port.host {
            Host::Domain(ref domain) => {
                // has the domain for certificate verification
                let ssl_stream = try!(self.ssl_connector.connect(&domain, stream));
                Ok(SslStream::from_ssl(ssl_stream))
            }
            Host::Ipv4(_) | Host::Ipv6(_) => Err(Error::DomainRequired),
        }
    }
}
