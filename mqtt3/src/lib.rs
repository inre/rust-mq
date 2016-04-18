extern crate byteorder;

mod error;
mod mqtt;
mod read;
mod write;
mod topic;
mod msg;

pub use error::{
    Error,
    Result
};

pub use msg::{
    Message
};

pub use mqtt::{
    Packet,
    Connect,
    Connack,
    Publish,
    Subscribe,
    Suback,
    Unsubscribe,
    SubscribeTopic,
    SubscribeReturnCodes
};

pub use topic::{
    Topic,
    TopicPath,
    ToTopicPath
};

pub use read::MqttRead;
pub use write::MqttWrite;

const MULTIPLIER: usize = 0x80 * 0x80 * 0x80 * 0x80;
const MAX_PAYLOAD_SIZE: usize = 268435455;

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    MQIsdp(u8),
    MQTT(u8)
}

impl Protocol {
    pub fn new(name: &str, level: u8) -> Result<Protocol> {
        match name {
            "MQIsdp" => match level {
                3 => Ok(Protocol::MQIsdp(3)),
                _ => Err(Error::UnsupportedProtocolVersion)
            },
            "MQTT" => match level {
                4 => Ok(Protocol::MQTT(4)),
                _ => Err(Error::UnsupportedProtocolVersion)
            },
            _ => Err(Error::UnsupportedProtocolName)
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            &Protocol::MQIsdp(_) => "MQIsdp",
            &Protocol::MQTT(_) => "MQTT"
        }
    }

    pub fn level(&self) -> u8 {
        match self {
            &Protocol::MQIsdp(level) => level,
            &Protocol::MQTT(level) => level
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QoS {
    AtMostOnce,
    AtLeastOnce,
    ExactlyOnce
}

impl QoS {
    pub fn from_u8(byte: u8) -> Result<QoS> {
        match byte {
            0 => Ok(QoS::AtMostOnce),
            1 => Ok(QoS::AtLeastOnce),
            2 => Ok(QoS::ExactlyOnce),
            _ => Err(Error::UnsupportedQualityOfService)
        }
    }

    #[inline]
    pub fn from_hd(hd: u8) -> Result<QoS> {
        Self::from_u8((hd & 0b110) >> 1)
    }

    pub fn to_u8(&self) -> u8 {
        match *self {
            QoS::AtMostOnce => 0,
            QoS::AtLeastOnce => 1,
            QoS::ExactlyOnce => 2
        }
    }

    pub fn min(&self, other: QoS) -> QoS {
        match *self {
            QoS::AtMostOnce => QoS::AtMostOnce,
            QoS::AtLeastOnce => {
                if other == QoS::AtMostOnce {
                    QoS::AtMostOnce
                } else {
                    QoS::AtLeastOnce
                }
            },
            QoS::ExactlyOnce => other
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketType {
	Connect,
	Connack,
	Publish,
	Puback,
	Pubrec,
	Pubrel,
	Pubcomp,
	Subscribe,
	Suback,
	Unsubscribe,
	Unsuback,
	Pingreq,
	Pingresp,
	Disconnect
}

impl PacketType {
    pub fn to_u8(&self) -> u8 {
        match *self {
            PacketType::Connect => 1,
            PacketType::Connack => 2,
            PacketType::Publish => 3,
            PacketType::Puback => 4,
            PacketType::Pubrec => 5,
            PacketType::Pubrel => 6,
            PacketType::Pubcomp => 7,
            PacketType::Subscribe => 8,
            PacketType::Suback => 9,
            PacketType::Unsubscribe => 10,
            PacketType::Unsuback => 11,
            PacketType::Pingreq => 12,
            PacketType::Pingresp => 13,
            PacketType::Disconnect => 14
        }
    }

    pub fn from_u8(byte: u8) -> Result<PacketType> {
        match byte {
            1 => Ok(PacketType::Connect),
            2 => Ok(PacketType::Connack),
            3 => Ok(PacketType::Publish),
            4 => Ok(PacketType::Puback),
            5 => Ok(PacketType::Pubrec),
            6 => Ok(PacketType::Pubrel),
            7 => Ok(PacketType::Pubcomp),
            8 => Ok(PacketType::Subscribe),
            9 => Ok(PacketType::Suback),
            10 => Ok(PacketType::Unsubscribe),
            11 => Ok(PacketType::Unsuback),
            12 => Ok(PacketType::Pingreq),
            13 => Ok(PacketType::Pingresp),
            14 => Ok(PacketType::Disconnect),
            _ => Err(Error::UnsupportedPacketType)
        }
    }

    #[inline]
    pub fn from_hd(hd: u8) -> Result<PacketType> {
        Self::from_u8(hd >> 4)
    }
}

impl fmt::Display for PacketType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str = format!("{:?}", self);
        let first_space = str.find(' ').unwrap_or(str.len());
        let (str, _) = str.split_at(first_space);
        f.write_str(&str)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectReturnCode {
    Accepted,
    RefusedProtocolVersion,
    RefusedIdentifierRejected,
    ServerUnavailable,
    BadUsernamePassword,
    NotAuthorized
}

impl ConnectReturnCode {
    pub fn to_u8(&self) -> u8 {
        match *self {
            ConnectReturnCode::Accepted => 0,
            ConnectReturnCode::RefusedProtocolVersion => 1,
            ConnectReturnCode::RefusedIdentifierRejected => 2,
            ConnectReturnCode::ServerUnavailable => 3,
            ConnectReturnCode::BadUsernamePassword => 4,
            ConnectReturnCode::NotAuthorized => 5
        }
    }

    pub fn from_u8(byte: u8) -> Result<ConnectReturnCode> {
        match byte {
            0 => Ok(ConnectReturnCode::Accepted),
            1 => Ok(ConnectReturnCode::RefusedProtocolVersion),
            2 => Ok(ConnectReturnCode::RefusedIdentifierRejected),
            3 => Ok(ConnectReturnCode::ServerUnavailable),
            4 => Ok(ConnectReturnCode::BadUsernamePassword),
            5 => Ok(ConnectReturnCode::NotAuthorized),
            _ => Err(Error::UnsupportedConnectReturnCode)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PacketIdentifier(pub u16);

impl PacketIdentifier {
    pub fn zero() -> PacketIdentifier {
        PacketIdentifier(0)
    }

    pub fn next(&self) -> PacketIdentifier {
        PacketIdentifier(self.0 + 1)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Header {
    hd: u8,
    pub typ: PacketType,
    pub len: usize
}

impl Header {
    pub fn new(hd: u8, len: usize) -> Result<Header> {
        Ok(Header {
            hd: hd,
            typ: try!(PacketType::from_hd(hd)),
            len: len
        })
    }

    #[inline]
    pub fn dup(&self) -> bool {
        (self.hd & 0b1000) != 0
    }

    #[inline]
    pub fn qos(&self) -> Result<QoS> {
        QoS::from_hd(self.hd)
    }

    #[inline]
    pub fn retain(&self) -> bool {
        (self.hd & 1) != 0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LastWill {
    pub topic: String,
    pub message: String,
    pub qos: QoS,
    pub retain: bool
}

#[cfg(test)]
mod test {
    use super::{QoS, Protocol, PacketIdentifier};

    #[test]
    fn protocol_test() {
        assert_eq!(Protocol::new("MQTT", 4).unwrap(), Protocol::MQTT(4));
        assert_eq!(Protocol::new("MQIsdp", 3).unwrap(), Protocol::MQIsdp(3));
        assert_eq!(Protocol::MQIsdp(3).name(), "MQIsdp");
        assert_eq!(Protocol::MQTT(4).name(), "MQTT");
        assert_eq!(Protocol::MQTT(3).level(), 3);
        assert_eq!(Protocol::MQTT(4).level(), 4);
    }

    #[test]
    fn qos_min_test() {
        assert_eq!(QoS::AtMostOnce.min(QoS::AtMostOnce), QoS::AtMostOnce);
        assert_eq!(QoS::AtMostOnce.min(QoS::AtLeastOnce), QoS::AtMostOnce);
        assert_eq!(QoS::AtLeastOnce.min(QoS::AtMostOnce), QoS::AtMostOnce);
        assert_eq!(QoS::AtLeastOnce.min(QoS::ExactlyOnce), QoS::AtLeastOnce);
        assert_eq!(QoS::ExactlyOnce.min(QoS::AtMostOnce), QoS::AtMostOnce);
        assert_eq!(QoS::ExactlyOnce.min(QoS::ExactlyOnce), QoS::ExactlyOnce);
    }

    #[test]
    fn packet_identifier_test() {
        let pid = PacketIdentifier::zero();
        assert_eq!(pid, PacketIdentifier(0));
        assert_eq!(pid.next(), PacketIdentifier(1));
    }
}
