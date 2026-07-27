---
name: crash-injection-testing
description: >
  Property-test a storage engine's crash/power-loss recovery by injecting simulated
  crashes through its storage-backend abstraction: a backend that separates "written"
  from "fsynced" bytes, plus a full write log that lets you cut the stream anywhere
  (including mid-commit) and tear individual writes. Use for any system that offers a
  pluggable storage/file backend and makes durability promises (databases, WALs, KV
  stores, queues). Developed against redb; found nothing there (its recovery held),
  but the technique and its two non-obvious modeling lessons transfer.
---

# Crash / power-loss injection via a fake storage backend

Durability promises ("committed-with-fsync data survives a crash at any point") are
almost never tested by ordinary harnesses, because a real crash kills the process. If
the system takes a pluggable storage backend (redb `StorageBackend`, anything with
`read/write/set_len/sync_data`), you can simulate power loss deterministically, in
process, thousands of times per second — and hegel shrinks the failing schedule.

## The backend

Interior mutability is required when trait methods take `&self`; keep a harness-side
clone of the `Arc` so you can inspect state after handing the backend to the system.

```rust
struct CrashState {
    data: Vec<u8>,      // "OS page cache": every write/set_len lands here
    durable: Vec<u8>,   // "disk": refreshed to data.clone() only in sync_data()
    full_log: Vec<LogEntry>, // COMPLETE stream: Write{offset,data} | SetLen(u64) | Sync
}
#[derive(Clone, Default)]
struct CrashBackend { state: Arc<Mutex<CrashState>> }
```

- `write`/`set_len` mutate `data` AND push a log entry (grow-with-zeros semantics).
- `sync_data()` sets `durable = data.clone()` and pushes a `Sync` marker.
- "Power loss" = build a FRESH backend seeded from a crash image, reopen, check.

## The property

Drive committed write transactions, each under a drawn durability (fsync or not).
Keep `history[j]` = committed model snapshot after commit `j`, plus for each commit
whether it fsynced and the `full_log.len()` when its commit call returned.

After a simulated crash, assert:
1. **Reopen succeeds** (recovery-by-design systems must never fail to open), and
2. **Recovered contents == `history[j]` for some `j >= durable_floor`** — a valid
   committed snapshot no older than the newest fully-completed fsynced commit.
   The "some j >=" form is not a cop-out; it is forced (see lessons below).

Two crash models, cheapest first:

- **Canonical power loss:** crash image = the `durable` bytes. Everything since the
  last fsync is gone. `durable_floor` = last fsynced commit.
- **Torn-write / mid-commit cuts:** crash image = replay of `full_log[..m]` for a
  DRAWN `m` (Sync markers replay as no-ops), optionally followed by the first `k`
  bytes of the next Write (a genuinely torn write). Key insight: under in-order
  persistence, **any prefix of the full stream is a valid crash image** — it is
  exactly "crash at issue point m where every issued-but-unsynced write happened to
  persist". You get mid-commit crashes for free, with no subset/reorder machinery.
  `durable_floor` = newest fsynced commit whose recorded log position <= m.
  (Cut at-or-after the initial schema-creating commit unless you also intend to model
  crash-during-database-creation, which is a separate scenario.)

## Two lessons that will bite you if unlearned

1. **End-of-run crashes explore almost nothing.** Non-durable commits typically live
   in the system's OWN page cache and produce zero backend writes until the next
   fsync — so after the run quiesces, the durable image is exactly the last fsynced
   state, every time. If you only crash after the loop, your torn-write model is a
   no-op. Mid-stream cuts are what make the technique see anything.
2. **Recovery may legitimately land AHEAD of the durable floor.** e.g. a 1-phase
   commit protocol writes the new root/god-byte BEFORE its trailing fsync; a cut
   between them recovers the newer state. So the oracle must be "some committed
   snapshot >= floor", never "== floor" — and equally never "== latest".

## Validate the oracle by mutation, always

A crash property that passes could be vacuous. Prove it has teeth by temporarily
strengthening it and REQUIRING failure:
- assert recovered == latest commit → must fail (non-durable commits get dropped);
- assert recovered == durable floor exactly → must fail under mid-commit cuts
  (commits become visible before their fsync).
If a strengthened oracle does NOT fail, your crash simulation isn't reaching the
states you think it is (that is how lesson 1 above was discovered). Revert after.

## Practical notes

- Read the system's commit protocol first (commit slots, checksums, phase structure)
  so `durable_floor` reflects its actual guarantee, not your guess.
- Keys/values from tiny domains (e.g. u64 in 0..16) keep images small and shrinking
  fast; ~300 cases x ~20 txns runs in a couple of seconds in memory.
- A failure report should include the recovered map, the floor index, and the
  admissible history suffix — that's usually enough to see which commit got lost.
- If reopen fails, that IS the finding (record it); don't catch-and-continue.

## See also

`stateful-model-based-testing` — the committed-history bookkeeping here is that
harness's model, tracked per-commit instead of per-op.
