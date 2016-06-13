use std::io::prelude::*;
use term;
use openssl::ssl;
use std::time::Duration;
use std::process::exit;
use mqtt3::{self, LastWill, SubscribeTopic, QoS, Protocol};
use netopt::{NetworkOptions, SslContext};
use mqttc::store;
use mqttc::{PubSub, ClientOptions, ReconnectMethod, Error};
use super::{Command, LocalStorage};
use client::logger::set_stdout_logger;

#[derive(Debug, Clone)]
pub struct SubscribeCommand {
    // Subscribe
    pub topics: Vec<SubscribeTopic>,

    // Connection
    pub address: String,
    pub port: u16,
    pub clean_session: bool,
    pub last_will: Option<LastWill>,
    pub keep_alive: u16,

    // Preferences
    pub log_file: Option<String>,
    pub debug: bool,
    pub reconnect: bool,
    pub protocol: Protocol,
    pub silence: bool,

    // Authorization
    pub client_id: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,

    // Filters
    pub limit: Option<u32>,
    pub retain: bool,
    pub topic_filters: Vec<String>,

    // SSL/TLS option
    pub ssl_context: Option<ssl::SslContext>,
}

impl Default for SubscribeCommand {
    fn default() -> SubscribeCommand {
        SubscribeCommand {
            topics: vec![SubscribeTopic { topic_path: "#".to_string(), qos: QoS::ExactlyOnce }],
            address: "localhost".to_string(),
            port: 1883,
            clean_session: true,
            last_will: None,
            log_file: None,
            debug: true,
            keep_alive: 30,
            reconnect: false,
            protocol: Protocol::MQTT(4),
            silence: false,
            client_id: None,
            username: None,
            password: None,
            limit: None,
            retain: false,
            topic_filters: Vec::new(),
            ssl_context: None
        }
    }
}

impl Command for SubscribeCommand {
    fn run(&self) -> ! {
        if self.debug {
            set_stdout_logger().unwrap();
        }

        debug!("{:?}", self);
        let mut netopt = NetworkOptions::new();

        if let Some(ref ssl_context) = self.ssl_context {
            let ssl = SslContext::new(ssl_context.clone());
            netopt.tls(ssl);
            //print_message("TLS", ssl_context., term::color::BRIGHT_GREEN );
        };

        let mut opts = ClientOptions::new();
        opts.set_protocol(self.protocol);
        opts.set_keep_alive(self.keep_alive);
        opts.set_clean_session(self.clean_session);
        opts.set_last_will_opt(self.last_will.clone());
        opts.set_incomming_store(LocalStorage::new());

        if self.reconnect {
            opts.set_reconnect(ReconnectMethod::ReconnectAfter(Duration::from_secs(1)));
        };

        if let Some(ref username) = self.username {
            opts.set_username(username.clone());
        };

        if let Some(ref password) = self.password {
            opts.set_password(password.clone());
        };

        if let Some(ref client_id) = self.client_id {
            opts.set_client_id(client_id.clone());
        };

        if !self.debug && !self.silence {
            print_legend();
        };

        let address = format!("{}:{}", self.address, self.port);

        if !self.debug && !self.silence {
            print_message("Connecting to", address.as_str(), term::color::BRIGHT_GREEN);
        };

        let mut client = opts.connect(address.as_str(), netopt).expect("Can't connect to server");

        if !self.debug && !self.silence {
            print_topics(&self.topics);
        };

        // Subscribe to topics
        client.subscribe(self.topics.clone()).unwrap();

        loop {
            match client.await() {
                Ok(some_message) => {
                    if let Some(ref message) = some_message {
                        if !self.debug {
                            let color = match message.qos {
                                QoS::AtMostOnce => term::color::BRIGHT_CYAN,
                                QoS::AtLeastOnce => term::color::BRIGHT_MAGENTA,
                                QoS::ExactlyOnce => term::color::BRIGHT_BLUE
                            };

                            let payload = match String::from_utf8((*message.payload).clone()) {
                                Ok(payload) => payload,
                                Err(_) => {
                                    format!("payload did not contain valid UTF-8 ({} bytes)", message.payload.len())
                                }
                            };

                            print_message(&message.topic.path, payload, color);
                        }

                        if message.qos == QoS::ExactlyOnce {
                            let _ = client.complete(message.pid.unwrap());
                        }

                    }
                },
                Err(e) => {
                    match e {
                        Error::UnhandledPuback(_) => { print_error("unhandled puback") },
                        Error::UnhandledPubrec(_) => { print_error("unhandled pubrec") },
                        Error::UnhandledPubrel(_) => { print_error("unhandled pubrel") },
                        Error::UnhandledPubcomp(_) => { print_error("unhandled pubcomp") },
                        Error::Mqtt(ref err) => match *err {
                            mqtt3::Error::TopicNameMustNotContainNonUtf8 => {
                                print_error("topic name contains non-UTF-8 characters")
                            },
                            mqtt3::Error::TopicNameMustNotContainWildcard => {
                                print_error("topic name contains wildcard")
                            },
                            _ => {
                                print_error(format!("{:?}", e));
                                exit(64);
                            }
                        },
                        Error::Storage(ref err) => match *err {
                            store::Error::NotFound(pid) => {
                                // we have lost something
                                let _ = client.complete(pid);
                            },
                            store::Error::Unavailable(_) => {
                                // do nothing, just wait next pubrel
                            }
                        },
                        Error::Disconnected | Error::ConnectionAbort => {
                            exit(64);
                        },
                        e => {
                            print_error(format!("{:?}", e));
                            client.terminate();
                            exit(64);
                        }
                    }
                }
            }
        }

    }
}

fn print_legend() {
    let mut t = term::stdout().unwrap();
    t.fg(term::color::BRIGHT_GREEN).unwrap();
    write!(t, "        Legend ").unwrap();
    t.fg(term::color::BRIGHT_CYAN).unwrap();
    write!(t, "QoS 1 ").unwrap();
    t.fg(term::color::BRIGHT_MAGENTA).unwrap();
    write!(t, "QoS 2 ").unwrap();
    t.fg(term::color::BRIGHT_BLUE).unwrap();
    writeln!(t, "QoS 3 ").unwrap();
    t.fg(term::color::BRIGHT_GREEN).unwrap();
    //writeln!(t, "Network").unwrap();
    //t.reset().unwrap();
}

fn print_topics(topics: &[SubscribeTopic]) {
    let mut t = term::stdout().unwrap();
    t.fg(term::color::BRIGHT_GREEN).unwrap();
    write!(t, "     Subscribe ").unwrap();
    for topic in topics {
        t.fg(match topic.qos {
            QoS::AtMostOnce => term::color::BRIGHT_CYAN,
            QoS::AtLeastOnce => term::color::BRIGHT_MAGENTA,
            QoS::ExactlyOnce => term::color::BRIGHT_BLUE
        }).unwrap();
        write!(t, "{} ", topic.topic_path).unwrap();
    }
    t.reset().unwrap();
    writeln!(t, "").unwrap();
}

fn print_message<T: AsRef<str>, M: AsRef<str>>(title: T, message: M, color: u16) {
    let mut t = term::stdout().unwrap();
    t.fg(color).unwrap();
    write!(t, "{:>14} ", title.as_ref()).unwrap();
    t.reset().unwrap();
    writeln!(t, "{}", message.as_ref()).unwrap();
    t.reset().unwrap();
}

fn print_error<M: AsRef<str>>(message: M) {
    print_message("Error", message, term::color::BRIGHT_RED);
}
