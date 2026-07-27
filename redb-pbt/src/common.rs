//! Shared test plumbing: the table under test, key/value generators, and whole-table
//! snapshot/check helpers used by every technique in this crate.

use hegel::generators as gs;
use redb::{Database, ReadableDatabase, ReadableTable, ReadableTableMetadata, TableDefinition};
use std::collections::BTreeMap;

pub const TABLE: TableDefinition<u64, u64> = TableDefinition::new("t");

pub fn key() -> impl gs::Generator<u64> {
    gs::integers::<u64>().min_value(0).max_value(15)
}

pub fn val() -> impl gs::Generator<u64> {
    gs::integers::<u64>().min_value(0).max_value(1000)
}

/// Snapshot of the whole table as seen by a fresh read transaction.
pub fn read_all(db: &Database) -> BTreeMap<u64, u64> {
    let rtxn = db.begin_read().unwrap();
    let table = rtxn.open_table(TABLE).unwrap();
    let mut out = BTreeMap::new();
    for row in table.iter().unwrap() {
        let (k, v) = row.unwrap();
        out.insert(k.value(), v.value());
    }
    out
}

/// Full oracle against the committed state: contents, len, and point gets.
pub fn check(db: &Database, model: &BTreeMap<u64, u64>) {
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

/// Create a database on the given backend and commit an initial (default-durability,
/// i.e. Immediate) transaction that creates `TABLE`, so read transactions never hit
/// `TableDoesNotExist` and the empty table is always part of the durable image.
pub fn create_db(backend: impl redb::StorageBackend) -> Database {
    let db = Database::builder().create_with_backend(backend).unwrap();
    let wtxn = db.begin_write().unwrap();
    {
        let _ = wtxn.open_table(TABLE).unwrap();
    }
    wtxn.commit().unwrap();
    db
}
