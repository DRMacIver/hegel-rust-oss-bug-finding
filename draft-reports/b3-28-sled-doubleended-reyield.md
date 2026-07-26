# Mixing `next()` and `next_back()` on the same `Iter` re-yields every key

The following program opens a fresh database, inserts 6 keys, then alternates `next()` and `next_back()` on a single `iter()`:

```rust
fn main() {
    let dir = std::env::temp_dir().join(format!("sled-repro-{}", std::process::id()));
    let db = sled::open(&dir).unwrap();

    for i in 0u32..6 {
        db.insert(i.to_be_bytes(), b"v").unwrap();
    }

    let mut iter = db.iter();
    let mut seen = Vec::new();
    let mut from_front = true;
    while let Some(item) = if from_front { iter.next() } else { iter.next_back() } {
        let (k, _) = item.unwrap();
        seen.push(u32::from_be_bytes(k.as_ref().try_into().unwrap()));
        from_front = !from_front;
    }

    println!("{:?}", seen);

    drop(db);
    let _ = std::fs::remove_dir_all(&dir);
}
```

Output:

```
[0, 5, 1, 4, 2, 3, 3, 2, 4, 1, 5, 0]
```

Each of the 6 keys is yielded twice instead of once, and the iterator only stops after 12 items. With a larger key count the same pattern holds: alternating `next()`/`next_back()` over N keys yields each key exactly twice before the iterator returns `None`.

Tested on `sled` 1.0.0-alpha.124.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
