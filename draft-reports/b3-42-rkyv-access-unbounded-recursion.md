# `rkyv::access` aborts the process with a stack overflow validating a deeply nested archive

`rkyv::access` is the checked entry point for reading untrusted archives. On a release build it aborts the whole process on a valid, ~1.6 MB archive that is only deeply nested:

```rust
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Serialize, Deserialize)]
#[rkyv(serialize_bounds(__S: rkyv::ser::Writer + rkyv::ser::Allocator, __S::Error: rkyv::rancor::Source))]
#[rkyv(deserialize_bounds(__D::Error: rkyv::rancor::Source))]
#[rkyv(bytecheck(bounds(__C: rkyv::validation::ArchiveContext)))]
enum List { Cons(#[rkyv(omit_bounds)] Box<List>), Nil }

fn main() {
    let mut list = List::Nil;
    for _ in 0..200_000 { list = List::Cons(Box::new(list)); }
    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&list).unwrap(); // ~1.6 MB
    let _ = rkyv::access::<ArchivedList, rkyv::rancor::Error>(&bytes);
}
```

Built with `--release`, it aborts:

```
thread 'main' has overflowed its stack
fatal runtime error: stack overflow, aborting
```

Serialization succeeds and the archive is well-formed — a linear `Cons` chain, no shared or cyclic pointers. `access` overflows the stack and aborts before it can return `Err`, rather than rejecting the input. A shallower archive (e.g. 50,000 levels) validates and returns `Ok`, so the crash is depth-dependent.

(#301 addressed the deserialize path; the checked `access` validation path still aborts.)

Tested on rkyv 0.8.17 (release build).

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
