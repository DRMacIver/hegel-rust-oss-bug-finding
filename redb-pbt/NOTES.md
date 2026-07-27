# redb PBT testbed — state of play

Goal: mirror the hecs exploratory project — use redb (embedded KV store, v4.1.0) as a
testbed for developing/​codifying new PBT techniques with hegel. Reuse and extend the
skills from the hecs work (`stateful-model-based-testing`,
`metamorphic-and-differential-testing`, `coverage-guided-property-testing`).

## Recon (2026-07-27)
- redb 4.1.0. Core API: `Database::builder().create_with_backend(InMemoryBackend::new())`
  for fast in-memory PBT; `TableDefinition<K,V>`; `begin_write()` → WriteTransaction
  (`open_table`, `set_durability(Durability::{None,Immediate})`, `ephemeral_savepoint`/
  `persistent_savepoint`/`restore_savepoint`, `commit`/`abort`); `begin_read()` →
  ReadTransaction (consistent snapshot). Tables impl `ReadableTable`
  (get/iter/range/len/insert/remove).
- **redb already ships a mature libFuzzer harness** (`fuzz/fuzz_targets/fuzz_redb.rs`,
  ~1150 lines): model-based (per-savepoint `BTreeMap`), savepoints (Ephemeral /
  NotYetDurablePersistent / Persistent), durability toggling, `quick_repair`, and
  crash-support/reopen. So plain model-vs-BTreeMap adds little on its own.
- **Frontier for NEW techniques** (what the fuzzer does single-threaded / less of):
  1. MVCC snapshot consistency over interleaved transactions (multiple concurrent read
     txns + a writer; every read snapshot must equal some committed state in the linear
     commit history). Deterministic interleaving is more hegel-friendly than real threads.
  2. Crash / power-loss injection via a custom `StorageBackend` that drops/reorders
     un-synced writes at a drawn point, then reopens and checks recovery == last durable
     commit. (redb has some crash-support; a fault-injection backend + model may deepen it.)

## Layout
- `src/lib.rs` — crate docs + test-only module declarations.
- `src/common.rs` — shared plumbing: `TABLE: TableDefinition<u64,u64>`, key/val
  generators, `read_all` (whole-table snapshot), `check` (contents+len+point-gets
  oracle), `create_db` (backend → Database with the table pre-created durably).
- `src/model.rs` — baseline stateful model-based harness: write-txn batches
  (insert/remove) with commit/abort under drawn durability; model = `BTreeMap<u64,u64>`;
  oracles for full contents, point gets, len, per-op return values; plus a
  snapshot-isolation check (a read txn opened before a commit keeps seeing the
  pre-commit snapshot).
- `src/crash.rs` — TECHNIQUE 1: crash/power-loss injection. `CrashBackend`
  (`Arc<Mutex<{data, durable}>>`): writes land in `data`, `sync_data()` copies `data`
  → `durable` (the fsync point). Property: run committed txns under drawn
  `Durability::{None,Immediate}`, tracking `history` (committed snapshot per commit)
  and `last_durable` (index of last Immediate commit); then "power-loss" = seed a
  fresh backend with the `durable` image, reopen, and require (a) reopen succeeds and
  (b) recovered contents == `history[j]` for some `j >= last_durable`. Mutation
  sanity-check: strengthening (b) to "== latest commit" fails immediately (recovered
  `{}` vs a non-durable insert), so the crash model genuinely drops un-fsynced state.

## Run
- `cargo test` — baseline 500 cases × ≤40 txns + crash 300 cases × ≤20 txns (~7s total).
- redb build/test conventions (for any upstream contribution): `just test`
  (fmt + clippy --all-targets --all-features + test --all-features), `just test_all` for
  workspace crates, `just fuzz` for the libFuzzer harness. Commit authorship = the human;
  add an `Assisted-by: <agent>` trailer (Linux-kernel style), NOT Co-Authored-By.

## Status
- [DONE] baseline substrate (model-vs-BTreeMap + commit/abort + durability + snapshot seed).
- [DONE] crash/power-loss injection StorageBackend (`src/crash.rs`) — passes 300 cases.
- [NEXT] MVCC interleaved-snapshot-consistency technique.
- [LATER] torn/partial-write crash model (arbitrary prefix of post-sync writes applied
  before reopen — read redb's 1-phase/2-phase commit + checksum-slot design first),
  savepoints (fuzzer covers; our angle: savepoint == restore-to-earlier-model),
  multimap tables, range/retain/drain query surface, metamorphic (txn grouping invariance).

## Findings
- None yet (baseline passes; redb is mature).
