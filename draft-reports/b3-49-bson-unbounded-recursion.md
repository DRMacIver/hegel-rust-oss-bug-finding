# Deeply nested BSON documents crash the process with a stack overflow when decoded

```rust
fn nested_bson_bytes(depth: usize) -> Vec<u8> {
    let mut doc: Vec<u8> = vec![5, 0, 0, 0, 0]; // empty document
    for _ in 0..depth {
        let mut body = vec![0x03]; // document-typed element
        body.extend_from_slice(b"d\0");
        body.extend_from_slice(&doc);
        body.push(0x00);
        let mut new_doc = ((4 + body.len()) as i32).to_le_bytes().to_vec();
        new_doc.extend_from_slice(&body);
        doc = new_doc;
    }
    doc
}

fn main() {
    let bytes = nested_bson_bytes(10_000);
    let _: bson::Document = bson::deserialize_from_slice(&bytes).unwrap();
}
```

Output:

```
thread 'main' has overflowed its stack
fatal runtime error: stack overflow, aborting
```

`Document::from_reader` crashes the same way on the same bytes:

```rust
let _ = bson::Document::from_reader(bytes.as_slice()).unwrap();
```

```
thread 'main' has overflowed its stack
fatal runtime error: stack overflow, aborting
```

The input here is just a document nested 10,000 levels deep under one field name, a valid BSON byte layout, and it takes the process down before either function returns a `Result`. Both crash in debug and release builds.

Tested on bson 3.1.0 (`serde` feature).

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
