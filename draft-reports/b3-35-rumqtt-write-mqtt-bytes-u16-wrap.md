# `Publish::write` produces a packet that decodes to a different topic and payload when the topic is 65536 bytes or longer

```rust
use bytes::BytesMut;
use rumqttc::mqttbytes::v4::{Packet, Publish};
use rumqttc::mqttbytes::QoS;

fn main() {
    let topic = "a".repeat(65536);
    let payload = b"hello".to_vec();

    let publish = Publish::new(topic.clone(), QoS::AtMostOnce, payload.clone());

    let mut buffer = BytesMut::new();
    let written = publish.write(&mut buffer).unwrap();
    println!("write() returned Ok({})", written);
    println!("buffer len = {}", buffer.len());

    let decoded = Packet::read(&mut buffer, 1_000_000_000).unwrap();
    let decoded = match decoded {
        Packet::Publish(p) => p,
        other => panic!("expected Publish, got {:?}", other),
    };

    println!("original topic len = {}", topic.len());
    println!("decoded topic = {:?}", decoded.topic);
    println!("decoded payload len = {}", decoded.payload.len());
    println!(
        "decoded payload starts with = {:?}",
        &decoded.payload[..20.min(decoded.payload.len())]
    );
}
```

```
write() returned Ok(65547)
buffer len = 65547
original topic len = 65536
decoded topic = ""
decoded payload len = 65541
decoded payload starts with = [97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 97]
```

`Publish::write` returns `Ok(65547)` with no error for a topic of 65536 bytes, but the packet it writes decodes back to an empty topic, with the topic bytes now sitting at the start of the payload (`97` is `b'a'`) followed by the real payload. Reducing the topic to 65535 bytes round-trips correctly. A topic that can't be encoded should be rejected with an error rather than written as a packet that doesn't round-trip.

Tested on rumqttc 0.25.1.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
