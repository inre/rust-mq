use super::*;
use std::io::{self, Read, Write};
use std::{cmp, mem};
use std::net::ToSocketAddrs;

#[derive(Clone, Debug)]
pub struct MockStream {
    peer_addr: SocketAddr,
    read_data: Vec<u8>,
    write_data: Vec<u8>,
    read_idx: usize,
}

impl MockStream {
    pub fn new<A>(peer_addr: A) -> MockStream
        where A: ToSocketAddrs {
        Self::with_read_data(peer_addr, None)
    }

    pub fn with_read_data<A, I>(peer_addr: A, read_data: I) -> MockStream
        where A: ToSocketAddrs, I: IntoIterator<Item = u8>
    {
        use std::iter::FromIterator;
        MockStream {
            peer_addr: peer_addr.to_socket_addrs().unwrap().next().unwrap(),
            read_data: Vec::from_iter(read_data),
            write_data: Vec::new(),
            read_idx: 0,
        }
    }

    pub fn drain_write_data(&mut self) -> Vec<u8> {
        self.write_data
            .drain(..)
            .collect()
    }

    pub fn replace_read_data<I>(&mut self, read_data: I) -> Vec<u8>
        where I: IntoIterator<Item = u8>
    {
        use std::iter::FromIterator;
        self.read_idx = 0;
        mem::replace(&mut self.read_data, Vec::from_iter(read_data))
    }

    pub fn swap_data(&mut self) {
        self.read_idx = 0;
        mem::swap(&mut self.read_data, &mut self.write_data);
    }
}

impl Write for MockStream {
    fn write(&mut self, msg: &[u8]) -> io::Result<usize> {
        self.write_data.extend_from_slice(msg);
        Ok(msg.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Read for MockStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let left_len: Option<usize> = self.read_data.len().checked_sub(self.read_idx);
        if let Some(left_len) = left_len {
            let len = cmp::min(left_len, buf.len());
            let start = self.read_idx;
            let end = start + len;
            self.read_idx += len;
            (&mut buf[..len]).copy_from_slice(&self.read_data[start..end]);
            Ok(len)
        } else {
            Ok(0)
        }
    }
}

impl NetworkStream for MockStream {
    #[inline]
    fn peer_addr(&mut self) -> io::Result<SocketAddr> {
        Ok(self.peer_addr)
    }

    #[inline]
    fn set_read_timeout(&self, _dur: Option<Duration>) -> io::Result<()> {
        Ok(())
    }

    #[inline]
    fn set_write_timeout(&self, _dur: Option<Duration>) -> io::Result<()> {
        Ok(())
    }

    #[inline]
    fn shutdown(&mut self, _how: Shutdown) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct MockConnector {
    read_data: Vec<u8>,
}

impl MockConnector {
    pub fn new() -> Self {
        Self::with_read_data(None)
    }

    pub fn with_read_data<I: IntoIterator<Item=u8>>(read_data: I) -> Self {
        use std::iter::FromIterator;
        MockConnector {
            read_data: Vec::from_iter(read_data),
        }
    }
}

impl NetworkConnector for MockConnector {
    type Stream = MockStream;

    fn connect(&self, host_port: &HostAndPort) -> Result<Self::Stream> {
        Ok(MockStream::with_read_data(host_port, self.read_data.iter().cloned()))
    }
}

#[cfg(test)]
mod test {
    use std::io::{Read, Write};
    use super::{MockConnector, MockStream, NetworkStream, NetworkConnector};
    use std::net::ToSocketAddrs;
    use url::{Host, HostAndPort};

    #[test]
    fn mock_connect_test() {
        let data = vec![0xFE, 0xFD];
        let connector = MockConnector::with_read_data(data);
        let addr = HostAndPort { host: Host::parse("172.0.0.1").unwrap(), port: 8432 };
        let mut client = connector.connect(&addr).unwrap();
        let mut buf = Vec::new();
        client.read_to_end(&mut buf).unwrap();
        assert_eq!(buf, vec![0xFE, 0xFD]);
    }

    #[test]
    fn write_take_test() {
        let mut mock = MockStream::new("localhost:1883");
        // write to mock stream
        mock.write(&[1, 2, 3]).unwrap();
        assert_eq!(mock.drain_write_data(), vec![1, 2, 3]);
    }

    #[test]
    fn read_with_vec_test() {
        let addr_str = "localhost:1883";
        let mut mock = MockStream::with_read_data(addr_str, vec![4, 5]);
        let mut vec = Vec::new();
        let addr = addr_str.to_socket_addrs().unwrap().next().unwrap();
        assert_eq!(mock.peer_addr().unwrap(), addr);
        mock.read_to_end(&mut vec).unwrap();
        assert_eq!(vec, vec![4, 5]);
    }

    #[test]
    fn clone_test() {
        let mut mock = MockStream::new("localhost:1883");
        mock.write(&[6, 7]).unwrap();
        let mut cloned = mock.clone();
        assert_eq!(cloned.drain_write_data(), vec![6, 7]);
    }

    #[test]
    fn swap_test() {
        let mut mock = MockStream::new("localhost:1883");
        let mut vec = Vec::new();
        mock.write(&[8, 9, 10]).unwrap();
        mock.swap_data();
        mock.read_to_end(&mut vec).unwrap();
        assert_eq!(vec, vec![8, 9, 10]);
    }
}
