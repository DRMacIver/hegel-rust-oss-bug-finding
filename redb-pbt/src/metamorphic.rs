//! BREADTH: metamorphic transaction-grouping invariance.
//!
//! The committed contents of the database must be a function of the SEQUENCE of applied
//! operations, not of how that sequence is partitioned into transactions: N ops in one
//! committed txn ≡ the same N ops split (at arbitrary drawn boundaries) across several
//! committed txns. We build two databases from the same drawn op list — route A applies
//! them in a single committed transaction, route B in drawn chunks (including empty
//! chunks) — and demand identical committed contents, which must also equal the model
//! fold. No model of transaction machinery is needed: the relation itself is the oracle.

use crate::common::{create_db, key, read_all, val, TABLE};
use hegel::generators as gs;
use redb::{backends::InMemoryBackend, Database};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy)]
enum Op {
    Insert(u64, u64),
    Remove(u64),
}

fn apply(table: &mut redb::Table<'_, u64, u64>, op: Op) {
    match op {
        Op::Insert(k, v) => {
            table.insert(k, v).unwrap();
        }
        Op::Remove(k) => {
            table.remove(k).unwrap();
        }
    }
}

/// Apply `ops` to `db`, committing at each boundary in `chunk_sizes` (which partitions
/// the op list; sizes may be zero, producing empty committed transactions).
fn apply_chunked(db: &Database, ops: &[Op], chunk_sizes: &[usize]) {
    let mut rest = ops;
    for &n in chunk_sizes {
        let (chunk, tail) = rest.split_at(n);
        rest = tail;
        let wtxn = db.begin_write().unwrap();
        {
            let mut table = wtxn.open_table(TABLE).unwrap();
            for &op in chunk {
                apply(&mut table, op);
            }
        }
        wtxn.commit().unwrap();
    }
    assert!(rest.is_empty(), "chunk sizes must partition the op list");
}

fn drive_grouping(tc: &hegel::TestCase, max_ops: u32) {
    // Draw the op list once, as data, so both routes replay the identical sequence.
    let n = tc.draw(gs::integers::<u32>().min_value(0).max_value(max_ops)) as usize;
    let mut ops = Vec::with_capacity(n);
    for _ in 0..n {
        ops.push(if tc.draw(gs::booleans()) {
            Op::Insert(tc.draw(key()), tc.draw(val()))
        } else {
            Op::Remove(tc.draw(key()))
        });
    }

    // Draw a partition of the op list into chunk sizes (zeros allowed).
    let mut chunk_sizes = Vec::new();
    let mut remaining = n;
    while remaining > 0 {
        let take = tc.draw(gs::integers::<usize>().min_value(0).max_value(remaining));
        chunk_sizes.push(take);
        remaining -= take;
    }

    // Route A: everything in ONE committed transaction.
    let db_a = create_db(InMemoryBackend::new());
    apply_chunked(&db_a, &ops, &[n]);

    // Route B: the same ops split across several committed transactions.
    let db_b = create_db(InMemoryBackend::new());
    apply_chunked(&db_b, &ops, &chunk_sizes);

    // Model fold, as an independent third opinion.
    let mut model: BTreeMap<u64, u64> = BTreeMap::new();
    for &op in &ops {
        match op {
            Op::Insert(k, v) => {
                model.insert(k, v);
            }
            Op::Remove(k) => {
                model.remove(&k);
            }
        }
    }

    let a = read_all(&db_a);
    let b = read_all(&db_b);
    assert_eq!(
        a, b,
        "txn grouping changed the committed contents (chunks: {chunk_sizes:?}, ops: {ops:?})"
    );
    assert_eq!(a, model, "committed contents != model fold (ops: {ops:?})");
}

#[hegel::test(test_cases = 400)]
fn txn_grouping_is_invariant(tc: hegel::TestCase) {
    drive_grouping(&tc, 30);
}
