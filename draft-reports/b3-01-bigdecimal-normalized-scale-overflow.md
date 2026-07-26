# `normalized()` panics on debug and returns a wrong value on release for large negative scales

```rust
use bigdecimal::BigDecimal;
use num_bigint::BigInt;

fn main() {
    let d = BigDecimal::new(BigInt::from(10), i64::MIN);
    println!("input = {}", d);
    let n = d.normalized();
    println!("normalized = {}", n);
}
```

On a debug build this panics:

```
input = 10e+9223372036854775808

thread 'main' panicked at src/lib.rs:926:21:
attempt to subtract with overflow
```

On a release build there is no panic, but the result is wrong:

```
input = 10e+9223372036854775808
normalized = 1E-9223372036854775807
```

`10E+9223372036854775808` and `1E-9223372036854775807` are not the same value; `normalized()` is supposed to only strip trailing zeros while preserving the value.

Tested on bigdecimal 0.4.10.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
