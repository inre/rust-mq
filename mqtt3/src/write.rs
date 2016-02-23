use byteorder::{WriteBytesExt, BigEndian};
use error::{Error, Result};
use std::io::{BufWriter, Write, Cursor};
use std::net::TcpStream;
use super::{PacketType, Header, QoS, Protocol, PacketIdentifier, MAX_PAYLOAD_SIZE};

use mqtt::{
    Packet,
    Connect,
    Connack,
    Publish,
    Subscribe,
    Suback,
    Unsubscribe
};

pub trait MqttWrite: WriteBytesExt {
    fn write_packet(&mut self, packet: &Packet) -> Result<()> {
        match packet {
            &Packet::Connect(_) => Err(Error::UnsupportedPacketType),
			&Packet::Connack(ref connack) => {
                try!(self.write(&[0x20, 0x02, connack.session_present as u8, connack.code.to_u8()]));
                Ok(())
            },
			&Packet::Publish(ref publish) => {
                try!(self.write_u8(0b00110000 | publish.retain as u8 | (publish.qos.to_u8() << 1) | ((publish.dup as u8) << 3)));
                try!(self.write_remaining_length(publish.topic_name.len() + 4 + publish.payload.len()));
                try!(self.write_mqtt_string(publish.topic_name.as_str()));
                if publish.qos != QoS::AtMostOnce {
                    if let Some(pid) = publish.pid {
                        try!(self.write_u16::<BigEndian>(pid.0));
                    }
                }
                try!(self.write(&publish.payload.as_ref()));
                Ok(())
            },
			&Packet::Puback(ref pid) => {
                try!(self.write(&[0x40, 0x02]));
                try!(self.write_u16::<BigEndian>(pid.0));
                Ok(())
            },
			&Packet::Pubrec(_) => Err(Error::UnsupportedPacketType),
			&Packet::Pubrel(ref pid) => {
                try!(self.write(&[0x62, 0x02]));
                try!(self.write_u16::<BigEndian>(pid.0));
                Ok(())
            },
			&Packet::Pubcomp(_) => Err(Error::UnsupportedPacketType),
			&Packet::Subscribe(_) => Err(Error::UnsupportedPacketType),
			&Packet::Suback(ref suback) => {
                try!(self.write(&[0x90]));
                try!(self.write_remaining_length(suback.return_codes.len() + 2));
                try!(self.write_u16::<BigEndian>(suback.pid.0));
                let payload: Vec<u8> = suback.return_codes.iter().map({ |&(err, qos)| ((err as u8) << 7) & qos.to_u8() }).collect();
                try!(self.write(&payload));
                Ok(())
            },
			&Packet::Unsubscribe(_) => Err(Error::UnsupportedPacketType),
			&Packet::Unsuback(ref pid) => {
                try!(self.write(&[0xB0, 0x02]));
                try!(self.write_u16::<BigEndian>(pid.0));
                Ok(())
            },
			&Packet::Pingreq => {
                try!(self.write(&[0xc0, 0]));
                Ok(())
            },
			&Packet::Pingresp => {
                try!(self.write(&[0xd0, 0]));
                Ok(())
            },
			&Packet::Disconnect => {
                try!(self.write(&[0xe0, 0]));
                Ok(())
            }
        }
    }

    fn write_mqtt_string(&mut self, string: &str) -> Result<()> {
        try!(self.write_u16::<BigEndian>(string.len() as u16));
        try!(self.write(string.as_bytes()));
        Ok(())
    }

    fn write_remaining_length(&mut self, len: usize) -> Result<()> {
        if len > MAX_PAYLOAD_SIZE {
            return Err(Error::PayloadTooLong);
        }

        let mut done = false;
        let mut x = len;

        while !done {
            let mut byte = (x % 128) as u8;
            x = x / 128;
            if x > 0 {
                byte = byte | 128;
            }
            try!(self.write_u8(byte));
            done = x <= 0;
        }

        Ok(())
    }
}

impl MqttWrite for TcpStream {}
impl MqttWrite for Cursor<Vec<u8>> {}
impl<T: Write> MqttWrite for BufWriter<T> {}

#[cfg(test)]
mod test {
    use std::io::Cursor;
    use std::sync::Arc;
    use super::{MqttWrite};
    use super::super::{Protocol, LastWill, QoS, PacketIdentifier, ConnectReturnCode};
    use super::super::mqtt::{
        Packet,
        Connack,
        Publish
    };

    #[test]
    fn write_packet_connack_test() {
        let connack = Packet::Connack(Connack {
            session_present: true,
            code: ConnectReturnCode::Accepted
        });

        let mut stream = Cursor::new(Vec::new());
        stream.write_packet(&connack);

        assert_eq!(stream.get_ref().clone(), vec![0b00100000, 0x02, 0x01, 0x00]);
    }

    #[test]
    fn write_packet_publish_test() {
        let publish = Packet::Publish(Arc::new(Publish {
            dup: false,
            qos: QoS::AtLeastOnce,
            retain: false,
            topic_name: "a/b".to_owned(),
            pid: Some(PacketIdentifier(10)),
            payload: Arc::new(vec![0xF1, 0xF2, 0xF3, 0xF4])
        }));

        let mut stream = Cursor::new(Vec::new());
        stream.write_packet(&publish);

        assert_eq!(stream.get_ref().clone(), vec![0b00110010, 11, 0x00, 0x03, 'a' as u8, '/' as u8, 'b' as u8, 0x00, 0x0a, 0xF1, 0xF2, 0xF3, 0xF4]);
    }
}
