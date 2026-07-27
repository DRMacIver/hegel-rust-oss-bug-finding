//! COVERAGE-GUIDED BREADTH: the type-encoding surface (`types.rs`, `tuple_types.rs`,
//! `complex_types.rs` — all previously 0% or near-0% covered).
//!
//! Four table instantiations exercise tuple keys, `Vec<T>` values, signed-integer keys,
//! `Option<T>` values, `&[u8]` keys, `bool`/`char` tuple values, `String` keys and
//! fixed-array values. The oracle in every case: after a committed batch of
//! inserts/removes, iteration must yield exactly the model's entries IN THE MODEL'S
//! (Rust `Ord`) ORDER — i.e. redb's byte-encoded key ordering must agree with the
//! natural ordering of the decoded type (this is where sign-handling, varint-length and
//! lexicographic-encoding bugs would show) — and point gets must round-trip every value.

use hegel::generators as gs;
use redb::{backends::InMemoryBackend, Database, ReadableDatabase, ReadableTable, TableDefinition};
use std::collections::BTreeMap;

const TUPLE_TABLE: TableDefinition<(u64, &str), Vec<u64>> = TableDefinition::new("tuple");
const SIGNED_TABLE: TableDefinition<i64, Option<u64>> = TableDefinition::new("signed");
const BYTES_TABLE: TableDefinition<&[u8], (bool, char)> = TableDefinition::new("bytes");
const STRING_TABLE: TableDefinition<String, [u64; 3]> = TableDefinition::new("string");

fn fresh_db() -> Database {
    Database::builder()
        .create_with_backend(InMemoryBackend::new())
        .unwrap()
}

/// Short strings over an alphabet that includes multi-byte UTF-8 (é is 2 bytes,
/// € is 3), so length-prefix and lexicographic-byte-order handling get stressed.
fn small_string(tc: &hegel::TestCase) -> String {
    const ALPHABET: [char; 4] = ['a', 'b', 'é', '€'];
    let len = tc.draw(gs::integers::<usize>().min_value(0).max_value(3));
    (0..len)
        .map(|_| ALPHABET[tc.draw(gs::integers::<usize>().min_value(0).max_value(3))])
        .collect()
}

fn small_bytes(tc: &hegel::TestCase) -> Vec<u8> {
    let len = tc.draw(gs::integers::<usize>().min_value(0).max_value(3));
    (0..len)
        .map(|_| tc.draw(gs::integers::<u8>().min_value(0).max_value(255)))
        .collect()
}

fn small_vec(tc: &hegel::TestCase) -> Vec<u64> {
    let len = tc.draw(gs::integers::<usize>().min_value(0).max_value(3));
    (0..len)
        .map(|_| tc.draw(gs::integers::<u64>().min_value(0).max_value(1000)))
        .collect()
}

fn tuple_keys(tc: &hegel::TestCase) {
    let db = fresh_db();
    let mut model: BTreeMap<(u64, String), Vec<u64>> = BTreeMap::new();
    let wtxn = db.begin_write().unwrap();
    {
        let mut table = wtxn.open_table(TUPLE_TABLE).unwrap();
        let n = tc.draw(gs::integers::<u32>().min_value(0).max_value(12));
        for _ in 0..n {
            let k = (
                tc.draw(gs::integers::<u64>().min_value(0).max_value(3)),
                small_string(tc),
            );
            if tc.draw(gs::booleans()) {
                let v = small_vec(tc);
                let prev = table.insert((k.0, k.1.as_str()), v.clone()).unwrap();
                assert_eq!(
                    prev.map(|g| g.value()),
                    model.insert(k, v),
                    "tuple insert prior value"
                );
            } else {
                let removed = table.remove((k.0, k.1.as_str())).unwrap();
                assert_eq!(
                    removed.map(|g| g.value()),
                    model.remove(&k),
                    "tuple remove return"
                );
            }
        }
    }
    wtxn.commit().unwrap();

    let rtxn = db.begin_read().unwrap();
    let table = rtxn.open_table(TUPLE_TABLE).unwrap();
    let got: Vec<((u64, String), Vec<u64>)> = table
        .iter()
        .unwrap()
        .map(|row| {
            let (k, v) = row.unwrap();
            let (a, b) = k.value();
            ((a, b.to_string()), v.value())
        })
        .collect();
    let want: Vec<((u64, String), Vec<u64>)> = model
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    assert_eq!(got, want, "tuple-key iteration order/contents");
    for (k, v) in &model {
        let g = table.get((k.0, k.1.as_str())).unwrap().unwrap();
        assert_eq!(&g.value(), v, "tuple point get");
    }
}

fn signed_keys(tc: &hegel::TestCase) {
    let db = fresh_db();
    let mut model: BTreeMap<i64, Option<u64>> = BTreeMap::new();
    let wtxn = db.begin_write().unwrap();
    {
        let mut table = wtxn.open_table(SIGNED_TABLE).unwrap();
        let n = tc.draw(gs::integers::<u32>().min_value(0).max_value(12));
        for _ in 0..n {
            let k = tc.draw(gs::integers::<i64>().min_value(-50).max_value(50));
            if tc.draw(gs::booleans()) {
                let v = if tc.draw(gs::booleans()) {
                    Some(tc.draw(gs::integers::<u64>().min_value(0).max_value(1000)))
                } else {
                    None
                };
                let prev = table.insert(k, v).unwrap();
                assert_eq!(
                    prev.map(|g| g.value()),
                    model.insert(k, v),
                    "signed insert prior value"
                );
            } else {
                let removed = table.remove(k).unwrap();
                assert_eq!(
                    removed.map(|g| g.value()),
                    model.remove(&k),
                    "signed remove return"
                );
            }
        }
    }
    wtxn.commit().unwrap();

    let rtxn = db.begin_read().unwrap();
    let table = rtxn.open_table(SIGNED_TABLE).unwrap();
    let got: Vec<(i64, Option<u64>)> = table
        .iter()
        .unwrap()
        .map(|row| {
            let (k, v) = row.unwrap();
            (k.value(), v.value())
        })
        .collect();
    let want: Vec<(i64, Option<u64>)> = model.iter().map(|(&k, &v)| (k, v)).collect();
    assert_eq!(
        got, want,
        "signed-key iteration must follow numeric (Rust Ord) order incl. negatives"
    );
}

fn byte_keys(tc: &hegel::TestCase) {
    const CHARS: [char; 4] = ['x', 'ß', '中', '🦀'];
    let db = fresh_db();
    let mut model: BTreeMap<Vec<u8>, (bool, char)> = BTreeMap::new();
    let wtxn = db.begin_write().unwrap();
    {
        let mut table = wtxn.open_table(BYTES_TABLE).unwrap();
        let n = tc.draw(gs::integers::<u32>().min_value(0).max_value(12));
        for _ in 0..n {
            let k = small_bytes(tc);
            if tc.draw(gs::booleans()) {
                let v = (
                    tc.draw(gs::booleans()),
                    CHARS[tc.draw(gs::integers::<usize>().min_value(0).max_value(3))],
                );
                let prev = table.insert(k.as_slice(), v).unwrap();
                assert_eq!(
                    prev.map(|g| g.value()),
                    model.insert(k, v),
                    "bytes insert prior value"
                );
            } else {
                let removed = table.remove(k.as_slice()).unwrap();
                assert_eq!(
                    removed.map(|g| g.value()),
                    model.remove(&k),
                    "bytes remove return"
                );
            }
        }
    }
    wtxn.commit().unwrap();

    let rtxn = db.begin_read().unwrap();
    let table = rtxn.open_table(BYTES_TABLE).unwrap();
    let got: Vec<(Vec<u8>, (bool, char))> = table
        .iter()
        .unwrap()
        .map(|row| {
            let (k, v) = row.unwrap();
            (k.value().to_vec(), v.value())
        })
        .collect();
    let want: Vec<(Vec<u8>, (bool, char))> = model.iter().map(|(k, &v)| (k.clone(), v)).collect();
    assert_eq!(got, want, "byte-key iteration order/contents");
}

fn string_keys(tc: &hegel::TestCase) {
    let db = fresh_db();
    let mut model: BTreeMap<String, [u64; 3]> = BTreeMap::new();
    let wtxn = db.begin_write().unwrap();
    {
        let mut table = wtxn.open_table(STRING_TABLE).unwrap();
        let n = tc.draw(gs::integers::<u32>().min_value(0).max_value(12));
        for _ in 0..n {
            let k = small_string(tc);
            if tc.draw(gs::booleans()) {
                let v = [
                    tc.draw(gs::integers::<u64>().min_value(0).max_value(9)),
                    tc.draw(gs::integers::<u64>().min_value(0).max_value(9)),
                    tc.draw(gs::integers::<u64>().min_value(0).max_value(9)),
                ];
                let prev = table.insert(k.clone(), v).unwrap();
                assert_eq!(
                    prev.map(|g| g.value()),
                    model.insert(k, v),
                    "string insert prior value"
                );
            } else {
                let removed = table.remove(k.clone()).unwrap();
                assert_eq!(
                    removed.map(|g| g.value()),
                    model.remove(&k),
                    "string remove return"
                );
            }
        }
    }
    wtxn.commit().unwrap();

    let rtxn = db.begin_read().unwrap();
    let table = rtxn.open_table(STRING_TABLE).unwrap();
    let got: Vec<(String, [u64; 3])> = table
        .iter()
        .unwrap()
        .map(|row| {
            let (k, v) = row.unwrap();
            (k.value().to_string(), v.value())
        })
        .collect();
    let want: Vec<(String, [u64; 3])> = model.iter().map(|(k, &v)| (k.clone(), v)).collect();
    assert_eq!(got, want, "string-key iteration order/contents");
}

#[hegel::test(test_cases = 300)]
fn typed_tables_round_trip_in_order(tc: hegel::TestCase) {
    tuple_keys(&tc);
    signed_keys(&tc);
    byte_keys(&tc);
    string_keys(&tc);
}
