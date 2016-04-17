# RustMQ

# Client

## Features

* QoS 0, QoS 1, QoS 2 publish/subscribe
* Last Will message
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

```bash
git clone https://github.com/inre/rust-mq.git
cd rust-mq
make
make install
mqttc sub
mqttc pub -t a/b/c -m "hello"
```

# Server

Maybe in future
