# `Name::to_ascii` can produce a string that `Name::from_ascii` rejects

```rust
use hickory_proto::rr::Name;

fn main() {
    let name = Name::from_labels(vec![&[0u8][..]]).unwrap();
    let ascii = name.to_ascii();
    println!("to_ascii() = {:?}", ascii);
    let parsed = Name::from_ascii(&ascii);
    println!("from_ascii({:?}) = {:?}", ascii, parsed);
}
```

Output:

```
to_ascii() = "\\000."
from_ascii("\\000.") = Err(Msg("Malformed label: \0"))
```

`Name::from_labels` accepts a label containing a `0x00` byte, and `to_ascii()` renders it as `\000.`, but feeding that exact string back into `Name::from_ascii` fails to parse. Either `from_labels` should reject a label it can't round-trip, or `to_ascii`'s output should parse back through `from_ascii`.

Tested on hickory-proto 0.26.1.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
