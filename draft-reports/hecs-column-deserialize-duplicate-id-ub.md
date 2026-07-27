<!-- FILED 2026-07-27 as https://github.com/Ralith/hecs/issues/449 -->

# `spawn_column_batch_at` panics or writes out of bounds when `handles` repeats an entity

Passing a `handles` slice containing the same entity twice to `spawn_column_batch_at` triggers a subtraction overflow on a debug build and an out-of-bounds write on a release build:

```rust
use hecs::{ColumnBatchType, Entity, World};

fn main() {
    let mut world = World::new();
    // A column batch of two entities in the unit (no-component) archetype.
    let batch = ColumnBatchType::new().into_batch(2).build().unwrap();

    let e = Entity::from_bits(1 << 32).unwrap(); // id 0, generation 1
    world.spawn_column_batch_at(&[e, e], batch); // the same handle twice
}
```

On a debug build this panics:

```
thread 'main' panicked at src/archetype.rs:321:20:
attempt to subtract with overflow
```

On a release build there is no panic; `remove` is called with index `u32::MAX`.

This is reachable from safe deserialization: `column::deserialize` collects the entity ids from the stream and passes them straight to `spawn_column_batch_at` without checking for duplicates, so deserializing a column-format stream whose entity-id list repeats an id hits the same path. Deserializing untrusted or corrupted data can therefore trigger it. Here is a stream with two entities sharing one id, built with the deserialize context from the crate's own column-serialize example:

```rust
// Given a `DeserializeContext` `Ctx` for a componentless archetype:
// stream = [archetype_count=1, entity_count=2, component_count=0, id=1<<32, id=1<<32]
let bytes = /* the 4-tuple sequence above, bincode-encoded */;
let mut de = bincode::Deserializer::from_slice(&bytes, bincode::options());
let _ = column::deserialize(&mut Ctx::default(), &mut de); // panics / UB, rather than Err
```

I would have expected duplicate ids to be rejected (an `Err` from `deserialize`, or a documented panic from `spawn_column_batch_at`) rather than an out-of-bounds access.

Tested on hecs 0.11.0 and on current `master` (`column-serialize` feature for the deserialize path).

BTW, this was found with [hegel](https://crates.io/crates/hegeltest) while property-testing hecs; happy to contribute the tests if useful.
