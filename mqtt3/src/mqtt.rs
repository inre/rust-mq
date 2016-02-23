use std::sync::Arc;
use super::{QoS, LastWill, PacketIdentifier, Protocol, ConnectReturnCode};

#[derive(Debug, Clone, PartialEq)]
pub enum Packet {
	Connect(Arc<Connect>),
	Connack(Connack),
	Publish(Arc<Publish>),
	Puback(PacketIdentifier),
	Pubrec(PacketIdentifier),
	Pubrel(PacketIdentifier),
	Pubcomp(PacketIdentifier),
	Subscribe(Arc<Subscribe>),
	Suback(Arc<Suback>),
	Unsubscribe(Arc<Unsubscribe>),
	Unsuback(PacketIdentifier),
	Pingreq,
	Pingresp,
	Disconnect
}

#[derive(Debug, Clone, PartialEq)]
pub struct Connect {
	pub protocol: Protocol,
    pub keep_alive: u16,
    pub client_id: String,
	pub clean_session: bool,
    pub last_will: Option<LastWill>,
    pub username: Option<String>,
    pub password: Option<String>
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Connack {
    pub session_present: bool,
    pub code: ConnectReturnCode
}

#[derive(Debug, Clone, PartialEq)]
pub struct Publish {
    pub dup: bool,
    pub qos: QoS,
    pub retain: bool,
    pub topic_name: String,
    pub pid: Option<PacketIdentifier>,
    pub payload: Arc<Vec<u8>>
}

#[derive(Debug, Clone, PartialEq)]
pub struct Subscribe {
    pub pid: PacketIdentifier,
	// (topic path, qos)
	pub topics: Vec<(String, QoS)>
}

#[derive(Debug, Clone, PartialEq)]
pub struct Suback {
    pub pid: PacketIdentifier,
	// (error, qos)
	pub return_codes: Vec<(bool, QoS)>
}

#[derive(Debug, Clone, PartialEq)]
pub struct Unsubscribe {
    pub pid: PacketIdentifier,
	pub topics: Vec<String>
}
