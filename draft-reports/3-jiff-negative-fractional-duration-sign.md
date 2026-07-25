# Parsing a negative duration whose only non-zero part is fractional drops the sign

```rust
use jiff::{SignedDuration, Span};

let span: Span = "-PT0.5S".parse().unwrap();
let dur: SignedDuration = "-PT0.5S".parse().unwrap();
println!("{span}");
println!("{}", span.is_negative());
println!("{}", dur.as_millis());
```

prints

```
PT0.5S
false
500
```

`-PT0.5S` parses as a positive half-second, for both `Span` and `SignedDuration`, and the `Span` prints back as `PT0.5S`, so it doesn't survive a print/parse round-trip. `-PT1S` round-trips fine, so it only happens when the sole non-zero component is the fraction. `-PT0.000000001S` parses as `+1ns` the same way, and the friendly format behaves the same (`-0.5s` and `0.5s ago` both parse positive).

Tested on jiff 0.2.34 (commit `7311a6a`).

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
