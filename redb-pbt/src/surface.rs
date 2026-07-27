//! COVERAGE-GUIDED BREADTH: the wider `Table` query/mutate surface, against the model.
//!
//! Aimed at the cold regions of redb's `table.rs` / `btree_mutator.rs` / `btree_iters.rs`
//! (never touched by the baseline insert/remove harness): `get_mut`, `pop_first`,
//! `pop_last`, `retain_in`, `extract_from_if`, `first`, `last`, and bounded `range`
//! iteration in both directions. Every call sits inside the BTreeMap oracle — same
//! model, wider key domain (0..=200) so trees get deep enough to split and merge.

use crate::common::{create_db, TABLE};
use hegel::generators as gs;
use redb::{backends::InMemoryBackend, ReadableDatabase, ReadableTable, ReadableTableMetadata};
use std::collections::BTreeMap;
use std::ops::Bound;

fn wide_key() -> impl gs::Generator<u64> {
    gs::integers::<u64>().min_value(0).max_value(200)
}
fn val() -> impl gs::Generator<u64> {
    gs::integers::<u64>().min_value(0).max_value(1000)
}

/// A drawn predicate over (key, value): keep/select iff (k + v) % modulus == residue.
fn draw_pred(tc: &hegel::TestCase) -> (u64, u64) {
    let modulus = tc.draw(gs::integers::<u64>().min_value(1).max_value(3));
    let residue = tc.draw(gs::integers::<u64>().min_value(0).max_value(modulus - 1));
    (modulus, residue)
}

/// A drawn half-open key range lo..hi (possibly empty).
fn draw_range(tc: &hegel::TestCase) -> (u64, u64) {
    let a = tc.draw(wide_key());
    let b = tc.draw(wide_key());
    (a.min(b), a.max(b))
}

fn drive_surface(tc: &hegel::TestCase, max_txns: u32) {
    let db = create_db(InMemoryBackend::new());
    let mut model: BTreeMap<u64, u64> = BTreeMap::new();

    let txns = tc.draw(gs::integers::<u32>().min_value(0).max_value(max_txns));
    for _ in 0..txns {
        let wtxn = db.begin_write().unwrap();
        let mut staged = model.clone();
        {
            let mut table = wtxn.open_table(TABLE).unwrap();
            let ops = tc.draw(gs::integers::<u32>().min_value(0).max_value(10));
            for _ in 0..ops {
                match tc.draw(gs::integers::<u8>().min_value(0).max_value(7)) {
                    // Bulk-ish insert to grow the tree.
                    0 | 1 | 2 => {
                        let k = tc.draw(wide_key());
                        let v = tc.draw(val());
                        let prev = table.insert(k, v).unwrap().map(|g| g.value());
                        assert_eq!(prev, staged.insert(k, v), "insert prior-value for {k}");
                    }
                    3 => {
                        let k = tc.draw(wide_key());
                        let removed = table.remove(k).unwrap().map(|g| g.value());
                        assert_eq!(removed, staged.remove(&k), "remove return for {k}");
                    }
                    // get_mut: read the current value, then overwrite it in place.
                    4 => {
                        let k = tc.draw(wide_key());
                        let guard = table.get_mut(k).unwrap();
                        match guard {
                            Some(mut g) => {
                                assert_eq!(
                                    Some(g.value()),
                                    staged.get(&k).copied(),
                                    "get_mut value for {k}"
                                );
                                let v = tc.draw(val());
                                g.insert(v).unwrap();
                                staged.insert(k, v);
                            }
                            None => {
                                assert!(!staged.contains_key(&k), "get_mut missed {k}");
                            }
                        }
                    }
                    // pop_first / pop_last against the model's ends.
                    5 => {
                        let popped = table
                            .pop_first()
                            .unwrap()
                            .map(|(k, v)| (k.value(), v.value()));
                        let want = staged.first_key_value().map(|(&k, &v)| (k, v));
                        assert_eq!(popped, want, "pop_first");
                        if let Some((k, _)) = popped {
                            staged.remove(&k);
                        }
                    }
                    6 => {
                        let popped = table
                            .pop_last()
                            .unwrap()
                            .map(|(k, v)| (k.value(), v.value()));
                        let want = staged.last_key_value().map(|(&k, &v)| (k, v));
                        assert_eq!(popped, want, "pop_last");
                        if let Some((k, _)) = popped {
                            staged.remove(&k);
                        }
                    }
                    // retain_in / extract_from_if over a drawn range with a drawn
                    // predicate. extract_from_if only removes entries actually read
                    // from the iterator, so read it to exhaustion.
                    _ => {
                        let (lo, hi) = draw_range(tc);
                        let (m, r) = draw_pred(tc);
                        if tc.draw(gs::booleans()) {
                            table.retain_in(lo..hi, |k, v| (k + v) % m == r).unwrap();
                            staged.retain(|&k, &mut v| !(lo..hi).contains(&k) || (k + v) % m == r);
                        } else {
                            let extracted: Vec<(u64, u64)> = table
                                .extract_from_if(lo..hi, |k, v| (k + v) % m == r)
                                .unwrap()
                                .map(|row| {
                                    let (k, v) = row.unwrap();
                                    (k.value(), v.value())
                                })
                                .collect();
                            let want: Vec<(u64, u64)> = staged
                                .range(lo..hi)
                                .filter(|(&k, &v)| (k + v) % m == r)
                                .map(|(&k, &v)| (k, v))
                                .collect();
                            assert_eq!(extracted, want, "extract_from_if({lo}..{hi}) % {m} == {r}");
                            for (k, _) in &extracted {
                                staged.remove(k);
                            }
                        }
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

        // Read-side oracles on the committed state: len, first/last, and a drawn
        // bounded range iterated forwards and backwards.
        let rtxn = db.begin_read().unwrap();
        let table = rtxn.open_table(TABLE).unwrap();
        assert_eq!(table.len().unwrap(), model.len() as u64, "len");
        assert_eq!(
            table.first().unwrap().map(|(k, v)| (k.value(), v.value())),
            model.first_key_value().map(|(&k, &v)| (k, v)),
            "first"
        );
        assert_eq!(
            table.last().unwrap().map(|(k, v)| (k.value(), v.value())),
            model.last_key_value().map(|(&k, &v)| (k, v)),
            "last"
        );
        let (lo, hi) = draw_range(tc);
        let bounds = (Bound::Included(lo), Bound::Excluded(hi));
        let forward: Vec<(u64, u64)> = table
            .range(bounds)
            .unwrap()
            .map(|row| {
                let (k, v) = row.unwrap();
                (k.value(), v.value())
            })
            .collect();
        let want_fwd: Vec<(u64, u64)> = model.range(lo..hi).map(|(&k, &v)| (k, v)).collect();
        assert_eq!(forward, want_fwd, "range({lo}..{hi}) forward");
        let backward: Vec<(u64, u64)> = table
            .range(bounds)
            .unwrap()
            .rev()
            .map(|row| {
                let (k, v) = row.unwrap();
                (k.value(), v.value())
            })
            .collect();
        let want_bwd: Vec<(u64, u64)> = model.range(lo..hi).rev().map(|(&k, &v)| (k, v)).collect();
        assert_eq!(backward, want_bwd, "range({lo}..{hi}) backward");
    }
}

#[hegel::test(test_cases = 400)]
fn table_surface_matches_btreemap(tc: hegel::TestCase) {
    drive_surface(&tc, 30);
}
