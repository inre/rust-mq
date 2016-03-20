use std::collections::{HashMap, VecDeque};
use std::io::{Read, Write, ErrorKind};
use std::net::{SocketAddr, ToSocketAddrs};
use std::time::Duration;
use netopt::{Connection, NetworkOptions, NetworkStream};
use rand::{self, Rng};
use mqtt3::{MqttRead, MqttWrite, Message};
use mqtt3::{self, Protocol, Packet, ConnectReturnCode, PacketIdentifier, LastWill, ToTopicPath};
use mqtt3::Error as MqttError;
use error::{Error, Result};
use sub::Subscription;
use {ClientState, PubOpt, ToPayload};

#[derive(Debug, Clone)]
pub struct ClientOptions {
    protocol: Protocol,
    keep_alive: Option<Duration>,
    clean_session: bool,
    client_id: Option<String>,
    last_will: Option<LastWill>,
    username: Option<String>,
    password: Option<String>,
    reconnect: bool
}

impl ClientOptions {
    pub fn new() -> ClientOptions {
        ClientOptions {
            protocol: Protocol::MQTT(4),
            keep_alive: Some(Duration::new(30, 0)),
            clean_session: true,
            client_id: None,
            last_will: None,
            username: None,
            password: None,
            reconnect: false
        }
    }

    pub fn set_keep_alive(&mut self, secs: u16) -> &mut ClientOptions {
        self.keep_alive = Some(Duration::new(secs as u64, 0)); self
    }

    pub fn set_protocol(&mut self, protocol: Protocol) -> &mut ClientOptions {
        self.protocol = protocol; self
    }

    pub fn set_client_id(&mut self, client_id: String) -> &mut ClientOptions {
        self.client_id = Some(client_id); self
    }

    pub fn generate_client_id(&mut self) -> &mut ClientOptions {
        let mut rng = rand::thread_rng();
        let id = rng.gen::<u32>();
        self.client_id = Some(format!("mqttc_{}", id));
        self
    }

    pub fn set_username(&mut self, username: String) -> &mut ClientOptions {
        self.username = Some(username); self
    }

    pub fn set_password(&mut self, password: String) -> &mut ClientOptions {
        self.password = Some(password); self
    }

    pub fn set_last_will<T: ToTopicPath, P: ToPayload>(&mut self, topic: T, message: String, pub_opt: PubOpt) -> Result<()> {
        let topic_name = try!(topic.to_topic_name());
        self.last_will = Some(LastWill {
            topic: try!(topic_name.to_topic_name()).path(),
            message: message,
            qos: pub_opt.qos(),
            retain: pub_opt.is_retain()
        });
        Ok(())
    }

    pub fn set_reconnect(&mut self, reconnect: bool) -> &mut ClientOptions {
        self.reconnect = reconnect; self
    }

    pub fn connect<A: ToSocketAddrs>(mut self, addr: A, netopt: NetworkOptions) -> Result<Client> {
        if self.client_id == None {
            self.generate_client_id();
        }

        let addr = try!(addr.to_socket_addrs()).next().expect("socket address is broken");

        info!("Establish connection to {}", addr);
        let (conn, _) = try!(self._reconnect(addr, &netopt));

        let mut client = Client {
            addr: addr,
            state: ClientState::Handshake,
            netopt: netopt,
            opts: self,
            conn: conn,
            session_present: false,

            // Queues
            last_pid: PacketIdentifier::zero(),
            await_ping: false,
            incomming: VecDeque::new(),
            outgoing: VecDeque::new(),
            await_suback: VecDeque::new(),
            await_unsuback: VecDeque::new(),
            subscriptions: HashMap::new()
            // Subscriptions
        };

        // send CONNECT
        try!(client.connect());
        // wait CONNACK
        try!(client.await());

        Ok(client)
    }

    /*pub fn async_connect<A: ToSocketAddrs>(self, addr: A, netopt: NetworkOptions) -> Result<(AsyncClient, Listener)> {
        if client_id == None {
            self.generate_client_id()
        }

        let conn = self._reconnect(addr, netopt);

        let mut client = Client {
            addr: addr,
            state: ClientState::Handshake,
            netopt: netopt,
            opts: self,
            conn: conn,
            session_present: false,

            // Queues
            last_pid: PacketIdentifier::zero(),
            await_ping: false,


            // Subscriptions
        };

        // send CONNECT
        //client.connect();
        // wait CONNACK
        //client.await();

        Ok(client)
    }*/

    fn _reconnect(&self, addr: SocketAddr, netopt: &NetworkOptions) -> Result<(Connection, NetworkStream)> {
        let stream = try!(netopt.connect(addr));
        stream.set_read_timeout(self.keep_alive);
        stream.set_write_timeout(self.keep_alive);
        Ok((try!(Connection::new(&stream)), stream))
    }

    fn _generate_connect_packet(&self) -> Box<mqtt3::Connect> {
        let keep_alive = if let Some(dur) = self.keep_alive {
            dur.as_secs() as u16
        } else {
            0
        };

        Box::new(mqtt3::Connect {
            protocol: self.protocol,
            keep_alive: keep_alive,
            client_id: self.client_id.clone().unwrap(),
            clean_session: self.clean_session,
            last_will: self.last_will.clone(),
            username: self.username.clone(),
            password: self.password.clone()
        })
    }
}

pub struct Client {
    addr: SocketAddr,
    state: ClientState,
    netopt: NetworkOptions,
    opts: ClientOptions,
    conn: Connection,
    session_present: bool,

    // Queues
    last_pid: PacketIdentifier,
    await_ping: bool,
    incomming: VecDeque<mqtt3::Message>, // only QoS > 0
    outgoing: VecDeque<mqtt3::Message>,  // only QoS > 0
    await_suback: VecDeque<Box<mqtt3::Subscribe>>,
    await_unsuback: VecDeque<Box<mqtt3::Unsubscribe>>,
    // Subscriptions
    subscriptions: HashMap<String, Subscription>
}

impl Client {
    pub fn await(&mut self) -> Result<Option<Message>> {
        loop {
            match self.accept() {
                Ok(message) => return Ok(message),
                Err(e) => match e {
                    Error::Timeout => self.ping(),
                    _ => return Err(e)
                }
            };
            if self._normalized() {
                return Ok(None);
            }
        }
    }

    pub fn accept(&mut self) -> Result<Option<Message>> {
        match self.conn.read_packet() {
            Ok(packet) => {
                self._parse_packet(&packet)
            },
            Err(err) => match err {
                mqtt3::Error::Io(e) => match e.kind() {
                    ErrorKind::WouldBlock | ErrorKind::TimedOut => {
                        Err(Error::Timeout)
                    },
                    _ => Err(Error::from(e))
                },
                _ => Err(Error::from(err))
            }
        }
    }

    pub fn connect(&mut self) -> Result<()> {
        let connect = self.opts._generate_connect_packet();
        debug!("CONNECT client_id: {}", connect.client_id);
        let packet = Packet::Connect(connect);
        self._write_packet(&packet);
        self._flush()
    }

    pub fn ping(&mut self) -> Result<()> {
        debug!("PING");
        self.await_ping = true;
        self._write_packet(&Packet::Pingreq);
        self._flush()
    }

    pub fn set_reconnect(&mut self, reconnect: bool) {
        self.opts.reconnect = reconnect;
    }

    pub fn session_present(&self) -> bool {
        self.session_present
    }

    fn _normalized(&self) -> bool {
        (self.state == ClientState::Connected) &&
        (!self.await_ping) &&
        (self.outgoing.len() == 0) &&
        (self.await_suback.len() == 0) &&
        (self.await_unsuback.len() == 0)
    }

    fn _parse_packet(&mut self, packet: &Packet) -> Result<Option<Message>> {
        trace!("{:?}", packet);
        match self.state {
            ClientState::Handshake => {
                match packet {
                    &Packet::Connack(connack) => {
                        if connack.code == ConnectReturnCode::Accepted {
                            self.session_present = connack.session_present;
                            self.state = ClientState::Connected;
                            Ok(None)
                        } else {
                            Err(Error::ConnectionRefused(connack.code))
                        }

                    },
                    _ => Err(Error::HandshakeFailed)
                }
            },
            ClientState::Connected => {
                match packet {
                    &Packet::Connack(_) => Err(Error::AlreadyConnected),
                    &Packet::Pingresp => {
                        self.await_ping = false;
                        Ok(None)
                    },
                    &Packet::Pubcomp | &Packet::Pubrec | &Packet::Pubrel => Err(Error::UnsupportedFeature),
                    _ => Err(Error::UnrecognizedPacket)
                }
            },
            ClientState::Disconnected => Err(Error::ConnectionAbort)
        }
    }

    fn _disconnect(&mut self) {
        self._write_packet(&Packet::Disconnect);
    }

    #[inline]
    fn _write_packet(&mut self, packet: &Packet) {
        self.conn.write_packet(&packet).unwrap();
    }

    fn _flush(&mut self) -> Result<()> {
        // TODO: in case of disconnection, try to reconnect
        try!(self.conn.flush());
        Ok(())
    }

    #[inline]
    fn _next_pid(&mut self) -> PacketIdentifier {
        self.last_pid = self.last_pid.next();
        self.last_pid
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;
    use super::ClientOptions;
    use netopt::{NetworkStream, NetworkOptions};
    use netopt::mock::MockStream;

    #[test]
    fn client_connect_test() {
        let stream = NetworkStream::Mock(MockStream::with_vec(vec![0b00100000, 0x02, 0x01, 0x00]));
        let options = ClientOptions::new();
        let mut netopt = NetworkOptions::new();
        netopt.attach(stream);
        // Connect and create MQTT client
        let client = options.connect("127.0.0.1:1883", netopt).unwrap();

    }
}
