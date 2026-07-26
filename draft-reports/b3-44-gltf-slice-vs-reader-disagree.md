# `Glb::from_slice` and `Glb::from_reader` disagree on the same valid GLB bytes

```rust
use gltf::binary::Glb;

fn main() {
    // 12-byte GLB header + 8-byte chunk header + empty JSON chunk body = 20 bytes,
    // plus one trailing byte after the declared header.length.
    let mut bytes: Vec<u8> = Vec::new();
    bytes.extend_from_slice(b"glTF"); // magic
    bytes.extend_from_slice(&2u32.to_le_bytes()); // version
    bytes.extend_from_slice(&20u32.to_le_bytes()); // total length (header.length)
    bytes.extend_from_slice(&0u32.to_le_bytes()); // chunk length = 0
    bytes.extend_from_slice(b"JSON"); // chunk type
    bytes.push(0xFF); // one trailing byte past header.length

    let from_slice_result = Glb::from_slice(&bytes);
    println!("from_slice: {:?}", from_slice_result);

    let from_reader_result = Glb::from_reader(std::io::Cursor::new(&bytes));
    println!("from_reader: {:?}", from_reader_result.is_ok());
}
```

Output:

```
from_slice: Err(Binary(Io(Error { kind: UnexpectedEof, message: "failed to fill whole buffer" })))
from_reader: true
```

`from_reader` stops reading once it has consumed `header.length` bytes and returns `Ok`, while `from_slice` walks chunks across the whole slice and errors trying to read past the end of it. The two public parsing entry points give different results for the same input bytes.

Tested on `gltf` 1.4.1.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
