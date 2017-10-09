use std::{self, fmt};

#[cfg(feature = "ssl")]
use openssl::ssl::HandshakeError;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    #[cfg(feature = "ssl")]
    Openssl(::openssl::error::ErrorStack),
    #[cfg(feature = "ssl")]
    SslHandshake(::openssl::ssl::Error),
    DomainRequired,
    Other(Box<std::error::Error + Send>),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

#[cfg(feature = "ssl")]
impl From<::openssl::error::ErrorStack> for Error {
    fn from(err: ::openssl::error::ErrorStack) -> Self {
        Error::Openssl(err)
    }
}

#[cfg(feature = "ssl")]
impl<S> From<HandshakeError<S>> for Error {
    fn from(err: HandshakeError<S>) -> Self {
        match err {
            HandshakeError::SetupFailure(err) => err.into(),
            HandshakeError::Failure(mid_stream) |
            HandshakeError::Interrupted(mid_stream) => Error::SslHandshake(mid_stream.into_error()),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Error::*;
        use std::error::Error;
        match *self {
            Io(ref err) => write!(f, "{}: {}", self.description(), err),
            #[cfg(ssl)]
            Openssl(ref err) => write!(f, "{}: {}", self.description(), err),
            #[cfg(ssl)]
            SslHandshake(ref err) => write!(f, "{}: {}", self.description(), err),
            Other(ref err) => write!(f, "{}: {}", self.description(), err),
            _ => self.description().fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        use self::Error::*;
        match *self {
            Io(_) => "IO error",
            #[cfg(feature = "ssl")]
            Openssl(_) => "Openssl error",
            #[cfg(feature = "ssl")]
            SslHandshake(_) => "SSL handshake error",
            DomainRequired => {
                "A domain for a host is required in order to establish an SSL connection"
            }
            Other(_) => "Other error",
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        use self::Error::*;
        match *self {
            Io(ref err) => Some(err),
            #[cfg(feature = "ssl")]
            Openssl(ref err) => Some(err),
            #[cfg(feature = "ssl")]
            SslHandshake(ref err) => Some(err),
            Other(ref err) => Some(&**err),
            _ => None,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
