//! TECHNIQUE: crash / power-loss injection via a custom `StorageBackend`.
//!
//! `CrashBackend` keeps two byte images: `data` (what the "OS page cache" holds — every
//! write lands here immediately) and `durable` (what has actually reached "disk" — it is
//! refreshed to a copy of `data` only when redb calls `sync_data()`, i.e. at fsync
//! points). Simulating a power loss = taking the `durable` image, seeding a FRESH
//! backend with it, and reopening the database: everything since the last fsync is gone.
//!
//! Property: drive a sequence of committed write transactions, each under a drawn
//! `Durability` (`Immediate` forces an fsync; `None` does not), keeping `history` (the
//! committed `BTreeMap` snapshot after every commit) and `last_durable`, the history
//! index of the most recent Immediate commit. After a crash:
//!   * reopen must SUCCEED (redb is designed to recover from a crash at any point), and
//!   * the recovered contents must equal `history[j]` for some `j` in
//!     `[last_durable ..= current]` — a valid committed snapshot no older than the last
//!     durable one. redb is free to have persisted-or-not the non-durable commits, so
//!     this invariant is robust by construction.

use crate::common::{create_db, key, read_all, val, TABLE};
use hegel::generators as gs;
use redb::{Database, Durability, StorageBackend};
use std::collections::BTreeMap;
use std::io;
use std::sync::{Arc, Mutex};

/// One event in the backend's full mutation stream, in order.
#[derive(Debug, Clone)]
pub enum LogEntry {
    Write { offset: u64, data: Vec<u8> },
    SetLen(u64),
    /// An fsync point (`sync_data`). No-op on replay; recorded so the harness can
    /// reason about which commits were durable at any cut point.
    Sync,
}

impl LogEntry {
    /// Replay this entry onto a raw byte image, with the same grow-with-zeros
    /// semantics as the live backend.
    pub fn apply(&self, image: &mut Vec<u8>) {
        match self {
            LogEntry::Write { offset, data } => {
                let offset = usize::try_from(*offset).unwrap();
                let end = offset + data.len();
                if end > image.len() {
                    image.resize(end, 0);
                }
                image[offset..end].copy_from_slice(data);
            }
            LogEntry::SetLen(len) => {
                image.resize(usize::try_from(*len).unwrap(), 0);
            }
            LogEntry::Sync => {}
        }
    }
}

#[derive(Debug, Default)]
struct CrashState {
    /// The live byte image: every `write`/`set_len` mutates this ("OS page cache").
    data: Vec<u8>,
    /// The byte image as of the last `sync_data()` call ("what's really on disk").
    durable: Vec<u8>,
    /// The COMPLETE mutation stream since backend creation, including `Sync` markers.
    /// Any prefix of it is a valid in-order power-loss disk image: a crash at issue
    /// point `m` where every issued-but-unsynced write happened to persist. Used by
    /// the torn-write crash model to cut anywhere — including mid-commit.
    full_log: Vec<LogEntry>,
}

/// A `StorageBackend` that models a volatile page cache over a durable disk.
/// Cloning shares the underlying state, so the harness can keep a handle to inspect
/// the durable image after handing the backend to redb.
#[derive(Debug, Clone, Default)]
pub struct CrashBackend {
    state: Arc<Mutex<CrashState>>,
}

impl CrashBackend {
    pub fn new() -> Self {
        Self::default()
    }

    /// A fresh backend whose disk contents are exactly `bytes` — i.e. the machine after
    /// power loss: the page cache is repopulated from disk on boot.
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self {
            state: Arc::new(Mutex::new(CrashState {
                data: bytes.clone(),
                durable: bytes,
                full_log: Vec::new(),
            })),
        }
    }

    /// The byte image as of the last fsync.
    pub fn durable_image(&self) -> Vec<u8> {
        self.state.lock().unwrap().durable.clone()
    }

    /// The complete mutation stream (with `Sync` markers) since backend creation.
    pub fn full_log(&self) -> Vec<LogEntry> {
        self.state.lock().unwrap().full_log.clone()
    }

    /// Current length of the full mutation stream.
    pub fn full_log_len(&self) -> usize {
        self.state.lock().unwrap().full_log.len()
    }
}

impl StorageBackend for CrashBackend {
    fn len(&self) -> Result<u64, io::Error> {
        Ok(self.state.lock().unwrap().data.len() as u64)
    }

    fn read(&self, offset: u64, out: &mut [u8]) -> Result<(), io::Error> {
        let state = self.state.lock().unwrap();
        let offset = usize::try_from(offset)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "offset out of range"))?;
        let end = offset
            .checked_add(out.len())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "read out of range"))?;
        if end <= state.data.len() {
            out.copy_from_slice(&state.data[offset..end]);
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "read past end of storage",
            ))
        }
    }

    fn set_len(&self, len: u64) -> Result<(), io::Error> {
        let mut state = self.state.lock().unwrap();
        let ulen = usize::try_from(len)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "len out of range"))?;
        state.data.resize(ulen, 0);
        state.full_log.push(LogEntry::SetLen(len));
        Ok(())
    }

    fn sync_data(&self) -> Result<(), io::Error> {
        let mut state = self.state.lock().unwrap();
        state.durable = state.data.clone();
        state.full_log.push(LogEntry::Sync);
        Ok(())
    }

    fn write(&self, offset: u64, data: &[u8]) -> Result<(), io::Error> {
        let mut state = self.state.lock().unwrap();
        let uoffset = usize::try_from(offset)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "offset out of range"))?;
        let end = uoffset
            .checked_add(data.len())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "write out of range"))?;
        // New positions must be initialized to zero (StorageBackend contract), so grow
        // with zeros if a write lands past the current end.
        if end > state.data.len() {
            state.data.resize(end, 0);
        }
        state.data[uoffset..end].copy_from_slice(data);
        state.full_log.push(LogEntry::Write {
            offset,
            data: data.to_vec(),
        });
        Ok(())
    }
}

/// The result of driving random committed transactions: for every history index, the
/// committed snapshot, whether that commit was `Durability::Immediate`, and the length
/// of the backend's full mutation log when the commit returned (so a log cut point can
/// be mapped back to which commits were fully on "disk").
struct CommitLog {
    history: Vec<BTreeMap<u64, u64>>,
    /// `immediate[j]` — was history[j] committed with fsync?
    immediate: Vec<bool>,
    /// `positions[j]` — full-log length right after history[j]'s commit returned.
    positions: Vec<usize>,
    /// History index of the most recent Immediate commit.
    last_durable: usize,
}

/// Run a drawn number (≤ `max_txns`) of committed write transactions against `db`,
/// each under a drawn durability. History index 0 = the initial empty state created
/// by `create_db` (whose table-creating commit used default durability, i.e.
/// Immediate, so the empty table is durable from the start).
fn run_random_txns(
    tc: &hegel::TestCase,
    db: &Database,
    backend: &CrashBackend,
    max_txns: u32,
) -> CommitLog {
    let mut history: Vec<BTreeMap<u64, u64>> = vec![BTreeMap::new()];
    let mut immediate_flags: Vec<bool> = vec![true];
    let mut positions: Vec<usize> = vec![backend.full_log_len()];
    let mut last_durable: usize = 0;

    let txns = tc.draw(gs::integers::<u32>().min_value(0).max_value(max_txns));
    for _ in 0..txns {
        let mut wtxn = db.begin_write().unwrap();
        let immediate = tc.draw(gs::booleans());
        wtxn.set_durability(if immediate {
            Durability::Immediate
        } else {
            Durability::None
        })
        .unwrap();

        let mut staged = history.last().unwrap().clone();
        {
            let mut table = wtxn.open_table(TABLE).unwrap();
            let ops = tc.draw(gs::integers::<u32>().min_value(0).max_value(6));
            for _ in 0..ops {
                if tc.draw(gs::booleans()) {
                    let k = tc.draw(key());
                    let v = tc.draw(val());
                    table.insert(k, v).unwrap();
                    staged.insert(k, v);
                } else {
                    let k = tc.draw(key());
                    table.remove(k).unwrap();
                    staged.remove(&k);
                }
            }
        }
        wtxn.commit().unwrap();
        history.push(staged);
        immediate_flags.push(immediate);
        positions.push(backend.full_log_len());
        if immediate {
            last_durable = history.len() - 1;
        }
    }
    CommitLog {
        history,
        immediate: immediate_flags,
        positions,
        last_durable,
    }
}

/// Reopen a database from `disk` (the post-crash byte image) and assert the two
/// recovery guarantees: reopen succeeds, and the recovered contents are a committed
/// snapshot no older than the last durable one.
fn check_recovery(
    disk: Vec<u8>,
    history: &[BTreeMap<u64, u64>],
    last_durable: usize,
    crash_kind: &str,
) {
    let recovered_db = match Database::builder().create_with_backend(CrashBackend::from_bytes(disk))
    {
        Ok(db) => db,
        Err(e) => panic!(
            "FINDING: redb failed to reopen after simulated power loss \
             ({crash_kind}, {} commits, last durable index {last_durable}): {e:?}",
            history.len() - 1,
        ),
    };
    let recovered = read_all(&recovered_db);

    let valid = history[last_durable..].iter().any(|h| h == &recovered);
    assert!(
        valid,
        "FINDING: recovered state after simulated power loss ({crash_kind}) is not any \
         committed snapshot >= the last durable one.\n  recovered: {recovered:?}\n  \
         last_durable index: {last_durable}\n  admissible history suffix: {:?}",
        &history[last_durable..]
    );
}

/// Canonical power-loss model: everything since the last fsync is lost.
fn drive_crash(tc: &hegel::TestCase, max_txns: u32) {
    let backend = CrashBackend::new();
    let db = create_db(backend.clone());
    let log = run_random_txns(tc, &db, &backend, max_txns);

    // --- SIMULATED POWER LOSS ---
    // Take the disk image as of the last fsync; everything after it is lost.
    let disk = backend.durable_image();
    drop(db); // the crashed process is gone

    check_recovery(
        disk,
        &log.history,
        log.last_durable,
        "revert to last fsync image",
    );
}

/// STRETCH: torn/partial-write model, cutting ANYWHERE in the mutation stream —
/// including in the middle of a durable commit's writes, before its fsync.
///
/// Under in-order persistence, any prefix `F[..m]` of the full mutation stream is a
/// valid power-loss disk image (a crash at issue point `m` where every issued write
/// happened to persist); we additionally tear the next write in the middle so only
/// its first bytes land. redb's commit protocol (checksummed commit slots, god-byte
/// flip before a trailing fsync in 1-phase mode) must recover to a committed snapshot
/// no older than the newest Immediate commit that fully completed before the cut: a
/// cut mid-commit leaves an invalid slot whose checksum fails, falling back to the
/// previous valid one. (Non-durable commits never write their commit slot before the
/// next fsync, so they may legitimately be present-or-absent; hence the "some j >=
/// last durable" form.)
///
/// We cut at or after the initial table-creating commit: crash-during-database-
/// CREATION is a different scenario from crash-during-operation (there is not yet any
/// committed state to preserve), and is not modeled here.
fn drive_torn(tc: &hegel::TestCase, max_txns: u32) {
    let backend = CrashBackend::new();
    let db = create_db(backend.clone());
    let log = run_random_txns(tc, &db, &backend, max_txns);
    let full = backend.full_log();
    drop(db); // the crashed process is gone

    // --- SIMULATED POWER LOSS AT AN ARBITRARY POINT IN THE WRITE STREAM ---
    let m = tc.draw(
        gs::integers::<usize>()
            .min_value(log.positions[0])
            .max_value(full.len()),
    );
    let mut disk = Vec::new();
    for entry in &full[..m] {
        entry.apply(&mut disk);
    }
    // Optionally tear the next write: only its first `cut` bytes reach the disk.
    if m < full.len() {
        if let LogEntry::Write { offset, data } = &full[m] {
            if data.len() >= 2 && tc.draw(gs::booleans()) {
                let cut = tc.draw(
                    gs::integers::<usize>()
                        .min_value(1)
                        .max_value(data.len() - 1),
                );
                LogEntry::Write {
                    offset: *offset,
                    data: data[..cut].to_vec(),
                }
                .apply(&mut disk);
            }
        }
    }

    // The newest Immediate commit that fully completed (writes + fsync) before the
    // cut is guaranteed durable; everything at least that new and committed is an
    // admissible recovery.
    let durable_floor = (0..log.history.len())
        .filter(|&j| log.immediate[j] && log.positions[j] <= m)
        .max()
        .expect("history[0] is durable and precedes every admissible cut");

    check_recovery(
        disk,
        &log.history,
        durable_floor,
        "arbitrary in-order cut of the full write stream, next write possibly torn",
    );
}

#[hegel::test(test_cases = 300)]
fn power_loss_recovers_to_committed_snapshot(tc: hegel::TestCase) {
    drive_crash(&tc, 20);
}

#[hegel::test(test_cases = 300)]
fn torn_write_crash_recovers_to_committed_snapshot(tc: hegel::TestCase) {
    drive_torn(&tc, 20);
}
