# hecs PBT testbed — state of play

Goal: property-based-test hecs to an unreasonable degree, as a testbed for developing
reusable PBT technique with hegel. Techniques are codified as the
`stateful-model-based-testing` and `metamorphic-and-differential-testing` skills
(../.claude/skills/).

## Layout
- `src/lib.rs` — core stateful model-based harness: `drive()` + cfg-gated normal/miri entry points.
- `tests/common/mod.rs` — shared model-free-oracle infrastructure: component universe,
  observational `fingerprint()`, twin-world builder (deterministic replay), Drop oracle.
- `tests/metamorphic.rs` — model-free relations: commuting disjoint ops, insert∘remove and
  exchange round-trip adjusted identities, spawn_batch ≡ loop (incl. partial consumption),
  clear() ≡ fresh world (incl. handle reuse), insert-order independence.
- `tests/differential_paths.rs` — same drawn specs through 5 construction paths (EntityBuilder,
  static tuples over all 16 subsets, reserve+insert, CommandBuffer, incremental insert_one),
  N-way fingerprint equality + spec ground truth + mutation-amplification phase.
- `tests/builder_bundle_api.rs` — coverage-guided builder/bundle/Ref/column API properties
  (bundle query-satisfaction vs spec, clone-builder algebra, add_bundle ≡ individual adds,
  Ref/RefMut map/clone/format, whole-column Archetype::get vs per-entity reads). Found
  findings E and F below.
- `tests/query_shapes.rs` — coverage-guided query-surface properties: every public read shape
  (Satisfies, &mut fetches via query(), Or accessors, QueryBorrow/QueryMut with/without/view/
  into_iter_batched, iterator len/size_hint, ViewIter, ViewBorrow get_mut/get_disjoint_mut/
  get_unchecked, full PreparedView surface) must project the fingerprint exactly.
- `tests/serialize_roundtrip.rs` — row + column serialize→deserialize round-trip.
- `tests/command_buffer.rs` — CommandBuffer (buffered run_on vs eager direct application).
- `tests/change_tracker.rs` — ChangeTracker added/changed/removed vs a model, exact semantics.
- `tests/column_batch.rs` — ColumnBatch / spawn_column_batch(_at) vs one-by-one spawn.
- `tests/serialize_errors.rs` — serialize edge/error paths: truncated/corrupted input safety,
  malformed-stream validation, serialize_satisfying, Drop-tracked leak hunt.

## Run
- `cargo test` — full suite (44 tests across 10 binaries; core harness ~18s, serialize_errors ~12s).
- Miri, with `MIRIFLAGS="-Zmiri-permissive-provenance -Zmiri-disable-isolation"`
  (`-Zmiri-disable-isolation` is REQUIRED: hegel reads its test-case DB from disk):
  - `cargo +nightly miri test --lib` (~32s), and
  - `cargo +nightly miri test --test metamorphic --test differential_paths
     --test serialize_roundtrip --test builder_bundle_api --test query_shapes` (~2.5min) —
    cfg(miri)-gated small configs; the serialize round-trips and builder/column/view paths
    are UB oracles there.

## Coverage of hecs (cargo-llvm-cov --dep-coverage hecs)
- Core harness alone (`--lib`): 32.9% → **57%** region coverage after the breadth push.
- Full suite (`--tests`): 32.9% → 81.3% → **91.2%** region coverage (2026-07-27, after the
  coverage-guided builder/bundle + query-shapes suites). Per-file: command_buffer 100%,
  take 100%, entity_ref 100%, entity_builder 98%, change_tracker 97%, bundle 94%,
  archetype 92%, world.rs 91%, query.rs 91%, query_one 91%, batch 87%, serialize/row 83%,
  serialize/column 83%, entities 83%.
- NOTE: hecs 0.11.0 has NO par_iter/rayon feature (only row-serialize/column-serialize/std) —
  that path does not exist to cover. Remaining gaps: entities.rs edge branches (freelist
  growth races that need concurrent reserve), serialize error-formatting arms, batch.rs
  error paths adjacent to the known findings, and Debug/dangling impls.

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

## New findings (unfiled — per-owner cap reached with #449/#450; confirmed on 0.11.0 only,
## master NOT checked this session (no network); both found 2026-07-27 by
## tests/builder_bundle_api.rs, the second only under Miri)
- **Finding E (memory safety): `From<BuiltEntityClone> for EntityBuilderClone` leaves a
  stale `indices` map** — `build()` sorts `info` by descending alignment without rebuilding
  `indices`, so a round-tripped builder's `get`/`get_mut`/`add` hit the wrong slot. Same-size
  components: silent value corruption/swap via safe code (how the suite caught it). Different
  sizes: `add` does an out-of-bounds read (Miri-confirmed at entity_builder.rs:351), `get` can
  read uninit. Deterministic repro with align-1-then-align-8 components.
  Draft: `../draft-reports/hecs-builtentityclone-stale-indices.md`. In-suite:
  `from_built_entity_clone_stale_indices_observation` pins the current wrong-slot read.
- **Finding F (UB): `EntityBuilderClone::clone` / `BuiltEntityClone::clone` on an empty or
  ZST-only builder calls `alloc` with a zero-size layout** (entity_builder.rs:411) — UB per
  GlobalAlloc's contract, Miri-confirmed on `EntityBuilderClone::new().clone()`. `drop` and
  `grow` both guard zero-size; `clone` doesn't. Draft:
  `../draft-reports/hecs-entitybuilderclone-clone-zero-alloc.md`. In-suite the
  clone-equivalence property is guarded to sized components (loudly commented) so Miri stays
  green; the repro lives in the draft.

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

## Techniques developed here (2026-07-27 session) and their observed value
- **Metamorphic relations** (tests/metamorphic.rs; skill: metamorphic-and-differential-testing):
  cheap to write, no model bookkeeping; pinned subtle documented contracts (clear() handle
  reuse, SpawnBatchIter Drop-drains). Passed clean on hecs — its value here was contract
  pinning + the reusable fingerprint/twin infrastructure.
- **Differential construction paths** (tests/differential_paths.rs): 5 API routes to the same
  spec + mutation amplification; validated hecs's allocation determinism and history-
  unobservability. Clean on hecs; shrinks planted bugs to minimal single-spec cases.
- **Coverage-guided property placement** (tests/builder_bundle_api.rs, tests/query_shapes.rs;
  skill: coverage-guided-property-testing): by far the highest bug yield per effort this
  session — both new findings (E, F) came from the least-covered file within hours, one
  Miri-only. Lesson: after a broad harness plateaus, dependency coverage IS the bug map.
- **Shrinking quality** (3 planted-bug experiments): hegel shrank every plant to the minimal
  spec (single entity, minimal field set), and the fingerprint-diff failure output reads as
  the semantic delta directly. Wrong-relation vs SUT-bug ambiguity is resolved by reading the
  dependency's source, never by weakening.

## Possible next steps
- Fold findings C/D/E/F into filings when the per-owner cap frees up (check master first —
  E/F were confirmed on 0.11.0 only, offline session).
- entities.rs (83%): concurrent reserve_entity from multiple threads (free_cursor CAS paths)
  needs a threaded harness — a new technique opportunity (concurrency PBT).
- serialize/{row,column} remaining arms are mostly error Display/serde plumbing; low value.
- Apply the three skills to the "unreasonable degree" scale-up target (redb).
