use std::collections::{HashMap, VecDeque};
use std::io::{Read, Write, ErrorKind};
use std::net::{SocketAddr, ToSocketAddrs};
use std::time::{Duration, Instant};
use std::thread;
use netopt::{Connection, NetworkOptions, NetworkStream};
use rand::{self, Rng};
use mqtt3::{MqttRead, MqttWrite, Message, QoS, SubscribeReturnCodes, SubscribeTopic};
use mqtt3::{self, Protocol, Packet, ConnectReturnCode, PacketIdentifier, LastWill, ToTopicPath};
use mqtt3::Error as MqttError;
use error::{Error, Result};
use sub::Subscription;
use {Mqttc, ClientState, ReconnectMethod, PubOpt, ToPayload, ToSubTopics, ToUnSubTopics};

#[derive(Debug, Clone)]
pub struct ClientOptions {
    protocol: Protocol,
    keep_alive: Option<Duration>,
    clean_session: bool,
    client_id: Option<String>,
    last_will: Option<LastWill>,
    username: Option<String>,
    password: Option<String>,
    reconnect: ReconnectMethod
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
            reconnect: ReconnectMethod::ForeverDisconnect
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

    pub fn set_reconnect(&mut self, reconnect: ReconnectMethod) -> &mut ClientOptions {
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
            state: ClientState::Disconnected,
            netopt: netopt,
            opts: self,
            conn: conn,
            session_present: false,

            // Queues
            last_flush: Instant::now(),
            last_pid: PacketIdentifier::zero(),
            await_ping: false,
            incomming: VecDeque::new(),
            outgoing: VecDeque::new(),
            await_suback: VecDeque::new(),
            await_unsuback: VecDeque::new(),
            subscriptions: HashMap::new()
            // Subscriptions
        };

        // Send CONNECT then wait CONNACK
        try!(client._handshake());

        Ok(client)
    }

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
    last_flush: Instant,
    last_pid: PacketIdentifier,
    await_ping: bool,
    incomming: VecDeque<Box<mqtt3::Message>>, // only QoS > 0
    outgoing: VecDeque<Box<mqtt3::Message>>,  // only QoS > 0
    await_suback: VecDeque<Box<mqtt3::Subscribe>>,
    await_unsuback: VecDeque<Box<mqtt3::Unsubscribe>>,
    // Subscriptions
    subscriptions: HashMap<String, Subscription>
}

impl Mqttc for Client {
    fn publish<T, P>(&mut self, topic: T, payload: P, pubopt: PubOpt) -> Result<()>
            where T : ToTopicPath, P: ToPayload {
        self._publish(topic, payload, pubopt);
        self._flush()
    }

    fn subscribe<S: ToSubTopics>(&mut self, subs: S) -> Result<()> {
        self._subscribe(subs);
        self._flush()
    }

    fn unsubscribe<U: ToUnSubTopics>(&mut self, unsubs: U) -> Result<()> {
        self._unsubscribe(unsubs);
        self._flush()
    }

    fn disconnect(mut self) -> Result<()> {
        //self._disconnect();
        self._flush()
    }
}

impl Client {
    pub fn await(&mut self) -> Result<Option<Box<Message>>> {
        loop {
            match self.accept() {
                Ok(message) => {
                    if let Some(m) = message {
                        return Ok(Some(m))
                    }
                },
                Err(e) => match e {
                    Error::Timeout => {
                        if self.state == ClientState::Connected {
                            if !self.await_ping {
                                let _ = self.ping();
                            } else {
                                self._unbind();
                            }
                        } else {
                            return Err(Error::Timeout)
                        }
                    },
                    _ => return Err(e)
                }
            }
            if self._normalized() {
                return Ok(None);
            }
        }
    }

    pub fn accept(&mut self) -> Result<Option<Box<Message>>> {
        match self.state {
            ClientState::Connected | ClientState::Handshake => {
                // Don't forget to send PING packets in time
                if let Some(keep_alive) = self.opts.keep_alive {
                    let elapsed = self.last_flush.elapsed();
                    if elapsed >= keep_alive {
                        return Err(Error::Timeout);
                    }
                    self.conn.set_read_timeout(Some(keep_alive - elapsed));
                }

                match self.conn.read_packet() {
                    Ok(packet) => {
                        match self._parse_packet(&packet) {
                            Ok(message) => Ok(message),
                            Err(err) => {
                                self._unbind();
                                debug!("{:?}", err);
                                if self._try_reconnect() {
                                    Ok(None)
                                } else {
                                    Err(err)
                                }
                            }
                        }
                    },
                    Err(err) => match err {
                        mqtt3::Error::UnexpectedEof => {
                            debug!("{:?}", err);
                            if self._try_reconnect() {
                                Ok(None)
                            } else {
                                Err(Error::Disconnected)
                            }
                        },
                        mqtt3::Error::Io(e) => match e.kind() {
                            ErrorKind::WouldBlock | ErrorKind::TimedOut => {
                                Err(Error::Timeout)
                            },
                            ErrorKind::UnexpectedEof | ErrorKind::ConnectionRefused |
                            ErrorKind::ConnectionReset | ErrorKind::ConnectionAborted => {
                                debug!("{:?}", e);
                                self._unbind();
                                if self._try_reconnect() {
                                    Ok(None)
                                } else {
                                    Err(Error::Disconnected)
                                }
                            },
                            _ => {
                                debug!("{:?}", e);
                                self._unbind();
                                Err(Error::from(e))
                            }
                        },
                        _ => {
                            debug!("{:?}", err);
                            self._unbind();
                            Err(Error::from(err))
                        }
                    }
                }
            },
            ClientState::Disconnected => {
                if self._try_reconnect() {
                    Ok(None)
                } else {
                    Err(Error::Disconnected)
                }
            }
        }
    }

    pub fn reconnect(&mut self) -> Result<()> {
        if self.state == ClientState::Connected {
            warn!("The client is already connected.");
            return Ok(());
        };
        let (conn, _) = try!(self.opts._reconnect(self.addr, &self.netopt));
        self.conn = conn;
        try!(self._handshake());
        Ok(())
    }

    pub fn ping(&mut self) -> Result<()> {
        debug!("PING");
        self.await_ping = true;
        self._write_packet(&Packet::Pingreq);
        self._flush()
    }

    pub fn set_reconnect(&mut self, reconnect: ReconnectMethod) {
        self.opts.reconnect = reconnect;
    }

    pub fn session_present(&self) -> bool {
        self.session_present
    }

    fn _normalized(&self) -> bool {
        (self.state == ClientState::Connected) &&
        (!self.await_ping) &&
        (self.outgoing.len() == 0) &&
        (self.incomming.len() == 0) &&
        (self.await_suback.len() == 0) &&
        (self.await_unsuback.len() == 0)
    }

    fn _parse_packet(&mut self, packet: &Packet) -> Result<Option<Box<Message>>> {
        trace!("{:?}", packet);
        match self.state {
            ClientState::Handshake => {
                match packet {
                    &Packet::Connack(connack) => {
                        if connack.code == ConnectReturnCode::Accepted {
                            self.session_present = connack.session_present;
                            self.state = ClientState::Connected;
                            info!("The client has connected successful.");
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
                    &Packet::Publish(ref publish) => {
                        let message = try!(Message::from_pub(publish.clone()));

                        match message.qos {
                            QoS::AtMostOnce => (),
                            QoS::AtLeastOnce => {
                                self.incomming.push_back(message.clone());
                                let pid = message.pid.unwrap();
                                debug!("PUBACK {}", pid.0);
                                self._write_packet(&Packet::Puback(pid));
                                try!(self._flush());
                                let _ = self.incomming.pop_front();
                            },
                            QoS::ExactlyOnce => {
                                return Err(Error::UnsupportedFeature);
                            }
                        }

                        Ok(Some(message))
                    },
                    &Packet::Puback(pid) => {
                        if let Some(publish) = self.outgoing.pop_front() {
                            if publish.pid == Some(pid) {
                                if publish.qos == QoS::AtLeastOnce {
                                    Ok(None)
                                } else {
                                    Err(Error::ProtocolViolation)
                                }
                            } else {
                                Err(Error::ProtocolViolation)
                            }
                        } else {
                            Err(Error::ProtocolViolation)
                        }
                    },
                    &Packet::Suback(ref suback) => {
                        if let Some(subscribe) = self.await_suback.pop_front() {
                            if subscribe.pid == suback.pid {
                                if subscribe.topics.len() == suback.return_codes.len() {
                                    let iter = suback.return_codes.iter().zip(&subscribe.topics);
                                    for (ref code, ref sub_topic) in iter {
                                        match **code {
                                            SubscribeReturnCodes::Success(qos) => {
                                                let sub = Subscription {
                                                    pid: subscribe.pid,
                                                    topic_path: try!(sub_topic.topic_path.to_topic_path()),
                                                    qos: qos
                                                };
                                                self.subscriptions.insert(sub_topic.topic_path.clone(), sub);
                                            },
                                            SubscribeReturnCodes::Failure => {
                                                // ignore subscription
                                            }
                                        }
                                    }
                                    Ok(None)
                                } else {
                                    Err(Error::ProtocolViolation)
                                }
                            } else {
                                Err(Error::ProtocolViolation)
                            }
                        } else {
                            Err(Error::ProtocolViolation)
                        }
                    },
                    &Packet::Unsuback(pid) => {
                        if let Some(unsubscribe) = self.await_unsuback.pop_front() {
                            if unsubscribe.pid == pid {
                                for topic in unsubscribe.topics.iter() {
                                    self.subscriptions.remove(topic);
                                }
                                Ok(None)
                            } else {
                                Err(Error::ProtocolViolation)
                            }
                        } else {
                            Err(Error::ProtocolViolation)
                        }
                    },
                    &Packet::Pingresp => {
                        self.await_ping = false;
                        Ok(None )
                    },
                    &Packet::Pubcomp | &Packet::Pubrec | &Packet::Pubrel => Err(Error::UnsupportedFeature),
                    _ => Err(Error::UnrecognizedPacket)
                }
            },
            ClientState::Disconnected => Err(Error::ConnectionAbort)
        }
    }

    fn _handshake(&mut self) -> Result<()> {
        self.state = ClientState::Handshake;
        // send CONNECT
        try!(self._connect());
        // wait CONNACK
        let _ = try!(self.await());
        Ok(())
    }

    fn _try_reconnect(&mut self) -> bool {
        match self.opts.reconnect {
            ReconnectMethod::ForeverDisconnect => false,
            ReconnectMethod::ReconnectAfter(dur) => {
                info!("Trying to reconnect in {} seconds...", dur.as_secs());
                thread::sleep(dur);
                let _ = self.reconnect();
                true
            }
        }
    }

    fn _connect(&mut self) -> Result<()> {
        let connect = self.opts._generate_connect_packet();
        debug!("CONNECT client_id: {}", connect.client_id);
        let packet = Packet::Connect(connect);
        self._write_packet(&packet);
        self._flush()
    }

    fn _publish<T: ToTopicPath, P: ToPayload>(&mut self, topic: T, payload: P, pubopt: PubOpt) -> Result<()> {
        let mut message = Box::new(Message {
            topic: try!(topic.to_topic_name()),
            qos: pubopt.qos(),
            retain: pubopt.is_retain(),
            pid: None,
            payload: payload.to_payload()
        });

        match message.qos {
            QoS::AtMostOnce => (),
            QoS::AtLeastOnce => {
                message.pid = Some(self._next_pid());
                self.outgoing.push_back(message.clone());
            },
            QoS::ExactlyOnce => {
                return Err(Error::UnsupportedFeature);
            }
        }

        debug!("PUBLISH {} > {} bytes ({:?})", message.topic.path(), message.payload.len(), message.qos);
        let packet = Packet::Publish(message.to_pub(None, false));
        self._write_packet(&packet);
        Ok(())
    }

    fn _subscribe<S: ToSubTopics>(&mut self, subs: S) -> Result<()> {
        let iter = try!(subs.to_subscribe_topics());
        let subscribe = Box::new(mqtt3::Subscribe {
            pid: self._next_pid(),
            topics: iter.collect()
        });
        debug!("SUBSCRIBE {:?}", subscribe.topics);
        self.await_suback.push_back(subscribe.clone());
        self._write_packet(&Packet::Subscribe(subscribe));
        Ok(())
    }

    fn _unsubscribe<U: ToUnSubTopics>(&mut self, unsubs: U) -> Result<()> {
        let iter = try!(unsubs.to_unsubscribe_topics());
        let unsubscribe = Box::new(mqtt3::Unsubscribe {
            pid: self._next_pid(),
            topics: iter.collect()
        });
        debug!("UNSUBSCRIBE {:?}", unsubscribe.topics);
        self.await_unsuback.push_back(unsubscribe.clone());
        self._write_packet(&Packet::Unsubscribe(unsubscribe));
        Ok(())
    }

    fn _disconnect(&mut self) {
        self._write_packet(&Packet::Disconnect);
    }

    #[inline]
    fn _write_packet(&mut self, packet: &Packet) {
        self.conn.write_packet(&packet).unwrap();
    }

    fn _flush(&mut self) -> Result<()> {
        // TODO: in case of disconnection, trying to reconnect
        try!(self.conn.flush());
        self.last_flush = Instant::now();
        Ok(())
    }

    fn _unbind(&mut self) {
        let _ = self.conn.terminate();
        self.await_unsuback.clear();
        self.await_suback.clear();
        self.await_ping = false;
        self.state = ClientState::Disconnected;
        info!("The client has been disconnected.");
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
