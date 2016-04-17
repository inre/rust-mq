use std::result;
use std::io;
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
