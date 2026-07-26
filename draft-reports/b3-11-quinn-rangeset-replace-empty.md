# Zero-length STREAM frames bypass `TooManyChunks` and grow the unordered-read assembler's dedup set

`RangeSet` and `Assembler` are crate-internal (`pub(crate)`/`pub(super)`), so these are tests to drop into the crate's own suite rather than an external program.

Added to the `tests` module at the bottom of `quinn-proto/src/range_set/btree_range_set.rs`:

```rust
#[test]
fn replace_empty_range_corrupts_set() {
    let mut set = RangeSet::new();
    assert_eq!(set.replace(0..0).collect::<Vec<_>>(), &[]);
    println!("is_empty() = {:?}", set.is_empty());
    println!("min() = {:?}", set.min());
    println!("len() = {:?}", set.len());
    assert!(set.is_empty());
}
```

Output:

```
is_empty() = false
min() = Some(0)
len() = 1

thread '...' panicked at src/range_set/btree_range_set.rs:380:9:
assertion failed: set.is_empty()
```

Added to the `test` module at the bottom of `quinn-proto/src/connection/assembler.rs`:

```rust
#[test]
fn unordered_empty_frames_grow_recvd_set_unbounded() {
    // Non-empty single-byte frames at gapped offsets are capped by `TooManyChunks`,
    // which exists precisely to bound memory use from many-small-frames attacks.
    let mut non_empty = Assembler::new();
    non_empty.ensure_ordering(false).unwrap();
    let mut err = None;
    for i in 0..10_000u64 {
        // allocation_size reflects a realistic full-datagram allocation behind a 1-byte
        // STREAM frame, as quinn's receive path does.
        if let Err(e) = non_empty.insert(i * 2, Bytes::from_static(b"x"), 1200) {
            err = Some((i, e));
            break;
        }
    }
    let (stopped_at, _) = err.expect("non-empty frames should hit TooManyChunks");
    println!("non-empty frames: TooManyChunks after {stopped_at} inserts");
    assert!(stopped_at < 10_000);

    // The same number of zero-length frames at gapped offsets hit no such limit: each
    // one adds a bookkeeping entry to the `recvd` RangeSet, which nothing ever caps.
    let mut empty = Assembler::new();
    empty.ensure_ordering(false).unwrap();
    for i in 0..10_000u64 {
        empty.insert(i * 2, Bytes::new(), 0).unwrap();
    }
    let len = match &empty.state {
        State::Unordered { recvd } => recvd.len(),
        State::Ordered => unreachable!(),
    };
    println!("empty frames: 10000 inserts accepted, recvd set now holds {len} entries");
    assert_eq!(len, 10_000, "every empty frame left its own permanent entry");
}
```

Output:

```
non-empty frames: TooManyChunks after 1035 inserts
empty frames: 10000 inserts accepted, recvd set now holds 10000 entries

test connection::assembler::test::unordered_empty_frames_grow_recvd_set_unbounded ... ok
```

`RangeSet::replace` on an empty range leaves a phantom zero-length entry in the set instead of leaving it unchanged. A peer sending zero-length STREAM frames at distinct offsets while a stream is being read unordered adds one permanent entry to `recvd` per frame. The same pattern using non-empty frames of the same count is stopped by the existing `TooManyChunks` limit (at 1035 of 10000 attempted here) well before it gets this far.

Tested on quinn-proto 0.11.16.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
