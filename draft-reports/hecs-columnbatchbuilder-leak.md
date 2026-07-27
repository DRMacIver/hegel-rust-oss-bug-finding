<!-- FILED 2026-07-27 as https://github.com/Ralith/hecs/issues/450 -->

# `ColumnBatchBuilder` leaks its written components when dropped without a successful `build()`

Components pushed into a `ColumnBatchBuilder` are never dropped if the builder is dropped without being consumed by a successful `build()`. This program pushes two components, drops the builder, and the two values' destructors never run:

```rust
use std::sync::atomic::{AtomicI64, Ordering};
use hecs::ColumnBatchType;

static LIVE: AtomicI64 = AtomicI64::new(0);

struct Tracked(#[allow(dead_code)] String);
impl Tracked {
    fn new() -> Self {
        LIVE.fetch_add(1, Ordering::SeqCst);
        Tracked("hello".into())
    }
}
impl Drop for Tracked {
    fn drop(&mut self) {
        LIVE.fetch_sub(1, Ordering::SeqCst);
    }
}

fn main() {
    let mut ty = ColumnBatchType::new();
    ty.add::<Tracked>();
    let builder = ty.into_batch(3); // room for 3 entities
    {
        let mut w = builder.writer::<Tracked>().unwrap();
        w.push(Tracked::new()).ok().unwrap();
        w.push(Tracked::new()).ok().unwrap();
    }
    drop(builder); // only 2 of 3 rows written, so the builder is never built
    println!("live Tracked after dropping the builder: {}", LIVE.load(Ordering::SeqCst));
}
```

Output:

```
live Tracked after dropping the builder: 2
```

The two `Tracked` values are still live after the builder is dropped; their `Drop` never runs. Expected output is `0`.

The same happens when a partially-filled builder is finalized with `build()`, which returns `Err(BatchIncomplete)` and leaks whatever was written:

```rust
    let builder = ty.into_batch(3);
    {
        let mut w = builder.writer::<Tracked>().unwrap();
        w.push(Tracked::new()).ok().unwrap();
        w.push(Tracked::new()).ok().unwrap();
    }
    let result = builder.build(); // Err(BatchIncomplete): one row short
    assert!(result.is_err());
    println!("live Tracked after build() returned Err: {}", LIVE.load(Ordering::SeqCst));
```

Output:

```
live Tracked after build() returned Err: 2
```

A fully-filled builder that is consumed by a successful `build()` and spawned into a `World` drops its components correctly; the leak only affects the drop-without-build and the `Err(BatchIncomplete)` paths.

The same leak is reachable from safe deserialization: `column::deserialize` builds its entities through an internal `ColumnBatchBuilder`, so a column-format stream that ends partway through the component data (truncated or corrupted) returns `Err` but leaks every component already deserialized into that builder. Deserializing a one-entity `(Tracked, i32)` column stream with the trailing byte removed leaks the one `Tracked` value.

Tested on hecs 0.11.0 and on current `master`.

BTW, this was found with [hegel](https://crates.io/crates/hegeltest) while property-testing hecs; happy to contribute the tests if useful.
