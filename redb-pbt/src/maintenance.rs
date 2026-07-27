//! COVERAGE-GUIDED BREADTH: database maintenance — `check_integrity` and `compact` —
//! as metamorphic identities: neither may change the committed contents, integrity
//! must hold before and after, and a compacted database must still pass the full
//! model check. Also a real-file round-trip: create a database on an actual temp
//! file (FileBackend, previously 0% covered), run committed txns, drop, `open`, and
//! demand identical contents.

use crate::common::{check, create_db, key, read_all, val, TABLE};
use hegel::generators as gs;
use redb::{backends::InMemoryBackend, Database};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

fn run_committed_txns(tc: &hegel::TestCase, db: &Database, max_txns: u32) -> BTreeMap<u64, u64> {
    let mut model: BTreeMap<u64, u64> = BTreeMap::new();
    let txns = tc.draw(gs::integers::<u32>().min_value(0).max_value(max_txns));
    for _ in 0..txns {
        let wtxn = db.begin_write().unwrap();
        {
            let mut table = wtxn.open_table(TABLE).unwrap();
            let ops = tc.draw(gs::integers::<u32>().min_value(0).max_value(8));
            for _ in 0..ops {
                if tc.draw(gs::booleans()) {
                    let k = tc.draw(key());
                    let v = tc.draw(val());
                    table.insert(k, v).unwrap();
                    model.insert(k, v);
                } else {
                    let k = tc.draw(key());
                    table.remove(k).unwrap();
                    model.remove(&k);
                }
            }
        }
        wtxn.commit().unwrap();
    }
    model
}

fn drive_maintenance(tc: &hegel::TestCase, max_txns: u32) {
    let mut db = create_db(InMemoryBackend::new());
    let model = run_committed_txns(tc, &db, max_txns);

    assert!(
        db.check_integrity().unwrap(),
        "cleanly-committed database reported not clean"
    );
    check(&db, &model);

    // compact() must not change the observable contents, and the result must still
    // pass an integrity check. (Its bool return only says whether anything shrank.)
    let _ = db.compact().unwrap();
    check(&db, &model);
    assert!(db.check_integrity().unwrap(), "integrity broken by compact");
    check(&db, &model);
}

#[hegel::test(test_cases = 200)]
fn compact_and_check_integrity_preserve_contents(tc: hegel::TestCase) {
    drive_maintenance(&tc, 15);
}

static FILE_COUNTER: AtomicU64 = AtomicU64::new(0);

fn drive_file_roundtrip(tc: &hegel::TestCase, max_txns: u32) {
    let path = std::env::temp_dir().join(format!(
        "redb-pbt-{}-{}.redb",
        std::process::id(),
        FILE_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    // Fresh path per case; stale files from a previous crashed run are removed first.
    let _ = std::fs::remove_file(&path);

    let db = Database::create(&path).unwrap();
    {
        let wtxn = db.begin_write().unwrap();
        {
            let _ = wtxn.open_table(TABLE).unwrap();
        }
        wtxn.commit().unwrap();
    }
    let model = run_committed_txns(tc, &db, max_txns);
    check(&db, &model);
    drop(db);

    // Reopen from the real file: contents must round-trip through actual file I/O.
    let db = Database::open(&path).unwrap();
    assert_eq!(read_all(&db), model, "file-backed reopen lost data");
    drop(db);
    std::fs::remove_file(&path).unwrap();
}

#[hegel::test(test_cases = 50)]
fn file_backed_database_round_trips(tc: hegel::TestCase) {
    drive_file_roundtrip(&tc, 8);
}
