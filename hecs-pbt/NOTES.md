# hecs PBT testbed — state of play

Goal: property-based-test hecs to an unreasonable degree, as a testbed for developing
reusable stateful-model PBT technique with hegel. Technique is codified as the
`stateful-model-based-testing` skill (../.claude/skills/).

## Layout
- `src/lib.rs` — core stateful model-based harness: `drive()` + cfg-gated normal/miri entry points.
- `tests/serialize_roundtrip.rs` — row + column serialize→deserialize round-trip.

## Run
- `cargo test` — normal. Core harness: 1000 cases × up to 300 steps (~14s).
- `cargo +nightly miri test --lib` with
  `MIRIFLAGS="-Zmiri-permissive-provenance -Zmiri-disable-isolation"` — UB hunt (~32s).
  `-Zmiri-disable-isolation` is REQUIRED: hegel reads its test-case DB from disk.

## Operations exercised (over a fixed universe A(i32), B(i32), C(ZST), D(Drop-tracked))
spawn(arbitrary subset) · despawn · insert_one · remove_one · reserve_entity · flush · take,
all targeting a handle pool that includes stale/despawned/reserved handles.

## Oracle families (checked after every op)
- bidirectional model equivalence (HashMap<Entity, M>) + `len`
- per-op ok/err vs model
- Drop leak/double-drop oracle (live-D count == modelled D count)
- archetype structural invariants (partition; Σ archetype.len == world.len; each id once)
- query correctness: &A, (&A,&B), With<&A,&B>, Without<&A,&B>
- reserved-state: reserved handles contained() but excluded from len/iter/queries/model
- serialize round-trip (row + column), exact Entity-handle preservation

## Axes status
1. [DONE] Miri-as-oracle — PASSES (hecs UB-clean over generated seqs, incl. reserve/flush/take).
2. [DONE] deferred entities: reserve_entity + flush; stale-generation handles.
3. [DONE] archetype/structural invariants.
4. [PARTIAL] query correctness (4 shapes; Or/Option/query_mut/query_one still open).
5. [DONE] serialize round-trip. TODO: columnar-batch vs individual-spawn path equivalence.
6. [DONE] richer universe (ZST + Drop-tracked) + scaled to 1000×300.

## Findings
- No bugs. hecs is mature and clean: model-vs-map + per-op + Drop + archetype + query +
  reserved-state + serialize + Miri all pass at scale. Value delivered = the reusable
  technique/skill + a high-coverage harness, not a bug (as the recon predicted).
- Infra note: hegel under Miri requires -Zmiri-disable-isolation.

## Possible next steps
- More hecs breadth (Or/Option query shapes, query_mut aliasing, ColumnBatch-vs-spawn
  metamorphic equivalence, CommandBuffer) — diminishing returns; hecs is clean.
- Apply the same harness/skill to the "unreasonable degree" scale-up target (redb):
  its easy model-vs-BTreeMap ground is already covered by its own fuzzer, so the frontier
  is deterministic crash/power-loss simulation (reorder/drop un-synced writes) and MVCC
  1-writer/N-reader linearizability — a bigger design effort worth steering.
