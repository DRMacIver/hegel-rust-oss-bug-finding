//! Baseline stateful model-based harness (see crate docs). Model = `BTreeMap<u64,u64>`;
//! write-txn batches of insert/remove with commit/abort under drawn durability; oracles
//! for contents/point-gets/len/return-values; plus a single snapshot-isolation check.

use crate::common::{check, create_db, key, val, TABLE};
use hegel::generators as gs;
use redb::{backends::InMemoryBackend, Durability, ReadableDatabase, ReadableTable};
use std::collections::BTreeMap;

fn drive(tc: &hegel::TestCase, max_txns: u32) {
    let db = create_db(InMemoryBackend::new());
    let mut model: BTreeMap<u64, u64> = BTreeMap::new();

    let txns = tc.draw(gs::integers::<u32>().min_value(0).max_value(max_txns));
    for _ in 0..txns {
        // Open a read transaction BEFORE the write, capturing the current snapshot.
        let snap_rtxn = db.begin_read().unwrap();
        let snap_before = model.clone();

        let mut wtxn = db.begin_write().unwrap();
        match tc.draw(gs::integers::<u8>().min_value(0).max_value(1)) {
            0 => wtxn.set_durability(Durability::None).unwrap(),
            _ => wtxn.set_durability(Durability::Immediate).unwrap(),
        }

        let mut staged = model.clone();
        {
            let mut table = wtxn.open_table(TABLE).unwrap();
            let ops = tc.draw(gs::integers::<u32>().min_value(0).max_value(8));
            for _ in 0..ops {
                if tc.draw(gs::booleans()) {
                    let k = tc.draw(key());
                    let v = tc.draw(val());
                    let prev = table.insert(k, v).unwrap().map(|g| g.value());
                    assert_eq!(prev, staged.insert(k, v), "insert prior-value for {k}");
                } else {
                    let k = tc.draw(key());
                    let removed = table.remove(k).unwrap().map(|g| g.value());
                    assert_eq!(removed, staged.remove(&k), "remove return for {k}");
                }
            }
        }

        if tc.draw(gs::booleans()) {
            wtxn.commit().unwrap();
            model = staged;
        } else {
            wtxn.abort().unwrap();
            // committed state unchanged
        }
        check(&db, &model);

        // Snapshot isolation: the read txn opened before the write must still observe
        // the pre-write committed state, regardless of the just-committed changes.
        {
            let table = snap_rtxn.open_table(TABLE).unwrap();
            let mut snap_now = BTreeMap::new();
            for row in table.iter().unwrap() {
                let (k, v) = row.unwrap();
                snap_now.insert(k.value(), v.value());
            }
            assert_eq!(
                snap_now, snap_before,
                "read-transaction snapshot changed under a concurrent commit"
            );
        }
    }
}

#[hegel::test(test_cases = 500)]
fn redb_matches_btreemap(tc: hegel::TestCase) {
    drive(&tc, 40);
}
