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

## Run
- `cargo test` — full suite (14 tests across 5 files; core harness ~18s).
- `cargo +nightly miri test --lib` with
  `MIRIFLAGS="-Zmiri-permissive-provenance -Zmiri-disable-isolation"` — UB hunt (~32s).
  `-Zmiri-disable-isolation` is REQUIRED: hegel reads its test-case DB from disk.

## Coverage of hecs (cargo-llvm-cov --dep-coverage hecs)
- Core harness alone (`--lib`): 32.9% → **54.2%** region coverage after the breadth push.
- Full suite (`--tests`): **77.8%** region coverage. Per-file highlights: command_buffer 100%,
  take 100%, world.rs 91%, change_tracker 97%, entities 82%, archetype 82%, batch 72%.
- Remaining gaps are mostly feature-gated (rayon `par_iter`), plus QueryOne with/without
  combinators, PreparedView, and some serialize error paths.

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

## Findings
- **BUG (reportable): `ColumnBatchBuilder` leaks its written components** when the builder
  is dropped without a successful `build()` — both on plain drop and when `build()` returns
  `Err(BatchIncomplete)`. Confirmed on 0.11.0 and on master; no newer release; no duplicate
  issue; hecs actively maintained (last commit 2026-06-10, no AI policy). Draft at
  `../draft-reports/hecs-columnbatchbuilder-leak.md`. Root cause (for us, not the report):
  `ColumnBatchBuilder::drop` steps a `*mut u8` by byte-index and calls `drop_in_place::<u8>`
  (a no-op), and `build()` moves the archetype out (len 0) before the completeness check so
  the `Err` path drops nothing. The happy path (full build → spawn → World drop) is correct.
- Everything else passes at scale: core model-vs-map + per-op + Drop + archetype + all query
  shapes + reserved-state + serialize + CommandBuffer + ChangeTracker + column-batch happy
  paths + Miri, all clean.
- Infra note: hegel under Miri requires -Zmiri-disable-isolation.

## Possible next steps
- Close remaining coverage: enable `parallel` feature to test `par_iter`; QueryOne
  with/without; PreparedView; serialize of more component universes + error paths.
- Run the new integration tests under Miri too (currently only the core harness is Miri-gated).
- Apply the same harness/skill to the "unreasonable degree" scale-up target (redb).
