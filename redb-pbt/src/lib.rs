//! Property-based-testing testbed for redb (embedded KV store) — stateful model-based
//! testing plus MVCC snapshot-isolation checks, as a substrate for developing new PBT
//! techniques (the frontier here is concurrency/linearizability and crash/power-loss
//! injection, since redb's own libFuzzer harness already covers single-threaded
//! model-vs-BTreeMap with savepoints/durability/repair).
//!
//! Baseline oracle: a `BTreeMap<u64, u64>` mirrors the committed contents. We drive a
//! sequence of write transactions (each a drawn batch of insert/remove ops, then commit
//! OR abort, under a drawn durability), and after every transaction assert:
//!   * full contents + point gets + len match the model,
//!   * per-op `remove` return value matches the model,
//!   * abort leaves the committed state unchanged,
//!   * SNAPSHOT ISOLATION: a read transaction opened before a commit keeps observing the
//!     pre-commit snapshot even after the writer commits.

#[cfg(test)]
mod model {
    use hegel::generators as gs;
    use redb::{
        backends::InMemoryBackend, Database, Durability, ReadableDatabase, ReadableTable,
        ReadableTableMetadata, TableDefinition,
    };
    use std::collections::BTreeMap;

    const TABLE: TableDefinition<u64, u64> = TableDefinition::new("t");

    fn key() -> impl gs::Generator<u64> {
        gs::integers::<u64>().min_value(0).max_value(15)
    }
    fn val() -> impl gs::Generator<u64> {
        gs::integers::<u64>().min_value(0).max_value(1000)
    }

    /// Snapshot of the whole table as seen by a fresh read transaction.
    fn read_all(db: &Database) -> BTreeMap<u64, u64> {
        let rtxn = db.begin_read().unwrap();
        let table = rtxn.open_table(TABLE).unwrap();
        let mut out = BTreeMap::new();
        for row in table.iter().unwrap() {
            let (k, v) = row.unwrap();
            out.insert(k.value(), v.value());
        }
        out
    }

    fn check(db: &Database, model: &BTreeMap<u64, u64>) {
        assert_eq!(&read_all(db), model, "committed contents != model");
        let rtxn = db.begin_read().unwrap();
        let table = rtxn.open_table(TABLE).unwrap();
        assert_eq!(table.len().unwrap(), model.len() as u64, "len mismatch");
        for (&k, &v) in model {
            assert_eq!(
                table.get(k).unwrap().map(|g| g.value()),
                Some(v),
                "point get for {k}"
            );
        }
    }

    fn drive(tc: &hegel::TestCase, max_txns: u32) {
        let db = Database::builder()
            .create_with_backend(InMemoryBackend::new())
            .unwrap();
        // Ensure the table exists so read transactions never hit TableDoesNotExist.
        {
            let wtxn = db.begin_write().unwrap();
            {
                let _ = wtxn.open_table(TABLE).unwrap();
            }
            wtxn.commit().unwrap();
        }
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
}
