//! COVERAGE-GUIDED BREADTH: the error surface (`error.rs` was 0% covered).
//!
//! Each case provokes a documented error through the public API and asserts the exact
//! variant, that Display carries the identifying detail (table name etc.), and that
//! Debug formatting works. Also exercises `Error`'s From-conversions via `.into()`.
//! Deterministic (no drawn data): these are contract checks on specific error paths.

use crate::common::TABLE;
use crate::crash::CrashBackend;
use redb::{
    backends::InMemoryBackend, Database, Durability, MultimapTableDefinition, ReadableDatabase,
    SavepointError, SetDurabilityError, TableDefinition, TableError,
};

fn fresh_db_with_table() -> Database {
    let db = Database::builder()
        .create_with_backend(InMemoryBackend::new())
        .unwrap();
    let wtxn = db.begin_write().unwrap();
    {
        let _ = wtxn.open_table(TABLE).unwrap();
    }
    wtxn.commit().unwrap();
    db
}

#[test]
fn table_does_not_exist_error() {
    let db = fresh_db_with_table();
    let rtxn = db.begin_read().unwrap();
    let missing: TableDefinition<u64, u64> = TableDefinition::new("nope");
    let err = rtxn.open_table(missing).unwrap_err();
    assert!(matches!(err, TableError::TableDoesNotExist(ref n) if n == "nope"));
    let shown = err.to_string();
    assert!(shown.contains("nope"), "display lacks table name: {shown}");
    let as_general: redb::Error = err.into();
    assert!(as_general.to_string().contains("nope"));
    let _ = format!("{as_general:?}");
}

#[test]
fn table_type_mismatch_error() {
    let db = fresh_db_with_table();
    let rtxn = db.begin_read().unwrap();
    let wrong: TableDefinition<&str, u64> = TableDefinition::new("t");
    let err = rtxn.open_table(wrong).unwrap_err();
    assert!(matches!(err, TableError::TableTypeMismatch { .. }), "{err:?}");
    let shown = err.to_string();
    assert!(!shown.is_empty());
    let _ = format!("{err:?}");
}

#[test]
fn table_is_not_multimap_error() {
    let db = fresh_db_with_table();
    let wtxn = db.begin_write().unwrap();
    let as_multimap: MultimapTableDefinition<u64, u64> = MultimapTableDefinition::new("t");
    let err = match wtxn.open_multimap_table(as_multimap) {
        Ok(_) => panic!("opening a normal table as multimap must fail"),
        Err(e) => e,
    };
    assert!(
        matches!(err, TableError::TableIsNotMultimap(ref n) if n == "t"),
        "{err:?}"
    );
    assert!(err.to_string().contains('t'));
    wtxn.abort().unwrap();
}

#[test]
fn table_already_open_error() {
    let db = fresh_db_with_table();
    let wtxn = db.begin_write().unwrap();
    let _first = wtxn.open_table(TABLE).unwrap();
    let err = wtxn.open_table(TABLE).unwrap_err();
    assert!(
        matches!(err, TableError::TableAlreadyOpen(ref n, _) if n == "t"),
        "{err:?}"
    );
    assert!(err.to_string().contains('t'));
}

#[test]
fn savepoint_on_dirty_transaction_is_invalid() {
    let db = fresh_db_with_table();
    let wtxn = db.begin_write().unwrap();
    {
        let mut table = wtxn.open_table(TABLE).unwrap();
        table.insert(1, 1).unwrap();
    }
    // Tables have been opened => the transaction is dirty => no savepoint allowed.
    let err = match wtxn.ephemeral_savepoint() {
        Ok(_) => panic!("savepoint on a dirty transaction must fail"),
        Err(e) => e,
    };
    assert!(matches!(err, SavepointError::InvalidSavepoint), "{err:?}");
    assert!(!err.to_string().is_empty());
    wtxn.abort().unwrap();
}

#[test]
fn durability_cannot_drop_after_persistent_savepoint() {
    let db = fresh_db_with_table();
    let mut wtxn = db.begin_write().unwrap();
    let _id = wtxn.persistent_savepoint().unwrap();
    let err = wtxn.set_durability(Durability::None).unwrap_err();
    assert!(
        matches!(err, SetDurabilityError::PersistentSavepointModified),
        "{err:?}"
    );
    assert!(!err.to_string().is_empty());
    wtxn.abort().unwrap();
}

#[test]
fn savepoint_from_another_database_is_invalid() {
    let db_a = fresh_db_with_table();
    let db_b = fresh_db_with_table();
    let wtxn_a = db_a.begin_write().unwrap();
    let sp_a = wtxn_a.ephemeral_savepoint().unwrap();
    wtxn_a.commit().unwrap();

    let mut wtxn_b = db_b.begin_write().unwrap();
    let err = wtxn_b.restore_savepoint(&sp_a).unwrap_err();
    assert!(matches!(err, SavepointError::InvalidSavepoint), "{err:?}");
    wtxn_b.abort().unwrap();
}

#[test]
fn opening_garbage_bytes_is_a_clean_error() {
    // A backend full of garbage that is long enough to look like a file but cannot
    // contain redb's magic number: opening must fail with a DatabaseError (never a
    // panic), and the error must format.
    let garbage = CrashBackend::from_bytes(vec![0xAB; 4096]);
    let err = Database::builder().create_with_backend(garbage).unwrap_err();
    let shown = err.to_string();
    assert!(!shown.is_empty());
    let _ = format!("{err:?}");
}
