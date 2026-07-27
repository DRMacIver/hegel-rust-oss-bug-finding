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

#[derive(Debug, Default)]
struct CrashState {
    /// The live byte image: every `write`/`set_len` mutates this ("OS page cache").
    data: Vec<u8>,
    /// The byte image as of the last `sync_data()` call ("what's really on disk").
    durable: Vec<u8>,
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
            })),
        }
    }

    /// The byte image as of the last fsync.
    pub fn durable_image(&self) -> Vec<u8> {
        self.state.lock().unwrap().durable.clone()
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
        let len = usize::try_from(len)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "len out of range"))?;
        state.data.resize(len, 0);
        Ok(())
    }

    fn sync_data(&self) -> Result<(), io::Error> {
        let mut state = self.state.lock().unwrap();
        state.durable = state.data.clone();
        Ok(())
    }

    fn write(&self, offset: u64, data: &[u8]) -> Result<(), io::Error> {
        let mut state = self.state.lock().unwrap();
        let offset = usize::try_from(offset)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "offset out of range"))?;
        let end = offset
            .checked_add(data.len())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "write out of range"))?;
        // New positions must be initialized to zero (StorageBackend contract), so grow
        // with zeros if a write lands past the current end.
        if end > state.data.len() {
            state.data.resize(end, 0);
        }
        state.data[offset..end].copy_from_slice(data);
        Ok(())
    }
}

/// Run `txns` committed write transactions against a `CrashBackend`-hosted database,
/// then simulate a power loss and check recovery. Returns nothing; panics on violation.
fn drive_crash(tc: &hegel::TestCase, max_txns: u32) {
    let backend = CrashBackend::new();
    let db = create_db(backend.clone());
    // create_db's table-creating commit used default durability (Immediate), so the
    // empty table is in the durable image: history[0] = {} and it is durable.
    let mut history: Vec<BTreeMap<u64, u64>> = vec![BTreeMap::new()];
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
        if immediate {
            last_durable = history.len() - 1;
        }
    }

    // --- SIMULATED POWER LOSS ---
    // Take the disk image as of the last fsync; everything after it is lost.
    let disk = backend.durable_image();
    drop(db); // the crashed process is gone

    let recovered_db = match Database::builder().create_with_backend(CrashBackend::from_bytes(disk))
    {
        Ok(db) => db,
        Err(e) => panic!(
            "FINDING: redb failed to reopen after simulated power loss \
             (crash at last fsync boundary, {} commits, last durable index {}): {e:?}",
            history.len() - 1,
            last_durable
        ),
    };
    let recovered = read_all(&recovered_db);

    let valid = history[last_durable..].iter().any(|h| h == &recovered);
    assert!(
        valid,
        "FINDING: recovered state after simulated power loss is not any committed \
         snapshot >= the last durable one.\n  recovered: {recovered:?}\n  \
         last_durable index: {last_durable}\n  admissible history suffix: {:?}",
        &history[last_durable..]
    );
}

#[hegel::test(test_cases = 300)]
fn power_loss_recovers_to_committed_snapshot(tc: hegel::TestCase) {
    drive_crash(&tc, 20);
}
