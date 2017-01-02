extern crate mqttc;
#[macro_use] extern crate log;
extern crate env_logger;
extern crate url;

use std::env;
use std::process::exit;
use std::time::Duration;
use mqttc::{ClientOptions, ReconnectMethod};
use url::Url;

fn main() {
    env_logger::init().unwrap();
    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: cargo run --example ping -- 127.0.0.1:1883");
        exit(0);
    }
    let ref address = Url::parse(&args[1]).unwrap();
    info!("Display logs");
    println!("Establish connection to {}", address);

    // Connect to broker, send CONNECT then wait CONNACK
    let mut opts = ClientOptions::new();
    opts.set_keep_alive(15);
    opts.set_reconnect(ReconnectMethod::ReconnectAfter(Duration::new(5,0)));
    let mut client = opts.connect(address).unwrap();

    loop {
        match client.await().unwrap() {
            Some(message) => println!("{:?}", message),
            None => {
                println!(".");
            }
        }
    }
}
