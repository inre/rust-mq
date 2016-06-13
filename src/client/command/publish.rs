use std::io::prelude::*;
use std::fs::OpenOptions;
use std::path::Path;
use std::process::exit;

use openssl::ssl;
use mqtt3::{QoS, Protocol};
use netopt::{NetworkOptions, SslContext};
use mqttc::{PubSub, ClientOptions, PubOpt};
use super::{Command, LocalStorage};
use client::logger::set_stdout_logger;

#[derive(Debug, Clone)]
pub struct PublishCommand {
    // Publish
    pub topic: String,
    pub message: Option<String>,
    pub file: Option<String>,
    pub qos: QoS,
    pub retain: bool,

    // Connection
    pub address: String,
    pub port: u16,
    pub keep_alive: u16,

    // preferences
    pub debug: bool,
    pub protocol: Protocol,

    // Authorization
    pub client_id: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,

    // SSL/TLS option
    pub ssl_context: Option<ssl::SslContext>
}

impl Default for PublishCommand {
    fn default() -> PublishCommand {
        PublishCommand {
            topic: "nodefault".to_string(),
            message: None,
            file: None,
            qos: QoS::AtLeastOnce,
            retain: false,

            address: "localhost".to_string(),
            port: 1883,
            keep_alive: 30,

            debug: true,
            protocol: Protocol::MQTT(4),

            client_id: None,
            username: None,
            password: None,

            ssl_context: None
        }
    }
}

impl Command for PublishCommand {
    fn run(&self) -> ! {
        if self.debug {
            set_stdout_logger().unwrap();
        }

        debug!("{:?}", self);
        let mut netopt = NetworkOptions::new();

        if let Some(ref ssl_context) = self.ssl_context {
            let ssl = SslContext::new(ssl_context.clone());
            netopt.tls(ssl);
        };

        let mut opts = ClientOptions::new();
        opts.set_protocol(self.protocol);
        opts.set_keep_alive(self.keep_alive);
        opts.set_clean_session(true);
        opts.set_outgoing_store(LocalStorage::new());

        if let Some(ref username) = self.username {
            opts.set_username(username.clone());
        };

        if let Some(ref password) = self.password {
            opts.set_password(password.clone());
        };

        if let Some(ref client_id) = self.client_id {
            opts.set_client_id(client_id.clone());
        };

        let address = format!("{}:{}", self.address, self.port);
        let mut client = opts.connect(address.as_str(), netopt).expect("Can't connect to server");

        if let Some(ref message) = self.message {
            client.publish(self.topic.clone(), message.clone(), PubOpt::new(self.qos, self.retain)).expect("Can't publish the message");
        } else if let Some(ref file) = self.file {
            let path = Path::new(file);
            if !path.exists() {
                panic!("File not found");
            };

            let mut payload = Vec::new();
            let mut f = OpenOptions::new().read(true).open(file).expect("Can't open file");
            f.read_to_end(&mut payload).expect("Can't read file");
            println!("Sending file {} bytes...", payload.len());
            client.publish(self.topic.clone(), payload, PubOpt::new(self.qos, self.retain)).expect("Can't publish the message");
        } else {
            client.publish(self.topic.clone(), "", PubOpt::new(self.qos, self.retain)).expect("Can't publish the message");
        }

        if self.qos != QoS::AtMostOnce {
            // wait normalization
            while client.await().unwrap().is_some() {};
        }

        exit(0);
    }
}
