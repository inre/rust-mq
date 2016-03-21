use std::result;
use std::io;
use mqtt3::{ConnectReturnCode};
use mqtt3::Error as MqttError;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    AlreadyConnected,
    UnsupportedFeature,
    UnrecognizedPacket,
    ConnectionAbort,
    HandshakeFailed,
    ProtocolViolation,
    Disconnected,
    Timeout,
    ConnectionRefused(ConnectReturnCode),
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
