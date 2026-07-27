//! BREADTH: savepoints as model-restore (metamorphic: restore ≡ the model captured at
//! savepoint time).
//!
//! When an `ephemeral_savepoint` is taken we snapshot the model; `restore_savepoint`
//! (in a later write txn, then committed) must revert the live table to exactly that
//! snapshot. Interleaved with ordinary committed mutations, and full-state checked
//! after every round. A savepoint stays valid after being restored (only savepoints
//! created AFTER it are invalidated), so we also exercise restoring the same savepoint
//! repeatedly from different subsequent states.
//!
//! API notes (from redb source): `ephemeral_savepoint` must be minted on a non-dirty
//! write transaction (no tables opened yet) and captures the committed state as of
//! that transaction's start; restore takes `&mut WriteTransaction` and becomes the
//! committed state when that transaction commits.

use crate::common::{check, create_db, key, val, TABLE};
use hegel::generators as gs;
use redb::{backends::InMemoryBackend, Savepoint};
use std::collections::BTreeMap;

fn drive_savepoints(tc: &hegel::TestCase, max_rounds: u32) {
    let db = create_db(InMemoryBackend::new());
    let mut model: BTreeMap<u64, u64> = BTreeMap::new();
    // The live savepoint (at most one) and the model snapshot it must restore to.
    let mut saved: Option<(Savepoint, BTreeMap<u64, u64>)> = None;

    let rounds = tc.draw(gs::integers::<u32>().min_value(0).max_value(max_rounds));
    for _ in 0..rounds {
        match tc.draw(gs::integers::<u8>().min_value(0).max_value(3)) {
            // Mint a savepoint on a fresh (non-dirty) write txn; snapshot the model.
            0 => {
                let wtxn = db.begin_write().unwrap();
                let sp = wtxn.ephemeral_savepoint().unwrap();
                wtxn.commit().unwrap(); // empty commit; the savepoint outlives the txn
                saved = Some((sp, model.clone()));
            }
            // Ordinary committed mutation batch.
            1 | 2 => {
                let wtxn = db.begin_write().unwrap();
                let mut staged = model.clone();
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
                model = staged;
            }
            // Restore to the savepoint: live table must revert to the captured model.
            _ => {
                if let Some((sp, snapshot)) = &saved {
                    let mut wtxn = db.begin_write().unwrap();
                    wtxn.restore_savepoint(sp).unwrap();
                    wtxn.commit().unwrap();
                    model = snapshot.clone();
                    // `sp` itself remains valid (only LATER savepoints are invalidated
                    // by a restore), so it may be restored again in a later round.
                }
            }
        }
        check(&db, &model);
    }
}

#[hegel::test(test_cases = 400)]
fn restore_savepoint_reverts_to_model_snapshot(tc: hegel::TestCase) {
    drive_savepoints(&tc, 30);
}
