extern crate mqtt3;
#[cfg(feature = "ssl")]
extern crate openssl;

#[cfg(feature = "ssl")]
mod ssl;
mod tcp;
pub mod mock;
pub mod conn;

pub use tcp::{
    NetworkOptions,
    NetworkListener,
    NetworkStream,
    NetworkWriter,
    NetworkReader
};

pub use ssl::{
    SslContext,
    SslStream,
    SslError
};

#[cfg(not(feature = "ssl"))]
pub mod ssl {
    use mock::MockStream;
    use std::net::TcpStream;
    use std::io;
    use std::error::Error;

    pub type SslStream = MockStream;
    pub type SslError = Error;

    #[derive(Debug, Clone)]
    pub struct SslContext;

    impl SslContext {
        pub fn accept(&self, _: TcpStream) -> Result<SslStream, io::Error> {
            panic!("ssl disabled");
        }
    }
}
