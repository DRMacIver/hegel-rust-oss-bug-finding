# hecs PBT testbed — state of play

Goal: property-based-test hecs to an unreasonable degree, as a testbed for developing
reusable stateful-model PBT technique with hegel. Technique is codified as the
`stateful-model-based-testing` skill (../.claude/skills/).

## Layout
- `src/lib.rs` — core stateful model-based harness: `drive()` + cfg-gated normal/miri entry points.
- `tests/serialize_roundtrip.rs` — row + column serialize→deserialize round-trip.
- `tests/command_buffer.rs` — CommandBuffer (buffered run_on vs eager direct application).
- `tests/change_tracker.rs` — ChangeTracker added/changed/removed vs a model, exact semantics.
- `tests/column_batch.rs` — ColumnBatch / spawn_column_batch(_at) vs one-by-one spawn.
- `tests/serialize_errors.rs` — serialize edge/error paths: truncated/corrupted input safety,
  malformed-stream validation, serialize_satisfying, Drop-tracked leak hunt.

## Run
- `cargo test` — full suite (14 tests across 5 files; core harness ~18s).
- `cargo +nightly miri test --lib` with
  `MIRIFLAGS="-Zmiri-permissive-provenance -Zmiri-disable-isolation"` — UB hunt (~32s).
  `-Zmiri-disable-isolation` is REQUIRED: hegel reads its test-case DB from disk.

## Coverage of hecs (cargo-llvm-cov --dep-coverage hecs)
- Core harness alone (`--lib`): 32.9% → **57%** region coverage after the breadth push.
- Full suite (`--tests`): 32.9% → **81.3%** region coverage. Per-file highlights: command_buffer
  100%, take 100%, world.rs 91%, change_tracker 97%, query_one 91%, batch 87%, serialize/row 83%,
  serialize/column 83%, entities 83%, archetype 82%.
- NOTE: hecs 0.11.0 has NO par_iter/rayon feature (only row-serialize/column-serialize/std) —
  that path does not exist to cover. Remaining gaps are DynamicBundleClone (bundle.rs 66%),
  some EntityRef/EntityBuilder methods, and obscure Fetch impls.

## Operations exercised (core harness; fixed universe A(i32), B(i32), C(ZST), D(Drop-tracked))
spawn(arbitrary subset) · despawn · insert_one · remove_one · insert(bundle) ·
remove::<(A,B)>/(C,D) (all-or-nothing) · exchange_one · get::<&mut> mutation ·
query_mut sweep · query_disjoint_mut / view_mut get_disjoint_mut (distinct handles) ·
clear · spawn_batch · reserve::<T> · reserve_entity / reserve_entities(bulk) · flush ·
take (drop path) · take-and-migrate into a scratch world (TakenEntity::put move path) ·
EntityBuilderClone spawn · builder introspection (has/get/component_types).
All target a handle pool that includes stale/despawned/reserved handles.

## Oracle families (checked after every op)
- bidirectional model equivalence (HashMap<Entity, M>) + `len`
- per-op ok/err vs model
- Drop leak/double-drop oracle (live-D count == modelled D count)
- archetype structural invariants (partition; Σ archetype.len == world.len; each id once)
- query correctness: &A, (&A,&B), With, Without, Or, Option, query_one, query_one_mut,
  satisfies, EntityRef view (has/get/len/component_types/query), View get/contains,
  iter_batched (== flat iter), PreparedQuery (== fresh query)
- reserved-state: reserved handles contained() but excluded from len/iter/queries/model
- serialize round-trip (row + column), exact Entity-handle preservation

## Findings (all confirmed on 0.11.0 AND master; no newer release; no duplicate issues;
## hecs actively maintained — last commit 2026-06-10; no AI policy)
Per-owner cap (max 2 to Ralith) reached — FILED 2026-07-27:
  - Finding A (UB): https://github.com/Ralith/hecs/issues/449
  - Finding B (leak): https://github.com/Ralith/hecs/issues/450
Findings C and D held (would exceed the cap).

- **Finding A (memory safety — strongest): `spawn_column_batch_at` with a duplicate handle
  → subtract-overflow panic (debug) / out-of-bounds write at index u32::MAX (release UB).**
  Reachable through the SAFE `column::deserialize` API: `visit_seq` passes the stream's raw
  entity-id list to `spawn_column_batch_at` with no dedup, so deserializing untrusted/corrupt
  column data whose id list repeats an id triggers it. Root cause: the second `alloc_at(id)`
  takes the "id already live" branch and returns the EMPTY sentinel location {archetype:0,
  index:u32::MAX}; hecs then calls `Archetype::remove(u32::MAX, true)` → `self.len - 1`
  underflow. Draft: `../draft-reports/hecs-column-deserialize-duplicate-id-ub.md`. Verified by
  a standalone repro (debug panic at archetype.rs:321).
- **Finding B (leak): `ColumnBatchBuilder` leaks its written components** when dropped without
  a successful `build()` — plain drop AND `build()`→`Err(BatchIncomplete)`. Also reachable via
  `column::deserialize` of truncated data (leaks every component already parsed into the
  internal builder). Draft: `../draft-reports/hecs-columnbatchbuilder-leak.md`. Root cause:
  `ColumnBatchBuilder::drop` steps a `*mut u8` by byte-index and calls `drop_in_place::<u8>`
  (no-op); `build()` moves the archetype out (len 0) before the completeness check.
- **Finding C (panic, lower severity): column `entity_count == u32::MAX`** trips
  `assert!(size < u32::MAX)` in `ColumnBatchType::into_batch` — panic instead of `Err` on
  malformed input (after `Vec::reserve`-ing ~34GB from the untrusted count). No separate draft
  yet; candidate to fold into a deserialize-hardening report.
- **Finding D (analysis, not filed): unbounded allocation** — both deserializers trust
  attacker-controlled ids/counts before validation (row `spawn_at` grows metadata to the id;
  column `entity_count` drives reserve), so one corrupted high byte can commit tens of GB.
- Everything else passes at scale: core model-vs-map + per-op + Drop + archetype + all query
  shapes (incl. spawn_at, disjoint-mut, PreparedView) + reserved-state + serialize round-trip +
  CommandBuffer + ChangeTracker + column-batch happy paths + Miri (UB-clean), all clean.
- Infra notes: hegel under Miri requires -Zmiri-disable-isolation; the heavy harness trips
  hegel's TooSlow health check under Miri (suppressed for the cfg(miri) entry point only).

## Possible next steps
- Close remaining coverage: enable `parallel` feature to test `par_iter`; QueryOne
  with/without; PreparedView; serialize of more component universes + error paths.
- Run the new integration tests under Miri too (currently only the core harness is Miri-gated).
- Apply the same harness/skill to the "unreasonable degree" scale-up target (redb).
