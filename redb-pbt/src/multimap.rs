//! BREADTH: multimap tables against a `BTreeMap<u64, BTreeSet<u64>>` model.
//!
//! Same shape as the baseline harness, but for `MultimapTableDefinition`: each key maps
//! to a SET of values. Oracles: per-op return values (`insert`/`remove` report whether
//! the exact pair was present; `remove_all` yields the removed values in ascending
//! order), full contents via `iter()` (keys and per-key value sets, in order), `len()`
//! (total number of key-value PAIRS), per-key `get` (ascending values; empty iterator
//! for absent keys), and commit/abort semantics.

use hegel::generators as gs;
use redb::{
    backends::InMemoryBackend, Database, MultimapTableDefinition, ReadableDatabase,
    ReadableMultimapTable, ReadableTableMetadata,
};
use std::collections::{BTreeMap, BTreeSet};

const MM: MultimapTableDefinition<u64, u64> = MultimapTableDefinition::new("mm");

type Model = BTreeMap<u64, BTreeSet<u64>>;

fn key() -> impl gs::Generator<u64> {
    gs::integers::<u64>().min_value(0).max_value(7)
}
fn val() -> impl gs::Generator<u64> {
    gs::integers::<u64>().min_value(0).max_value(7)
}

fn read_all_mm(db: &Database) -> Model {
    let rtxn = db.begin_read().unwrap();
    let table = rtxn.open_multimap_table(MM).unwrap();
    let mut out = Model::new();
    for row in table.iter().unwrap() {
        let (k, values) = row.unwrap();
        let mut set = BTreeSet::new();
        for v in values {
            set.insert(v.unwrap().value());
        }
        assert!(
            out.insert(k.value(), set).is_none(),
            "iter() yielded duplicate key {}",
            k.value()
        );
    }
    out
}

fn check_mm(db: &Database, model: &Model) {
    assert_eq!(&read_all_mm(db), model, "committed contents != model");
    let rtxn = db.begin_read().unwrap();
    let table = rtxn.open_multimap_table(MM).unwrap();
    let pairs: u64 = model.values().map(|s| s.len() as u64).sum();
    assert_eq!(table.len().unwrap(), pairs, "len() != total pair count");
    // Per-key point lookups, including a key guaranteed absent (empty iterator).
    for (&k, set) in model {
        let got: Vec<u64> = table
            .get(k)
            .unwrap()
            .map(|g| g.unwrap().value())
            .collect();
        let want: Vec<u64> = set.iter().copied().collect();
        assert_eq!(got, want, "get({k}) values (must be ascending)");
    }
    let absent = table.get(u64::MAX).unwrap();
    assert_eq!(absent.count(), 0, "get(absent key) must be empty");
}

fn drive_mm(tc: &hegel::TestCase, max_txns: u32) {
    let db = Database::builder()
        .create_with_backend(InMemoryBackend::new())
        .unwrap();
    {
        let wtxn = db.begin_write().unwrap();
        {
            let _ = wtxn.open_multimap_table(MM).unwrap();
        }
        wtxn.commit().unwrap();
    }
    let mut model = Model::new();

    let txns = tc.draw(gs::integers::<u32>().min_value(0).max_value(max_txns));
    for _ in 0..txns {
        let wtxn = db.begin_write().unwrap();
        let mut staged = model.clone();
        {
            let mut table = wtxn.open_multimap_table(MM).unwrap();
            let ops = tc.draw(gs::integers::<u32>().min_value(0).max_value(8));
            for _ in 0..ops {
                match tc.draw(gs::integers::<u8>().min_value(0).max_value(5)) {
                    // insert(k, v): returns true iff the pair was ALREADY present.
                    0 | 1 => {
                        let k = tc.draw(key());
                        let v = tc.draw(val());
                        let was_present = table.insert(k, v).unwrap();
                        let model_present = !staged.entry(k).or_default().insert(v);
                        assert_eq!(was_present, model_present, "insert({k},{v}) return");
                    }
                    // remove(k, v): returns true iff the pair was present.
                    2 | 3 => {
                        let k = tc.draw(key());
                        let v = tc.draw(val());
                        let removed = table.remove(k, v).unwrap();
                        let model_removed = match staged.get_mut(&k) {
                            Some(set) => {
                                let r = set.remove(&v);
                                if set.is_empty() {
                                    staged.remove(&k);
                                }
                                r
                            }
                            None => false,
                        };
                        assert_eq!(removed, model_removed, "remove({k},{v}) return");
                    }
                    // Bulk insert: many distinct values under ONE key, to push the
                    // per-key collection out of its inline representation into a
                    // subtree (redb stores small value sets inline in the leaf and
                    // spills to a full B-tree beyond that).
                    4 => {
                        let k = tc.draw(key());
                        let n = tc.draw(gs::integers::<u64>().min_value(1).max_value(150));
                        let base = tc.draw(gs::integers::<u64>().min_value(0).max_value(10_000));
                        let set = staged.entry(k).or_default();
                        for v in base..base + n {
                            let was_present = table.insert(k, v).unwrap();
                            assert_eq!(was_present, !set.insert(v), "bulk insert({k},{v})");
                        }
                    }
                    // remove_all(k): yields the removed values in ascending order.
                    _ => {
                        let k = tc.draw(key());
                        let removed: Vec<u64> = table
                            .remove_all(k)
                            .unwrap()
                            .map(|g| g.unwrap().value())
                            .collect();
                        let want: Vec<u64> = staged
                            .remove(&k)
                            .unwrap_or_default()
                            .into_iter()
                            .collect();
                        assert_eq!(removed, want, "remove_all({k}) values");
                    }
                }
            }
        }
        if tc.draw(gs::booleans()) {
            wtxn.commit().unwrap();
            model = staged;
        } else {
            wtxn.abort().unwrap();
        }
        check_mm(&db, &model);
    }
}

#[hegel::test(test_cases = 400)]
fn multimap_matches_btreemap_of_sets(tc: hegel::TestCase) {
    drive_mm(&tc, 30);
}
