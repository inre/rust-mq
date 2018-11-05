extern crate url;

#[cfg(feature = "ssl")]
pub mod ssl;
pub mod tcp;
pub mod mock;
pub mod error;

use std::time::Duration;
use std::io::{self, Read, Write};
use std::net::{SocketAddr, Shutdown};

pub use self::error::{Error, Result};

pub use url::HostAndPort;

/// An abstraction over network streams
///
/// Works with raw TCP, SSL, or even just a mock for testing
pub trait NetworkStream: Read + Write + Send {
    /// Get the remote address of the underlying connection.
    fn peer_addr(&mut self) -> io::Result<SocketAddr>;

    /// Set the maximum time to wait for a read to complete.
    fn set_read_timeout(&self, dur: Option<Duration>) -> io::Result<()>;

    /// Set the maximum time to wait for a write to complete.
    fn set_write_timeout(&self, dur: Option<Duration>) -> io::Result<()>;

    /// This will be called when Stream should no longer be kept alive.
    #[inline]
    fn shutdown(&mut self, _how: Shutdown) -> io::Result<()> {
        Ok(())
    }
}

impl NetworkStream for Box<NetworkStream> {
    #[inline]
    fn peer_addr(&mut self) -> io::Result<SocketAddr> {
        (**self).peer_addr()
    }

    #[inline]
    fn set_read_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        (**self).set_read_timeout(dur)
    }

    #[inline]
    fn set_write_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        (**self).set_write_timeout(dur)
    }

    #[inline]
    fn shutdown(&mut self, _how: Shutdown) -> io::Result<()> {
        (**self).shutdown(_how)
    }
}


/// An abstraction to listen for connections on a certain port.
pub trait NetworkListener: Clone {
    /// The stream produced for each connection.
    type Stream: NetworkStream + Send;

    /// Returns an iterator of streams.
    fn accept(&mut self) -> ::Result<(Self::Stream, SocketAddr)>;

    /// Get the address this Listener ended up listening on.
    fn local_addr(&mut self) -> io::Result<SocketAddr>;

    /// Returns an iterator over incoming connections.
    fn incoming(&mut self) -> NetworkConnections<Self> {
        NetworkConnections(self)
    }
}

/// An iterator wrapper over a `NetworkAcceptor`.
pub struct NetworkConnections<'a, N: NetworkListener + 'a>(&'a mut N);

impl<'a, N: NetworkListener + 'a> Iterator for NetworkConnections<'a, N> {
    type Item = ::Result<(N::Stream, SocketAddr)>;
    fn next(&mut self) -> Option<::Result<(N::Stream, SocketAddr)>> {
        Some(self.0.accept())
    }
}

/// A connector creates a NetworkStream.
pub trait NetworkConnector: Clone {
    /// Type of `Stream` to create
    type Stream: NetworkStream + Send;

    /// Connect to a remote address.
    fn connect(&self, host_port: &HostAndPort) -> Result<Self::Stream>;
}

trait BoxConnector {
    fn connect_box(&self, host_port: &HostAndPort) -> Result<Box<NetworkStream>>;
    fn clone_box(&self) -> Box<BoxConnector>;
}

impl<S, C> BoxConnector for C
    where S: NetworkStream + 'static,
          C: NetworkConnector<Stream = S> + 'static
{
    fn connect_box(&self, host_port: &HostAndPort) -> Result<Box<NetworkStream>> {
        let stream = try!(self.connect(host_port));
        Ok(Box::new(stream))
    }

    fn clone_box(&self) -> Box<BoxConnector> {
        Box::new(self.clone())
    }
}

/// A `NetworkConnector` that is `Box`ed and returns `Box`ed `NetworkStream`s. The underlying
/// implementation can vary.
pub struct BoxedConnector(Box<BoxConnector>);

impl BoxedConnector {
    /// Creates a new `BoxedConnector` containing the specified `connector`
    #[inline]
    pub fn new<C>(connector: C) -> Self
        where C: NetworkConnector + Sized + 'static
    {
        BoxedConnector(Box::new(connector))
    }
}

impl<C> From<Box<C>> for BoxedConnector
    where C: NetworkConnector + Sized + 'static
{
    fn from(connector: Box<C>) -> Self {
        BoxedConnector(connector)
    }
}

impl Clone for BoxedConnector {
    fn clone(&self) -> Self {
        BoxedConnector(self.0.clone_box())
    }
}

impl NetworkConnector for BoxedConnector {
    type Stream = Box<NetworkStream>;

    fn connect(&self, host_port: &HostAndPort) -> Result<Self::Stream> {
        self.0.connect_box(host_port)
    }
}

pub use self::tcp::{TcpStream, TcpListener, TcpConnector};

#[cfg(feature = "ssl")]
pub use self::ssl::{SslStream, SslError, SslConnector};
