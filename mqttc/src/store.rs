use std::result;
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
