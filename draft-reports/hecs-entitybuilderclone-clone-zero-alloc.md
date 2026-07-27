<!-- UNFILED (per-owner cap of 2 open AI-assisted issues to Ralith reached with #449/#450).
     Found 2026-07-27 by running the builder/bundle property suite under Miri
     (hecs-pbt/tests/builder_bundle_api.rs). Confirmed on hecs 0.11.0 (local registry copy);
     NOT yet checked against master (no network access this session) — do that before filing. -->

# Cloning an empty (or ZST-only) `EntityBuilderClone` calls `alloc` with a zero-size layout — UB

`Clone for Common<DynamicClone>` (`src/entity_builder.rs:411`, reached from `EntityBuilderClone::clone` and `BuiltEntityClone::clone`) unconditionally does

```rust
storage: NonNull::new_unchecked(alloc(self.layout)),
```

A fresh builder has `layout = Layout::from_size_align(0, 8)`, and adding only zero-sized components leaves it that way, so cloning such a builder calls `alloc` with a zero-size layout. `GlobalAlloc::alloc` documents that as undefined behavior, and Miri reports it:

```rust
use hecs::EntityBuilderClone;

fn main() {
    let empty = EntityBuilderClone::new();
    let _clone = empty.clone();
}
```

```
error: Undefined Behavior: creating allocation with size 0
   --> hecs-0.11.0/src/entity_builder.rs:411:49
    |
411 |                 storage: NonNull::new_unchecked(alloc(self.layout)),
```

The same happens for a ZST-only builder (`builder.add(Marker)` where `Marker` is zero-sized, then `builder.clone()`).

Even if the allocator happens to return null instead, `NonNull::new_unchecked(null)` is UB in its own right; there is no zero-size guard on this path, unlike `Common::drop` (which checks `self.layout.size() != 0` before `dealloc`) and `Common::grow` (which rounds up to at least 64 bytes). The natural fix is the same guard in `clone`, keeping `NonNull::dangling()` as the storage for zero-size layouts, mirroring `Common::default`.
