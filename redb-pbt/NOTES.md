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
- `src/mvcc.rs` — TECHNIQUE 2: MVCC snapshot consistency over interleaved transactions.
  Up to 4 OPEN read txns at once, each tagged with the `history` index at which it was
  opened; interleaved ops = {open reader, write txn (commit → history push, or abort),
  close reader (via `ReadTransaction::close`)}; after EVERY step, every open reader must
  observe exactly `history[its tag]`. Deterministic (no threads) — the concurrency is
  transaction-lifetime overlap, which is what MVCC must get right — and shrinks well.
  Mutation sanity-check: asserting readers see the LATEST state fails immediately, so
  the harness genuinely holds stale snapshots across commits.
- `src/crash.rs` — TECHNIQUE 1: crash/power-loss injection. `CrashBackend`
  (`Arc<Mutex<{data, durable}>>`): writes land in `data`, `sync_data()` copies `data`
  → `durable` (the fsync point). Property: run committed txns under drawn
  `Durability::{None,Immediate}`, tracking `history` (committed snapshot per commit)
  and `last_durable` (index of last Immediate commit); then "power-loss" = seed a
  fresh backend with the `durable` image, reopen, and require (a) reopen succeeds and
  (b) recovered contents == `history[j]` for some `j >= last_durable`. Mutation
  sanity-check: strengthening (b) to "== latest commit" fails immediately (recovered
  `{}` vs a non-durable insert), so the crash model genuinely drops un-fsynced state.
  TORN-WRITE STRETCH (same file): the backend also records the FULL mutation stream
  (`Write`/`SetLen`/`Sync` markers). Key insight: under in-order persistence, ANY
  prefix `F[..m]` of the stream is a valid power-loss image (crash at issue point `m`
  with all issued writes persisted), so cutting anywhere — including MID-COMMIT,
  before an Immediate commit's fsync — plus optionally tearing the next write at a
  drawn byte offset, models torn writes without needing subset/reorder semantics.
  Admissible recovery = any committed snapshot ≥ the newest Immediate commit that
  fully completed (its recorded log position ≤ cut). Two empirical facts the mutation
  checks established: (1) `Durability::None` commits produce NO backend writes by
  quiesce (they live in redb's own page cache), so end-of-run crashes always recover
  exactly the last durable state — mid-stream cuts are ESSENTIAL for the torn model
  to explore anything; (2) with mid-commit cuts, recovery genuinely lands AHEAD of
  the durable floor sometimes (1-phase commit writes the god byte BEFORE its fsync),
  so the "some j ≥ floor" oracle shape is required, not just convenient.

## Run
- `cargo test` — 21 tests (13 hegel properties + 8 deterministic error-contract tests),
  ~11s total.
- Coverage of the redb dependency (`cargo llvm-cov --tests --dep-coverage redb
  --summary-only`): **57.97% → 73.15% region coverage** over this session (functions
  60.8% → 76.1%). Remaining cold: `error.rs` 17.9% (Display arms of hard-to-provoke
  variants: upgrade/poisoned-lock/io), `multimap_table.rs` 52.9% (deep subtree
  reorganisation paths), `db.rs` 67% (repair callback, upgrade, ReadOnlyDatabase).
- redb build/test conventions (for any upstream contribution): `just test`
  (fmt + clippy --all-targets --all-features + test --all-features), `just test_all` for
  workspace crates, `just fuzz` for the libFuzzer harness. Commit authorship = the human;
  add an `Assisted-by: <agent>` trailer (Linux-kernel style), NOT Co-Authored-By.

## Status
- [DONE] baseline substrate (model-vs-BTreeMap + commit/abort + durability + snapshot seed).
- [DONE] crash/power-loss injection StorageBackend (`src/crash.rs`) — passes 300 cases.
- [DONE] MVCC interleaved-snapshot-consistency (`src/mvcc.rs`) — passes 500 cases.
- [DONE] savepoints as model-restore (`src/savepoints.rs`) — ephemeral + persistent
  (persistent adds a model of the savepoint table itself: list/delete/restore-deletes-newer).
- [DONE] metamorphic txn-grouping invariance (`src/metamorphic.rs`).
- [DONE] torn-write crash model (arbitrary in-order cut of the full write stream,
  next write optionally torn mid-write) with drawn 2-phase-commit/quick-repair flags.
- [DONE] multimap tables (`src/multimap.rs`) incl. bulk inserts to force inline→subtree
  spill and reverse iteration (rows and per-key values descending).
- [DONE] coverage-guided pass: `src/surface.rs` (get_mut/pop_first/pop_last/retain_in/
  extract_from_if/first/last/bounded range both directions, wide key domain),
  `src/tables.rs` (catalog: create/delete/rename/list with full rename error contract),
  `src/typed.rs` (tuple/Vec/i64/Option/&[u8]/bool/char/String/[u64;3] encodings must
  iterate in Rust-Ord order and round-trip), `src/errors.rs` (8 provoked-error
  contracts), `src/maintenance.rs` (check_integrity/compact identities + real-file
  FileBackend round-trip). redb region coverage 57.97% → 73.15%.
- [LATER] crash-during-database-creation window; write-reordering (non-prefix subset)
  crash model — needs a page-granularity argument for what redb actually guarantees;
  real-thread reader/writer stress (record-and-check, nondeterministic — keep separate);
  deeper multimap subtree churn (many keys × many values with remove_all).

## Findings
- No redb divergences: every oracle above passes at the stated case counts, including
  power-loss/torn-write recovery under all four commit-protocol combinations
  (1-phase/2-phase × quick-repair on/off). redb is mature and its crash story held up
  exactly as documented. (The only red run all session was a harness bug of ours: the
  multimap reverse-iteration oracle first compared descending values against an
  ascending model — hegel shrank it to a single key with two values instantly.)
- Techniques validated by mutation instead: each major oracle was temporarily
  strengthened to a wrong invariant and REQUIRED to fail (see crash/mvcc module docs);
  all did, so green means tested, not vacuous.

## Skills
- NEW: `.claude/skills/crash-injection-testing/SKILL.md` — the CrashBackend pattern,
  any-prefix-is-a-valid-crash-image insight, the two modeling lessons, and the
  mutation-validation discipline.
- EXTENDED: `stateful-model-based-testing` gained the tagged-snapshot-handles (MVCC)
  oracle family.
