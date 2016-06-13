extern crate mqttc;
extern crate netopt;
#[macro_use] extern crate log;
extern crate env_logger;

use std::env;
use std::process::exit;
use std::time::Duration;
use netopt::NetworkOptions;
use mqttc::{PubSub, Client, ClientOptions, ReconnectMethod, PubOpt};

fn main() {
    env_logger::init().unwrap();
    let mut args: Vec<_> = env::args().collect();
    if args.len() < 4 {
        println!("Usage: RUST_LOG=main,mqttc cargo run --example pub -- 127.0.0.1:1883 a/b/c \"a message\"");
        exit(0);
    }
    let ref address = args[1];
    let ref topic = args[2];
    let ref message = args[3];
    // Connect to broker, send CONNECT then wait CONNACK
    let netopt = NetworkOptions::new();
    let mut opts = ClientOptions::new();
    opts.set_keep_alive(15);
    opts.set_reconnect(ReconnectMethod::ReconnectAfter(Duration::new(5,0)));
    let mut client = opts.connect(address.as_str(), netopt).unwrap();

    client.publish(topic.as_str(), message.as_str(), PubOpt::at_most_once()).unwrap();
    client.publish(topic.as_str(), message.as_str(), PubOpt::at_least_once()).unwrap();
    client.await().unwrap();
}
