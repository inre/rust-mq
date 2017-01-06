use std::{error, fmt, result};
use std::collections::BTreeMap;
use mqtt3::{Message, PacketIdentifier};

pub type Result<T> = result::Result<T, Error>;

pub trait Store {
    fn put(&mut self, message: Message) -> Result<()>;
    fn get(&mut self, pid: PacketIdentifier) -> Result<Message>;
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

pub struct MemoryStorage(BTreeMap<PacketIdentifier, Message>);

impl MemoryStorage {
    pub fn new() -> Box<MemoryStorage> {
        Box::new(MemoryStorage(BTreeMap::new()))
    }
}

impl Store for MemoryStorage {
    fn put(&mut self, message: Message) -> Result<()> {
        self.0.insert(message.pid.unwrap(), message);
        Ok(())
    }

    fn get(&mut self, pid: PacketIdentifier) -> Result<Message> {
        match self.0.get(&pid) {
            Some(m) => Ok(m.clone()),
            None => Err(Error::NotFound(pid))
        }
    }

    fn delete(&mut self, pid: PacketIdentifier) -> Result<()> {
        self.0.remove(&pid);
        Ok(())
    }
}
