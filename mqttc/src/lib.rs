extern crate rand;
extern crate byteorder;
extern crate mqtt3;
extern crate netopt;

mod error;
mod sub;
mod client;

pub use sub::{
    ToSubTopics,
    ToUnSubTopics
};

use std::sync::Arc;
use std::ops;
use mqtt3::{QoS, ToTopicPath};
use error::Result;

const MAX_QOS: QoS = mqtt3::QoS::AtLeastOnce;

pub trait Mqttc {
    fn ping(&mut self) -> Result<()>;
    fn publish<T: ToTopicPath, P: ToPayload>(&mut self, topic: ToTopicPath, payload: ToPayload, pubopt: PubOpt) -> Result<()>;
    fn subscribe<S: ToSubTopics>(&mut self, subs: S) -> Result<()>;
    fn unsubscribe<U: ToUnSubTopics>(&mut self, unsubs: U) -> Result<()>;
    fn disconnect(self) -> Result<()>;
}

pub enum ClientState {
    Handshake,
    Connected,
    Disconnected
}

pub struct PubOpt(u8);

impl PubOpt {
    #[inline]
    pub fn at_most_once() -> PubOpt {
        PubOpt(0x00)
    }

    #[inline]
    pub fn at_least_once() -> PubOpt {
        PubOpt(0x01)
    }

    #[inline]
    pub fn exactly_once() -> PubOpt {
        PubOpt(0x02)
    }

    #[inline]
    pub fn retain() -> PubOpt {
        PubOpt(0x04)
    }

    #[inline]
    pub fn bits(&self) -> u8 {
        self.0
    }

    pub fn qos(&self) -> QoS {
        if (self.0 & PubOpt::exactly_once().bits()) != 0 {
            return QoS::ExactlyOnce;
        }
        if (self.0 & PubOpt::at_least_once().bits()) != 0 {
            return QoS::AtLeastOnce;
        }
        QoS::AtMostOnce
    }

    pub fn is_retain(&self) -> bool {
        (self.0 & PubOpt::retain().bits()) != 0
    }
}


impl ops::BitOr for PubOpt {
    type Output = PubOpt;

    #[inline]
    fn bitor(self, other: PubOpt) -> PubOpt {
        PubOpt(self.bits() | other.bits())
    }
}

impl ops::BitXor for PubOpt {
    type Output = PubOpt;

    #[inline]
    fn bitxor(self, other: PubOpt) -> PubOpt {
        PubOpt(self.bits() ^ other.bits())
    }
}

impl ops::BitAnd for PubOpt {
    type Output = PubOpt;

    #[inline]
    fn bitand(self, other: PubOpt) -> PubOpt {
        PubOpt(self.bits() & other.bits())
    }
}

impl ops::Sub for PubOpt {
    type Output = PubOpt;

    #[inline]
    fn sub(self, other: PubOpt) -> PubOpt {
        PubOpt(self.bits() & !other.bits())
    }
}

impl ops::Not for PubOpt {
    type Output = PubOpt;

    #[inline]
    fn not(self) -> PubOpt {
        PubOpt(!self.bits() & 0b111)
    }
}

pub type Payload = Arc<Vec<u8>>;

pub trait ToPayload {
    fn to_payload(&self) -> Payload;
}

impl ToPayload for String {
    fn to_payload(&self) -> Payload {
        Arc::new(self.clone().into_bytes())
    }
}

impl<'a> ToPayload for &'a str {
    fn to_payload(&self) -> Payload {
        Arc::new(self.as_bytes().to_vec())
    }
}

impl ToPayload for Vec<u8> {
    fn to_payload(&self) -> Payload {
        Arc::new(self.clone())
    }
}

impl ToPayload for Arc<Vec<u8>> {
    fn to_payload(&self) -> Payload {
        self.clone()
    }
}
