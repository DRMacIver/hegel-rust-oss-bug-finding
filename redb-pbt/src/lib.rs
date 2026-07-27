//! Property-based-testing testbed for redb (embedded KV store) — stateful model-based
//! testing plus MVCC snapshot-isolation checks, as a substrate for developing new PBT
//! techniques (the frontier here is crash/power-loss injection via a custom
//! `StorageBackend`, and multi-reader MVCC snapshot consistency, since redb's own
//! libFuzzer harness already covers single-threaded model-vs-BTreeMap with
//! savepoints/durability/repair).
//!
//! Modules (all test-only):
//!   * `common` — table definition, generators, whole-table snapshot + check oracles.
//!   * `model`  — baseline: model = `BTreeMap<u64,u64>`, write-txn batches
//!     (insert/remove) with commit/abort under drawn durability; oracles for full
//!     contents, point gets, len, per-op return values; plus a snapshot-isolation seed
//!     check (a read txn opened before a commit keeps seeing the pre-commit snapshot).
//!   * `crash`  — TECHNIQUE: crash/power-loss injection. A `CrashBackend` separates
//!     "written" from "fsynced" bytes; at a drawn point we revert to the last-fsync
//!     image, reopen, and require recovery to a committed snapshot no older than the
//!     last durable commit.

#[cfg(test)]
mod common;
#[cfg(test)]
mod crash;
#[cfg(test)]
mod metamorphic;
#[cfg(test)]
mod model;
#[cfg(test)]
mod multimap;
#[cfg(test)]
mod mvcc;
#[cfg(test)]
mod savepoints;
