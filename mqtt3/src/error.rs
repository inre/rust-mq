use std::result;
use std::io;
use std::fmt;
use std::error;
use std::string::FromUtf8Error;
use byteorder;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    IncorrectPacketFormat,
    InvalidTopicPath,
    UnsupportedProtocolName,
    UnsupportedProtocolVersion,
    UnsupportedQualityOfService,
    UnsupportedPacketType,
    UnsupportedConnectReturnCode,
    PayloadSizeIncorrect,
    PayloadTooLong,
    PayloadRequired,
    TopicNameMustNotContainNonUtf8,
    TopicNameMustNotContainWildcard,
    MalformedRemainingLength,
    UnexpectedEof,
    Io(io::Error)
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(_: FromUtf8Error) -> Error {
        Error::TopicNameMustNotContainNonUtf8
    }
}

impl From<byteorder::Error> for Error {
    fn from(err: byteorder::Error) -> Error {
        match err {
            byteorder::Error::UnexpectedEOF => Error::UnexpectedEof,
            byteorder::Error::Io(err) => Error::Io(err)
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::write(f, format_args!("{:?}", *self))
    }
}

impl error::Error for Error {
    fn description<'a>(&'a self) -> &'a str {
        match *self {
            Error::IncorrectPacketFormat => "Incorrect Packet Format",
            Error::InvalidTopicPath => "Invalid Topic Path",
            Error::UnsupportedProtocolName => "Unsupported Protocol Name",
            Error::UnsupportedProtocolVersion => "Unsupported Protocol Version",
            Error::UnsupportedQualityOfService => "Unsupported Quality Of Service",
            Error::UnsupportedPacketType => "Unsupported Packet Type",
            Error::UnsupportedConnectReturnCode => "Unsupported Connect Return Code",
            Error::PayloadSizeIncorrect => "Payload Size Incorrect",
            Error::PayloadTooLong => "Payload Too Long",
            Error::PayloadRequired => "Payload Required",
            Error::TopicNameMustNotContainNonUtf8 => "Topic Name Must Not Contain Non Utf 8",
            Error::TopicNameMustNotContainWildcard => "Topic Name Must Not Contain Wildcard",
            Error::MalformedRemainingLength => "Malformed Remaining Length",
            Error::UnexpectedEof => "Unexpected Eof",
            Error::Io(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Io(ref err) => Some(err),
            _ => None,
        }
    }
}
