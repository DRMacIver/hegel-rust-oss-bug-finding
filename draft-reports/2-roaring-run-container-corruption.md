# `remove_smallest` / `remove_biggest` can corrupt a bitmap

```rust
use roaring::RoaringBitmap;

let mut b = RoaringBitmap::new();
b.insert_range(0..=2);
b.insert(4);
b.remove_smallest(3);
```

On a debug build this panics:

```
thread 'main' panicked at src/bitmap/store/interval_store.rs:966:19:
attempt to subtract with overflow
```

On a release build there is no error, but the bitmap is corrupted. After removing the 3 smallest values from `{0, 1, 2, 4}` the result should be `{4}`; instead `b.len()` returns `65537` and iterating the bitmap yields `[3, 4, 5, 6, 7, 8, 9, 10, 11, 12, ...]`.

`remove_biggest` has the same problem, e.g. building `{2, 4, 5, 6, 7, 8}` and calling `remove_biggest(5)`. It happens when the amount removed exactly consumes one of the bitmap's internal runs; `RoaringTreemap` is affected too.

Tested on roaring 0.11.4.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
