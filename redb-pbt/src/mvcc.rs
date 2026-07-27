//! TECHNIQUE: MVCC snapshot consistency over interleaved transactions.
//!
//! Generalizes the baseline's single snapshot-isolation check: keep up to K OPEN read
//! transactions simultaneously, each tagged with the index into `history` (the linear
//! sequence of committed states) at which it was opened. Interleave: opening readers,
//! committing/aborting write transactions, and closing readers — and after EVERY step
//! assert that every open reader still observes exactly `history[its tag]`, regardless
//! of how many commits have happened since it was opened.
//!
//! This is deterministic (no threads): the "concurrency" is transaction lifetime
//! overlap, which is precisely what MVCC must get right, and it shrinks well.

use crate::common::{create_db, key, val, TABLE};
use hegel::generators as gs;
use redb::{backends::InMemoryBackend, ReadTransaction, ReadableDatabase, ReadableTable};
use std::collections::BTreeMap;

const MAX_OPEN_READERS: usize = 4;

fn reader_contents(rtxn: &ReadTransaction) -> BTreeMap<u64, u64> {
    let table = rtxn.open_table(TABLE).unwrap();
    let mut out = BTreeMap::new();
    for row in table.iter().unwrap() {
        let (k, v) = row.unwrap();
        out.insert(k.value(), v.value());
    }
    out
}

fn drive_mvcc(tc: &hegel::TestCase, max_steps: u32) {
    let db = create_db(InMemoryBackend::new());
    let mut history: Vec<BTreeMap<u64, u64>> = vec![BTreeMap::new()];
    // Open read transactions, each tagged with the history index it must observe.
    let mut readers: Vec<(ReadTransaction, usize)> = Vec::new();

    let steps = tc.draw(gs::integers::<u32>().min_value(0).max_value(max_steps));
    for _ in 0..steps {
        match tc.draw(gs::integers::<u8>().min_value(0).max_value(3)) {
            // Open a new read transaction, snapshotting the current committed state.
            0 => {
                if readers.len() < MAX_OPEN_READERS {
                    readers.push((db.begin_read().unwrap(), history.len() - 1));
                }
            }
            // Run a write transaction: a batch of inserts/removes, then commit or abort.
            1 | 2 => {
                let wtxn = db.begin_write().unwrap();
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
                if tc.draw(gs::booleans()) {
                    wtxn.commit().unwrap();
                    history.push(staged);
                } else {
                    wtxn.abort().unwrap();
                }
            }
            // Close one open reader (explicitly, exercising ReadTransaction::close).
            _ => {
                if !readers.is_empty() {
                    let i = tc.draw(
                        gs::integers::<usize>()
                            .min_value(0)
                            .max_value(readers.len() - 1),
                    );
                    let (rtxn, _) = readers.swap_remove(i);
                    rtxn.close().unwrap();
                }
            }
        }

        // THE INVARIANT: every open reader observes exactly the committed state at the
        // moment it was opened — no lost updates, no phantom writes, no snapshot drift.
        for (rtxn, tag) in &readers {
            assert_eq!(
                &reader_contents(rtxn),
                &history[*tag],
                "open read txn (opened at history index {tag}, now at {}) \
                 no longer observes its snapshot",
                history.len() - 1
            );
        }
    }
}

#[hegel::test(test_cases = 500)]
fn open_readers_observe_their_snapshot(tc: hegel::TestCase) {
    drive_mvcc(&tc, 40);
}
