<!-- UNFILED (per-owner cap of 2 open AI-assisted issues to Ralith reached with #449/#450).
     Found 2026-07-27 by the builder/bundle API property suite (hecs-pbt/tests/builder_bundle_api.rs).
     Confirmed on hecs 0.11.0 (local registry copy); NOT yet checked against master (no network
     access during this session) — do that before any filing. -->

# Converting `BuiltEntityClone` back into `EntityBuilderClone` leaves a stale component index — wrong-slot reads, and UB on `add`

`EntityBuilderClone::build()` (via `From<EntityBuilderClone> for BuiltEntityClone`) sorts the internal `info` vector by descending alignment, but never rebuilds `indices`, the `TypeId -> info index` map. `BuiltEntityClone` itself is unaffected (spawning iterates `info` only), but the documented round-trip `From<BuiltEntityClone> for EntityBuilderClone` hands the desynchronized state back to a builder, whose `get`/`get_mut`/`add` all resolve components through `indices`. Whenever the sort actually permuted `info` (guaranteed when a lower-alignment component was added before a higher-alignment one), those methods operate on the wrong `(TypeInfo, offset)` slot.

```rust
use hecs::EntityBuilderClone;

#[derive(Clone, Debug, PartialEq)]
struct Small(u8);   // align 1, added first
#[derive(Clone, Debug, PartialEq)]
struct Big(u64);    // align 8, added second — build() sorts it in front

fn main() {
    let mut b = EntityBuilderClone::new();
    b.add(Small(7));
    b.add(Big(0x4242_4242_4242_4242));
    let rebuilt: EntityBuilderClone = b.build().into();
    println!("get::<&Small>() = {:?}", rebuilt.get::<&Small>().map(|s| s.0));
}
```

Output:

```
get::<&Small>() = Some(66)
```

Expected `Some(7)`; the returned value is `0x42`, the first byte of `Big`, read through the stale index. (`get::<&Big>()` on this builder would read `Big`-sized memory from `Small`'s 1-byte slot — uninitialized-memory UB on the same root cause.)

Calling `add` on the round-tripped builder is worse: the occupied-entry path in `Common::add` looks up the stale index, takes the *other* component's `TypeInfo` and offset, drops that component, and copies `wrong_ty.layout().size()` bytes from the caller's new component — an out-of-bounds read when the sizes differ. Miri reports it:

```rust
    let mut b = EntityBuilderClone::new();
    b.add(Small(7));
    b.add(Big(0x4242_4242_4242_4242));
    let mut rebuilt: EntityBuilderClone = b.build().into();
    rebuilt.add(Small(9)); // resolves to Big's slot: drops Big, copies 8 bytes from &Small
```

```
error: Undefined Behavior: memory access failed: attempting to access 8 bytes,
but got alloc7964 which is only 1 byte from the end of the allocation
   --> hecs-0.11.0/src/entity_builder.rs:351:17
    |
351 |                 ptr::copy_nonoverlapping(ptr, storage, ty.layout().size());
```

Without Miri the same program silently corrupts the builder: spawning it afterwards yields `Big == 265` (the `Small` byte plus adjacent stack garbage) while `Small` keeps its old value.

Root cause, all in `src/entity_builder.rs`:

- `From<EntityBuilderClone> for BuiltEntityClone` does `info.sort_unstable_by_key(|y| y.0)` and extends `ids`, leaving `indices` pointing at pre-sort positions.
- `From<BuiltEntityClone> for EntityBuilderClone` only clears `ids`; `Common::get`/`get_mut`/`add` then trust `indices`.

A fix is either to rebuild `indices` after the sort, or to rebuild it in the `BuiltEntityClone -> EntityBuilderClone` conversion.

Note the same-size case (e.g. two `i32` newtypes, when `TypeId` ordering happens to permute them) is silent value corruption through entirely safe code: `add` overwrites the wrong component, `get` returns the wrong component's bytes.
