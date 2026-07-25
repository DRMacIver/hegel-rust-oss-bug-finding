# Encoding an `ObjectIdentifier` whose second arc exceeds 39 yields a different OID

```rust
use rasn::types::ObjectIdentifier;

let oid = ObjectIdentifier::new(vec![0, 40]).unwrap();
let der = rasn::der::encode(&oid).unwrap();
let back: ObjectIdentifier = rasn::der::decode(&der).unwrap();
println!("{:?}", &*back);
```

prints

```
Oid([1, 0])
```

`ObjectIdentifier::new(vec![0, 40])` is accepted, but encoding and decoding it back gives `0.40` → `1.0` — a different OID. `[0, 999]` comes back as `[2, 919]` the same way. When the first arc is 0 or 1 the second arc must be ≤ 39, and values above that alias onto other OIDs through the encoding.

Tested on rasn 0.28.13.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
