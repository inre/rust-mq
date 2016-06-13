use std::process::exit;
use getopts::Options;
use openssl::ssl::{SslMethod, SslContext, SslVerifyMode};
use openssl::x509::X509FileType;
use mqtt3::{LastWill, SubscribeTopic, QoS, Protocol};
use super::command::{Command, SubscribeCommand, PublishCommand};

pub struct CLI {
    program: String,
    command: String,
    arguments: Vec<String>
}

impl CLI {
    pub fn new<C: IntoIterator<Item=String>>(args: C) -> CLI {
        let mut args: Vec<String> = args.into_iter().collect();
        let program = args.remove(0);
        let command = if args.len() < 1 {
            "help".to_string()
        } else {
            args.remove(0)
        };

        CLI {
            program: program,
            command: command,
            arguments: args
        }
    }

    pub fn parse(&self) -> Box<Command> {
        match self.command.as_str() {
            "subscribe" | "sub" => Box::new(self.subscribe_parse()),
            "publish" | "pub" => Box::new(self.publish_parse()),
            "help" | _ => {
                self.print_usage();
                exit(0);
            }
        }
    }

    pub fn publish_parse(&self) -> PublishCommand {
        let default = PublishCommand::default();

        let mut opts = Options::new();
        opts.optopt("t", "", "The topic name which payload data is published.", "topic");
        opts.optopt("m", "", "Message payload to send", "message");
        opts.optopt("f", "", "Send file as the payload", "file");
        opts.optopt("q", "", "Quality of service level", "qos");
        opts.optflag("r", "", "Message should be retained");

        opts.optopt("a", "", "Address to connect to. Defaults to localhost", "address");
        opts.optopt("p", "", "Port to connect to. Defaults to 1883", "port");
        opts.optopt("k", "", "Keep alive the link with the server then try to send ping request. Defaults to 60", "seconds");
        opts.optopt("i", "", "Specifies a client id", "client_id");
        opts.optopt("u", "", "Specifies a username with which to authenticate to", "username");
        opts.optopt("P", "", "Specifies a password with which to authenticate to", "password");
        opts.optopt("v", "", "MQTT protocol version. Can be 3.1 or 3.1.1", "version");
        opts.optflag("d", "", "Show debug messages");

        opts.optopt("", "tls", "Enables TLS and sets protocol version. Can be tlsv1, tlsv1.1, tlsv1.2", "");
        opts.optopt("", "cafile", "Specifies the file that contains trusted CA certificates.", "file");
        //opts.optopt("", "capath", "TODO", "path");
        opts.optopt("", "key", "Path to private key", "path");
        opts.optopt("", "cert", "Path to certificate", "path");
        opts.optflag("", "no-verify", "Disables client cert requests");

        opts.optflag("h", "help", "Display this message");

        let matches = match opts.parse(&self.arguments[..]) {
            Ok(m) => { m }
            Err(f) => {
                self.cli_error(f.to_string());
            }
        };

        if matches.opt_present("h") {
            self.publish_print_usage(opts);
            exit(0);
        };

        let topic = matches.opt_str("t").unwrap_or_else(|| self.cli_error("Please set the topic name"));
        let message = matches.opt_str("m");
        let file = matches.opt_str("f");
        if message.is_some() && file.is_some() {
            self.cli_error("Shouldn't set both message and file together");
        };

        let qos = matches.opt_str("q").map_or(QoS::AtLeastOnce, |s| self.parse_qos(s));
        let retain = matches.opt_present("r");

        let address = matches.opt_str("a").unwrap_or(default.address);
        let port = if matches.opt_present("p") {
            match matches.opt_str("p").unwrap().parse::<u16>() {
                Ok(v) => v,
                Err(_) => {
                    self.cli_error("port format error");
                }
            }
        } else {
            default.port
        };

        let client_id = matches.opt_str("i");
        let username = matches.opt_str("u");
        let password = matches.opt_str("P");

        let debug = matches.opt_present("d");
        let keep_alive = if matches.opt_present("k") {
            match matches.opt_str("k").unwrap().parse::<u16>() {
                Ok(v) => v,
                Err(_) => {
                    self.cli_error("keep alive format error");
                }
            }
        } else {
            default.keep_alive
        };
        let protocol = if matches.opt_present("v") {
            match matches.opt_str("v").unwrap().as_ref() {
                "3.1" => Protocol::MQIsdp(3),
                "3.1.1" => Protocol::MQTT(4),
                _ => {
                    self.cli_error("unsupported protocol version");
                }
            }
        } else {
            default.protocol
        };

        let cafile = matches.opt_str("cafile");
        let key = matches.opt_str("key");
        let cert = matches.opt_str("cert");
        let ssl_method = if matches.opt_present("tls") {
            match matches.opt_str("tls").unwrap().as_ref() {
                "1" => Some(SslMethod::Tlsv1),
                "1.1" => Some(SslMethod::Tlsv1_1),
                "1.2" => Some(SslMethod::Tlsv1_2),
                _ => {
                    self.cli_error("unsupported TLS version")
                }
            }
        } else {
            None
        };
        let verify_mode = if matches.opt_present("no-verify") {
            SslVerifyMode::from_bits_truncate(0)
        } else {
            SslVerifyMode::from_bits_truncate(1)
        };

        let ssl_context = ssl_method.map(|ssl| {
            let mut context = SslContext::new(ssl).unwrap();
            context.set_verify(verify_mode, None);
            if let Some(ref cafile_path) = cafile {
                context.set_CA_file(cafile_path).unwrap();
            }
            if let Some(ref key_path) = key {
                context.set_private_key_file(key_path, X509FileType::PEM).unwrap();
            }
            if let Some(ref cert_path) = cert {
                context.set_certificate_file(cert_path, X509FileType::PEM).unwrap();
            }
            context
        });

        PublishCommand {
            topic: topic,
            message: message,
            file: file,
            qos: qos,
            retain: retain,

            // Connection
            address: address,
            port: port,
            keep_alive: keep_alive,

            // preferences
            debug: debug,
            protocol: protocol,

            // Authorization
            client_id: client_id,
            username: username,
            password: password,

            // SSL/TLS option
            ssl_context: ssl_context
        }
    }

    pub fn subscribe_parse(&self) -> SubscribeCommand {
        let default = SubscribeCommand::default();

        let mut opts = Options::new();
        opts.optopt("a", "", "Address to connect to. Defaults to localhost", "address");
        opts.optopt("p", "", "Port to connect to. Defaults to 1883", "port");
        opts.optopt("q", "", "Maximum quality of service level", "qos");
        opts.optopt("k", "", "Keep alive the link with the server then try to send ping request. Defaults to 60", "seconds");
        opts.optflag("r", "", "Reconnect automatically if connection was broken.");
        opts.optopt("i", "", "Specifies a client id", "client_id");
        opts.optopt("u", "", "Specifies a username with which to authenticate to", "username");
        opts.optopt("P", "", "Specifies a password with which to authenticate to", "password");
        //opts.optopt("l", "", "Log messages to specified file", "log_file");
        opts.optopt("v", "", "MQTT protocol version. Can be 3.1 or 3.1.1", "version");
        opts.optflag("c", "", "Set 'clean session' flag");
        opts.optflag("d", "", "Show debug messages");
        opts.optflag("s", "", "Prevent to show a connection messages");


        //opts.optopt("f", "", "Piece of topic path to filter out incomming messages. Can be repeated.", "filter");
        //opts.optflag("", "no-retain", "Hide retained messages");
        //opts.optopt("", "limit", "Disconnect after `limit` received messages.", "");
        opts.optopt("", "will-message", "Message for the client Will", "");
        opts.optopt("", "will-topic", "Topic for the client Will", "");
        opts.optopt("", "will-qos", "QoS level for the client Will", "");
        opts.optopt("", "will-retain", "Make the client Will retained", "");

        opts.optopt("", "tls", "Enables TLS and sets protocol version. Can be tlsv1, tlsv1.1, tlsv1.2", "");
        opts.optopt("", "cafile", "Specifies the file that contains trusted CA certificates.", "file");
        //opts.optopt("", "capath", "TODO", "path");
        opts.optopt("", "key", "Path to private key", "path");
        opts.optopt("", "cert", "Path to certificate", "path");
        opts.optflag("", "no-verify", "Disables client cert requests");

        opts.optflag("h", "help", "Display this message");

        let matches = match opts.parse(&self.arguments[..]) {
            Ok(m) => { m }
            Err(f) => {
                self.cli_error(f.to_string());
            }
        };

        if matches.opt_present("h") {
            self.subscribe_print_usage(opts);
            exit(0);
        };

        let address = matches.opt_str("a").unwrap_or(default.address);
        let port = if matches.opt_present("p") {
            match matches.opt_str("p").unwrap().parse::<u16>() {
                Ok(v) => v,
                Err(_) => {
                    self.cli_error("port format error");
                }
            }
        } else {
            default.port
        };
        let client_id = matches.opt_str("i");
        let clean_session = matches.opt_present("c");
        let username = matches.opt_str("u");
        let password = matches.opt_str("P");
        let topic_filters = Vec::new(); // TODO: matches.opt_strs("f");
        let limit = None; /*TODO: if matches.opt_present("limit") {
            match matches.opt_str("limit").unwrap().parse::<u32>() {
                Ok(v) => Some(v),
                Err(_) => {
                    self.cli_error("limit format error");
                }
            }
        } else {
            None
        };*/
        let will_topic = matches.opt_str("will-topic");
        let will_message = matches.opt_str("will-message");
        let will_qos = matches.opt_str("will-qos");
        let will_retain = matches.opt_present("will-retain");
        let debug = matches.opt_present("d");
        let silence = matches.opt_present("s");
        let keep_alive = if matches.opt_present("k") {
            match matches.opt_str("k").unwrap().parse::<u16>() {
                Ok(v) => v,
                Err(_) => {
                    self.cli_error("keep alive format error");
                }
            }
        } else {
            default.keep_alive
        };
        let reconnect = matches.opt_present("r");
        let protocol = if matches.opt_present("v") {
            match matches.opt_str("v").unwrap().as_ref() {
                "3.1" => Protocol::MQIsdp(3),
                "3.1.1" => Protocol::MQTT(4),
                _ => {
                    self.cli_error("unsupported protocol version");
                }
            }
        } else {
            default.protocol
        };
        let log_file = None; // TODO: matches.opt_str("l");
        let last_will = if will_topic.is_some() && will_message.is_some() {
            Some(LastWill {
                topic: will_topic.unwrap(),
                message: will_message.unwrap(),
                qos: will_qos.map_or(QoS::AtMostOnce, |s| self.parse_qos(s)),
                retain: will_retain
            })
        } else {
            if !will_topic.is_none() || !will_topic.is_none() {
                self.cli_error("both will-topic and will-message required");
            };
            None
        };
        let cafile = matches.opt_str("cafile");
        let key = matches.opt_str("key");
        let cert = matches.opt_str("cert");
        let ssl_method = if matches.opt_present("tls") {
            match matches.opt_str("tls").unwrap().as_ref() {
                "1" => Some(SslMethod::Tlsv1),
                "1.1" => Some(SslMethod::Tlsv1_1),
                "1.2" => Some(SslMethod::Tlsv1_2),
                _ => {
                    self.cli_error("unsupported TLS version")
                }
            }
        } else {
            None
        };
        let verify_mode = if matches.opt_present("no-verify") {
            SslVerifyMode::from_bits_truncate(0)
        } else {
            SslVerifyMode::from_bits_truncate(1)
        };

        let ssl_context = ssl_method.map(|ssl| {
            let mut context = SslContext::new(ssl).unwrap();
            context.set_verify(verify_mode, None);
            if let Some(ref cafile_path) = cafile {
                context.set_CA_file(cafile_path).unwrap();
            }
            if let Some(ref key_path) = key {
                context.set_private_key_file(key_path, X509FileType::PEM).unwrap();
            }
            if let Some(ref cert_path) = cert {
                context.set_certificate_file(cert_path, X509FileType::PEM).unwrap();
            }
            context
        });

        let retain = false; //TODO: !matches.opt_present("no-retain");

        let qos = matches.opt_str("q").map_or(QoS::ExactlyOnce, |s| self.parse_qos(s));
        let topics = if !matches.free.is_empty() {
            matches.free.iter().map(|topic| SubscribeTopic { topic_path: topic.clone(), qos: qos} ).collect()
        } else {
            default.topics.iter().map(|topic| SubscribeTopic { topic_path: topic.topic_path.clone(), qos: qos} ).collect()
        };

        SubscribeCommand {
            topics: topics,
            address: address,
            port: port,
            clean_session: clean_session,
            last_will: last_will,
            log_file: log_file,
            debug: debug,
            silence: silence,
            keep_alive: keep_alive,
            reconnect: reconnect,
            protocol: protocol,
            client_id: client_id,
            username: username,
            password: password,
            limit: limit,
            retain: retain,
            topic_filters: topic_filters,
            ssl_context: ssl_context
        }
    }

    fn print_usage(&self) {
        let mut brief = "mqttc is a simple MQTT client that provides to publish message or subscribe to topics.\n\n".to_string();
        brief = brief + format!("Usage:\n    {} command\n    {} --help\n\n", self.program, self.program).as_str();
        brief = brief +         "Commands:\n";
        brief = brief +         "    publish/pub \tPublish message to a topic\n";
        brief = brief +         "    subscribe/sub \tSubscribe to topics\n\n";
        print!("{}", brief);
    }

    pub fn publish_print_usage(&self, opts: Options) {
        let brief = format!("Usage: {} publish [OPTIONS]", self.program);
        print!("{}", opts.usage(&brief));
    }

    pub fn subscribe_print_usage(&self, opts: Options) {
        let brief = format!("Usage: {} subscribe [OPTIONS] [TOPICS...]", self.program);
        print!("{}", opts.usage(&brief));
    }

    fn parse_qos(&self, s: String) -> QoS {
        match s.parse::<u8>() {
            Ok(v) => {
                match QoS::from_u8(v) {
                    Ok(qos) => qos,
                    Err(_) => {
                        self.cli_error("unsupported qos value");
                    }
                }
            },
            Err(_) => {
                self.cli_error("qos format error");
            }
        }
    }

    fn cli_error<M: AsRef<str>>(&self, msg: M) -> ! {
        println!("{}: {}", self.program, msg.as_ref());
        exit(64); // command line usage error
    }
}
