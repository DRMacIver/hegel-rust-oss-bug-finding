# Writing a v4 `Connect` into a non-empty buffer corrupts the packet

```rust
use rumqttc::mqttbytes::v4::{Connect, Login, Packet};
use bytes::BytesMut;

fn connect_bytes(prefix_len: usize) -> Vec<u8> {
    let mut connect = Connect::new("cid");
    connect.clean_session = true;
    connect.login = Some(Login::new("user", "pass"));
    let mut buf = BytesMut::new();
    buf.extend_from_slice(&vec![0u8; prefix_len]);
    connect.write(&mut buf).unwrap();
    buf[prefix_len..].to_vec()
}

let empty = connect_bytes(0);
let prefixed = connect_bytes(4);
println!("{empty:?}");
println!("{prefixed:?}");
println!("{:?}", Packet::read(&mut BytesMut::from(&prefixed[..]), 10240).err());
```

prints

```
[16, 27, 0, 4, 77, 81, 84, 84, 4, 194, 0, 10, 0, 3, 99, 105, 100, 0, 4, 117, 115, 101, 114, 0, 4, 112, 97, 115, 115]
[16, 27, 0, 4, 77, 194, 84, 84, 4, 2, 0, 10, 0, 3, 99, 105, 100, 0, 4, 117, 115, 101, 114, 0, 4, 112, 97, 115, 115]
Some(TopicNotUtf8)
```

The same `Connect` serialized into an empty buffer and into a buffer already holding 4 bytes produces different packet bytes. In the second case the connect-flags byte (`194`) is written 4 bytes too early — over the `Q` of the `MQTT` protocol string — and the real flags byte is left as the placeholder `2`, so the username/password flags are lost and the frame no longer reads back.

Tested on rumqttc 0.25.1.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
