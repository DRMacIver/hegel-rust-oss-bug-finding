# `Vector::new_with_bits` accepts values that make the very first `push_back` panic

```rust
fn main() {
    let v: rpds::Vector<i32> = rpds::Vector::new_with_bits(64);
    let v = v.push_back(0);
    println!("{:?}", v.get(0));
}
```

```
thread 'main' panicked at src/vector/mod.rs:387:9:
attempt to shift left with overflow
```

`new_with_bits` only requires `bits > 0`, so `64` is accepted and construction succeeds; the first `push_back` then panics in debug (in release it does not panic). Either `new_with_bits` should reject a `bits` value it can't support, or the resulting `Vector` should be usable — not accept it and then panic on the first push.

Tested on rpds 1.2.1.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
