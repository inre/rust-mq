extern crate mqtt3;

use std::env;
use std::net::TcpStream;
use std::io::{Write, BufReader, BufWriter};
use std::process::exit;
use mqtt3::{MqttRead, MqttWrite, Packet, Connect, Publish, Protocol, QoS, PacketIdentifier};
use std::sync::Arc;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: cargo run --example mqtt3_pub -- 127.0.0.1:1883");
        exit(1);
    }
    let ref address = args[1];
    println!("Establish connection to {}", address);
    let stream = TcpStream::connect(address.as_str()).unwrap();
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut writer = BufWriter::new(stream.try_clone().unwrap());

    // CONNECT -> CONNACK
    let connect = Packet::Connect(Box::new(Connect {
        protocol: Protocol::MQTT(4),
        keep_alive: 30,
        client_id: "rust-mq-example-pub".to_owned(),
        clean_session: true,
        last_will: None,
        username: None,
        password: None
    }));
    println!("{:?}", connect);
    writer.write_packet(&connect).unwrap();
    writer.flush().unwrap();
    let packet = reader.read_packet().unwrap();
    println!("{:?}", packet);

    // PUBLISH
    let publish = Packet::Publish(Box::new(Publish {
        dup: false,
        qos: QoS::AtLeastOnce,
        retain: false,
        topic_name: "/a/b".to_owned(),
        pid: Some(PacketIdentifier(10)),
        payload: Arc::new("Hello world".to_string().into_bytes())
    }));
    println!("{:?}", publish);
    writer.write_packet(&publish).unwrap();
    writer.flush().unwrap();
    let packet = reader.read_packet().unwrap();
    println!("{:?}", packet);
}
