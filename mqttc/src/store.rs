use std::result;
use std::error;
use std::fmt;
use mqtt3::{Message, PacketIdentifier};

pub type Result<T> = result::Result<T, Error>;

pub trait Store {
    fn put(&mut self, message: Box<Message>) -> Result<()>;
    fn get(&mut self, pid: PacketIdentifier) -> Result<Box<Message>>;
    fn delete(&mut self, pid: PacketIdentifier) -> Result<()>;
//    fn iter() -> Iterator<Message>;
}

#[derive(Debug)]
pub enum Error {
    NotFound(PacketIdentifier),
    Unavailable(PacketIdentifier)
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::NotFound(PacketIdentifier(packet_identifier)) =>
                fmt::write(f, format_args!("Packet {} not found", packet_identifier)),
            Error::Unavailable(PacketIdentifier(packet_identifier)) =>
                fmt::write(f, format_args!("Packet {} unavailable", packet_identifier)),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::NotFound(PacketIdentifier(_)) =>  "Packet not found",
            Error::Unavailable(PacketIdentifier(_)) => "Packet unavailable",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}
