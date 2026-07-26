# `OpLog::load_from` panics on corrupted/malformed bytes

```rust
use diamond_types::list::{ListCRDT, OpLog};
use diamond_types::list::encoding::{EncodeOptions, ENCODE_FULL};

fn main() {
    let mut doc = ListCRDT::new();
    let agent = doc.get_or_create_agent_id("agent 0");
    doc.insert(agent, 0, "\0");
    let bytes = doc.oplog.encode(EncodeOptions { store_deleted_content: true, ..ENCODE_FULL });

    for i in 0..=bytes.len() {
        for b in [0u8, 2, 0xff] {
            let mut corrupted = bytes.clone();
            corrupted.splice(i..i, [b]);
            let result = std::panic::catch_unwind(|| {
                drop(OpLog::load_from(&corrupted));
            });
            if result.is_err() {
                println!("panicked at splice position {} with byte {}", i, b);
                return;
            }
        }
    }
    println!("no panic found");
}
```

```
thread 'main' panicked at src/rle/rle_vec.rs:152:39:
called `Option::unwrap()` on a `None` value
panicked at splice position 43 with byte 2
```

`OpLog::load_from` is documented to return `Result<Self, ParseError>`, but splicing a single extra byte into an otherwise valid encoded oplog at various positions makes it panic instead. This happens on both debug and release builds.

Tested on diamond-types 1.0.0.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
