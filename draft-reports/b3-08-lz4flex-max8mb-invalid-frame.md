# `BlockSize::Max8MB` produces a frame that `FrameDecoder` itself rejects

```rust
use lz4_flex::frame::{BlockSize, FrameDecoder, FrameInfo};
use std::io::{Read, Write};

fn main() {
    let mut info = FrameInfo::new();
    info.block_size = BlockSize::Max8MB;

    let mut compressed = Vec::new();
    {
        let mut enc = lz4_flex::frame::FrameEncoder::with_frame_info(info, &mut compressed);
        enc.write_all(b"hello world").unwrap();
        enc.finish().unwrap();
    }

    let mut decoder = FrameDecoder::new(&compressed[..]);
    let mut out = Vec::new();
    let result = decoder.read_to_end(&mut out);
    println!("{:?}", result);
}
```

Output:

```
Err(Custom { kind: InvalidData, error: ReservedBitsSet })
```

A frame written with `BlockSize::Max8MB` cannot be read back by `FrameDecoder`; it fails with `ReservedBitsSet` before any data is decoded.

Tested on lz4_flex 0.14.0.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
