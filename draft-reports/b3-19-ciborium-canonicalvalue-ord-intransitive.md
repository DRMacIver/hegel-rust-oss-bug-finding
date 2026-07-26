# `CanonicalValue`'s `Ord` is not transitive

```rust
use ciborium::value::{CanonicalValue, Value};

fn main() {
    let a: CanonicalValue = Value::Tag(0, Box::new(Value::Bytes(vec![0; 10]))).into();
    let b: CanonicalValue = Value::Tag(24, Box::new(Value::Null)).into();
    let c: CanonicalValue = Value::Text("abc".into()).into();

    println!("a < b: {}", a < b);
    println!("b < c: {}", b < c);
    println!("c < a: {}", c < a);
}
```

```
a < b: true
b < c: true
c < a: true
```

All three comparisons return `true`, so `a < b`, `b < c`, and `c < a` all hold at once: the ordering is not transitive.

Tested on ciborium 0.2.2.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
