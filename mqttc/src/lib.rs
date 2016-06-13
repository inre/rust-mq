#[macro_use] extern crate log;
extern crate rand;
extern crate byteorder;
extern crate mqtt3;
extern crate netopt;

mod error;
mod sub;
mod client;
pub mod store;

pub use error::{
    Error,
    Result
};

pub use sub::{
    ToSubTopics,
    ToUnSubTopics
};

pub use client::{
    Client,
    ClientOptions
};

use std::sync::Arc;
use std::ops;
use std::time::Duration;
use mqtt3::{QoS, ToTopicPath};

const MAX_QOS: QoS = mqtt3::QoS::AtLeastOnce;

pub trait PubSub {
    fn publish<T: ToTopicPath, P: ToPayload>(&mut self, topic: T, payload: P, pubopt: PubOpt) -> Result<()>;
    fn subscribe<S: ToSubTopics>(&mut self, subs: S) -> Result<()>;
    fn unsubscribe<U: ToUnSubTopics>(&mut self, unsubs: U) -> Result<()>;
    fn disconnect(self) -> Result<()>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientState {
    Handshake,
    Connected,
    Disconnected
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReconnectMethod {
    ForeverDisconnect,
    ReconnectAfter(Duration)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PubOpt(u8);

impl PubOpt {
    pub fn new(qos: QoS, retain: bool) -> PubOpt {
        let mut opt = PubOpt(qos.to_u8());
        if retain {
            opt = opt | PubOpt::retain();
        }
        opt
    }

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

#[cfg(test)]
mod test {
    use super::PubOpt;
    use mqtt3::QoS;

    #[test]
    fn pubopt_test() {
        let pubopt = PubOpt::at_least_once();
        assert_eq!(pubopt.qos(), QoS::AtLeastOnce);
        assert!(!pubopt.is_retain());

        let pubopt = PubOpt::at_most_once();
        assert_eq!(pubopt.qos(), QoS::AtMostOnce);
        assert!(!pubopt.is_retain());

        let pubopt = PubOpt::exactly_once();
        assert_eq!(pubopt.qos(), QoS::ExactlyOnce);
        assert!(!pubopt.is_retain());

        let pubopt = PubOpt::at_least_once() | PubOpt::retain();
        assert_eq!(pubopt.qos(), QoS::AtLeastOnce);
        assert!(pubopt.is_retain());

        let pubopt = PubOpt::at_most_once() | PubOpt::retain();
        assert_eq!(pubopt.qos(), QoS::AtMostOnce);
        assert!(pubopt.is_retain());

        let pubopt = PubOpt::exactly_once() | PubOpt::retain();
        assert_eq!(pubopt.qos(), QoS::ExactlyOnce);
        assert!(pubopt.is_retain());

        let pubopt = PubOpt::new(QoS::AtLeastOnce, true);
        assert_eq!(pubopt.qos(), QoS::AtLeastOnce);
        assert!(pubopt.is_retain());

        let pubopt = PubOpt::new(QoS::ExactlyOnce, true);
        assert_eq!(pubopt.qos(), QoS::ExactlyOnce);
        assert!(pubopt.is_retain());

        let pubopt = PubOpt::new(QoS::AtMostOnce, true);
        assert_eq!(pubopt.qos(), QoS::AtMostOnce);
        assert!(pubopt.is_retain());
    }
}
