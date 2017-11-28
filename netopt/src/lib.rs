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

#[cfg(feature = "ssl")]
pub use ssl::{
    SslContext,
    SslStream,
    SslError
};

pub use conn::{
    Connection
};

#[cfg(not(feature = "ssl"))]
pub mod ssl {
    use mock::MockStream;
    use std::net::TcpStream;
    use std::io;
    use std::error::Error;
    use std::path::Path;

    pub type SslStream = MockStream;
    pub type SslError = Error + 'static;

    #[derive(Debug, Clone, Default)]
    pub struct SslContext;

    impl SslContext {
        pub fn new(_: SslContext) -> Self {
            panic!("ssl disabled");
        }

        pub fn with_cert_and_key<C, K, E>(_: C, _: K) -> Result<SslContext, E>
        where C: AsRef<Path>, K: AsRef<Path>, E: Error + 'static {
            panic!("ssl disabled");
        }

        pub fn accept(&self, _: TcpStream) -> Result<SslStream, io::Error> {
            panic!("ssl disabled");
        }

        pub fn connect(&self, _: TcpStream) -> Result<SslStream, io::Error> {
            panic!("ssl disabled");
        }
    }
}
