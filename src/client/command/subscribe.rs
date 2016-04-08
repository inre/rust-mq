use std::io::prelude::*;
use term;
use openssl::ssl;
use std::time::Duration;
use mqtt3::{LastWill, SubscribeTopic, QoS, Protocol};
use netopt::{NetworkOptions, SslContext};
use mqttc::{Mqttc, ClientOptions, ReconnectMethod};
use super::Command;
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

    // Preferences
    pub log_file: Option<String>,
    pub debug: bool,
    pub keep_alive: u16,
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
    pub ssl_context: Option<ssl::SslContext>
}

impl Default for SubscribeCommand {
    fn default() -> SubscribeCommand {
        SubscribeCommand {
            topics: vec![SubscribeTopic { topic_path: "#".to_string(), qos: QoS::AtLeastOnce }],
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

        if !self.silence {
            print_legend();
        };

        let address = format!("{}:{}", self.address, self.port);

        if !self.silence {
            print_message("Connecting to", format!("{}", address), term::color::BRIGHT_GREEN);
        };

        let mut client = opts.connect(address.as_str(), netopt).unwrap();

        if !self.silence {
            print_topics(&self.topics);
        };

        // Subscribe to topics
        client.subscribe(self.topics.clone()).unwrap();

        loop {
            match client.await().unwrap() {
                Some(message) => {
                    let payload = String::from_utf8((*message.payload).clone()).unwrap();
                    let color = match message.qos {
                        QoS::AtMostOnce => term::color::BRIGHT_CYAN,
                        QoS::AtLeastOnce => term::color::BRIGHT_MAGENTA,
                        QoS::ExactlyOnce => term::color::BRIGHT_BLUE
                    };
                    print_message(message.topic.path, payload, color);
                },
                None => {
                    //println!(".");
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

fn print_topics(topics: &Vec<SubscribeTopic>) {
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
