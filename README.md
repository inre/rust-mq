# RustMQ

[![Build Status](https://travis-ci.org/inre/rust-mq.svg?branch=master)](https://travis-ci.org/inre/rust-mq)

This repository is the bundle of crates devoted to the MQTT protocol.

## Crates

* mqtt3 - MQTT protocol reader/writer ![Crates.io](https://img.shields.io/crates/v/mqtt3.svg)
* mqttc - Rust MQTT client ![Crates.io](https://img.shields.io/crates/v/mqttc.svg)

## Binaries

* mqttc - Console MQTT client

# Client

The client has the following functionality:

* QoS 0, QoS 1, QoS 2 publish/subscribe
* Last Will message
* Auto-Ping
* Auto-Reconnect
* SSL supported (include TLS v1.1, TLS v1.2)
* Modular: mqtt3, netopt
* Logging

## Connect

```rust
let netopt = NetworkOptions::new();
let mut opts = ClientOptions::new();
opts.set_reconnect(ReconnectMethod::ReconnectAfter(Duration::from_secs(1)));
let mut client = opts.connect("127.0.0.1:1883", netopt).expect("Can't connect to server");
```

## Publish

```rust
client.publish("a/b/c", "hello", PubOpt.at_least_once()).unwrap();
while (client.await().unwrap().is_some()) {};
```

## Subscribe

```rust
client.subscribe("a/b/c").unwrap();
loop {
    match client.await().unwrap() {
        Some(message) => {
            println!("{:?}", message);
        },
        None => {}
    }
}
```

## Command line interface

![mqtt-cli](https://cloud.githubusercontent.com/assets/9905/14590517/0aeac094-0505-11e6-9334-eab7067e1842.png)

MQTT Client installation:

```bash
git clone https://github.com/inre/rust-mq.git
cd rust-mq
make && make install
```

Subscribe to all topics:

```bash
mqttc sub
```

Or try,

```bash
mqttc sub -a test.mosquitto.org
mqttc sub -a iot.eclipse.org
mqttc sub -a test.mosca.io
mqttc sub -a broker.hivemq.com
```

Publish to the topic:

```bash
mqttc pub -t a/b/c -m "hello"
```

# Server

Maybe in future
