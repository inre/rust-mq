use std::result;
use std::io;
use std::fmt;
use std::error;
use mqtt3::{ConnectReturnCode, PacketIdentifier};
use mqtt3::Error as MqttError;
use store::Error as StorageError;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    AlreadyConnected,
    UnsupportedFeature,
    UnrecognizedPacket,
    ConnectionAbort,
    IncommingStorageAbsent,
    OutgoingStorageAbsent,
    HandshakeFailed,
    ProtocolViolation,
    Disconnected,
    Timeout,
    UnhandledPuback(PacketIdentifier),
    UnhandledPubrec(PacketIdentifier),
    UnhandledPubrel(PacketIdentifier),
    UnhandledPubcomp(PacketIdentifier),
    ConnectionRefused(ConnectReturnCode),
    Storage(StorageError),
    Mqtt(MqttError),
    Io(io::Error)
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<MqttError> for Error {
    fn from(err: MqttError) -> Error {
        match err {
            MqttError::Io(e) => Error::Io(e),
            _ => Error::Mqtt(err)
        }
    }
}

impl From<StorageError> for Error {
    fn from(err: StorageError) -> Error {
        Error::Storage(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            // Both underlying errors already impl `Display`, so we defer to
            // their implementations.
            Error::UnhandledPuback(PacketIdentifier(pi)) => fmt::write(f, format_args!("{:?}", pi)),
            Error::UnhandledPubrec(PacketIdentifier(pi)) => fmt::write(f, format_args!("{:?}", pi)),
            Error::UnhandledPubrel(PacketIdentifier(pi)) => fmt::write(f, format_args!("{:?}", pi)),
            Error::UnhandledPubcomp(PacketIdentifier(pi)) => fmt::write(f, format_args!("{:?}", pi)),
            Error::ConnectionRefused(crc) => fmt::write(f, format_args!("{:?}", crc)),
            Error::Storage(ref err) => write!(f, "Storage error: {:?}", err),
            Error::Mqtt(ref err) => write!(f, "MQTT error: {:?}", err),
            Error::Io(ref err) => write!(f, "IO error: {}", err),
            _ => fmt::write(f, format_args!("{:?}", *self)),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        // Underlying errors already impl `Error`, so we defer to their
        // implementations.
        match *self {
            Error::AlreadyConnected => "AlreadyConnected",
            Error::UnsupportedFeature => "UnsupportedFeature",
            Error::UnrecognizedPacket => "UnrecognizedPacket",
            Error::ConnectionAbort => "ConnectionAbort",
            Error::IncommingStorageAbsent => "IncommingStorageAbsent",
            Error::OutgoingStorageAbsent => "OutgoingStorageAbsent",
            Error::HandshakeFailed => "HandshakeFailed",
            Error::ProtocolViolation => "ProtocolViolation",
            Error::Disconnected => "Disconnected",
            Error::Timeout => "Timeout",
            Error::UnhandledPuback(_) => "UnhandledPuback",
            Error::UnhandledPubrec(_) => "UnhandledPubrec",
            Error::UnhandledPubrel(_) => "UnhandledPubrel",
            Error::UnhandledPubcomp(_) => "UnhandledPubcomp",
            Error::ConnectionRefused(_) => "ConnectionRefused",
            Error::Storage(ref err) => err.description(),
            Error::Mqtt(ref err) => err.description(),
            Error::Io(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            // N.B. These errors implicitly cast `err` from their concrete
            // types (either `&io::Error` or `&num::ParseIntError`)
            // to a trait object `&Error`. This works because both error types
            // implement `Error`.
            Error::Storage(ref err) => Some(err),
            Error::Mqtt(ref err) => Some(err),
            Error::Io(ref err) => Some(err),
            _ => None,
        }
    }
}
