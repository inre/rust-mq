extern crate mqtt3;

use std::env;
use std::net::TcpStream;
use std::io::{Write, BufReader, BufWriter};
use std::process::exit;
use mqtt3::{MqttRead, MqttWrite, Packet, Connect, Subscribe, Protocol, QoS, PacketIdentifier};

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: cargo run --example mqtt3_sub -- 127.0.0.1:1883");
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
        client_id: "rust-mq-example-sub".to_owned(),
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

    // SUBSCRIBE
    let subscribe = Packet::Subscribe(Box::new(Subscribe {
        pid: PacketIdentifier(260),
        topics: vec![
            mqtt3::SubscribeTopic { topic_path: "/a/b".to_owned(), qos: QoS::ExactlyOnce }
        ]
    }));
    println!("{:?}", subscribe);
    writer.write_packet(&subscribe).unwrap();
    writer.flush().unwrap();
    let packet = reader.read_packet().unwrap();
    println!("{:?}", packet);

    loop {
        let packet = reader.read_packet().unwrap();
        println!("{:?}", packet);

        // PUBACK
        match packet {
            Packet::Publish(publish) => {
                if publish.qos == QoS::AtLeastOnce {
                    if let Some(pid) = publish.pid {
                        let packet = Packet::Puback(pid);
                        println!("{:?}", packet);
                        writer.write_packet(&packet).unwrap();
                        writer.flush().unwrap();
                    }
                }
            },
            _ => ()
        }
    }
}
