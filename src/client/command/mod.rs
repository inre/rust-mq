pub mod publish;
pub mod subscribe;

pub use client::command::publish::PublishCommand;
pub use client::command::subscribe::SubscribeCommand;

use std::collections::BTreeMap;
use mqtt3::{PacketIdentifier, Message};
use mqttc::store;

pub trait Command {
    fn run(&self) -> !;
}

pub struct LocalStorage(BTreeMap<PacketIdentifier, Box<Message>>);

impl LocalStorage {
    pub fn new() -> Box<LocalStorage> {
        Box::new(LocalStorage(BTreeMap::new() as BTreeMap<PacketIdentifier, Box<Message>>))
    }
}

impl store::Store for LocalStorage {
    fn put(&mut self, message: Box<Message>) -> store::Result<()> {
        self.0.insert(message.pid.unwrap(), message);
        Ok(())
    }

    fn get(&mut self, pid: PacketIdentifier) -> store::Result<Box<Message>> {
        match self.0.get(&pid) {
            Some(m) => Ok(m.clone()),
            None => Err(store::Error::NotFound(pid))
        }
    }

    fn delete(&mut self, pid: PacketIdentifier) -> store::Result<()> {
        self.0.remove(&pid);
        Ok(())
    }
}
