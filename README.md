# RustMQ

In progress...

# Client

## Installation

```bash
git clone https://github.com/inre/rust-mq.git
cd rust-mq
make
make install
mqttc sub -h
```
## Features

* QoS 0, QoS 1, QoS 2 publish/subscribe
* Last Will message
* Auto-Reconnect
* SSL supported (include TLS v1.1, TLS v1.2)
* Modular: mqtt3, netopt
* Logging
