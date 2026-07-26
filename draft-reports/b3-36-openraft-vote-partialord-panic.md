# Comparing two `Vote`s with equal terms panics when one has `voted_for: None`

```rust
use openraft::Vote;
use openraft::vote::leader_id_std::LeaderId;

fn main() {
    let a = Vote::<LeaderId<u64, u64>> {
        leader_id: LeaderId {
            term: 5,
            voted_for: None,
        },
        committed: false,
    };
    let b = Vote::<LeaderId<u64, u64>> {
        leader_id: LeaderId {
            term: 5,
            voted_for: Some(1),
        },
        committed: false,
    };

    println!("a = {}", a);
    println!("b = {}", b);
    println!("a < b = {}", a < b);
}
```

It prints

```
a = <T5-NNone:->
b = <T5-N1:->
```

and then panics before the third line:

```
thread 'main' panicked at src/vote/leader_id/leader_id_std.rs:134:33:
called `Option::unwrap()` on a `None` value
```

`Vote` and `LeaderId` expose `term` and `voted_for` as public fields, with no documented restriction against `voted_for: None`. Comparing two votes with equal terms where one side has `voted_for: None` panics; comparing votes with unequal terms works.

Tested on openraft 0.10.0-alpha.30 (`serde` feature).

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
