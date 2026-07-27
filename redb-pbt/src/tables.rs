//! COVERAGE-GUIDED BREADTH: table administration — create/delete/rename/list — against
//! a model of the table CATALOG (`BTreeMap<name, BTreeMap<u64,u64>>` over a small name
//! universe). Oracles: `delete_table` returns whether the table existed; `rename_table`
//! errors with `TableDoesNotExist` (missing source) / `TableExists` (present target)
//! and otherwise moves the contents; `list_tables` equals the model's name set after
//! every round; each listed table's contents equal its model entry.

use hegel::generators as gs;
use redb::{
    backends::InMemoryBackend, Database, ReadableDatabase, ReadableTable, TableDefinition,
    TableError, TableHandle,
};
use std::collections::BTreeMap;

const NAMES: [&str; 3] = ["ta", "tb", "tc"];

fn def(name: &str) -> TableDefinition<'_, u64, u64> {
    TableDefinition::new(name)
}

fn draw_name(tc: &hegel::TestCase) -> &'static str {
    NAMES[tc.draw(
        gs::integers::<usize>()
            .min_value(0)
            .max_value(NAMES.len() - 1),
    )]
}

fn drive_tables(tc: &hegel::TestCase, max_rounds: u32) {
    let db = Database::builder()
        .create_with_backend(InMemoryBackend::new())
        .unwrap();
    let mut catalog: BTreeMap<&'static str, BTreeMap<u64, u64>> = BTreeMap::new();

    let rounds = tc.draw(gs::integers::<u32>().min_value(0).max_value(max_rounds));
    for _ in 0..rounds {
        let wtxn = db.begin_write().unwrap();
        match tc.draw(gs::integers::<u8>().min_value(0).max_value(3)) {
            // Upsert data into a drawn table (open_table creates it if absent).
            0 | 1 => {
                let name = draw_name(tc);
                let entry = catalog.entry(name).or_default();
                {
                    let mut table = wtxn.open_table(def(name)).unwrap();
                    let ops = tc.draw(gs::integers::<u32>().min_value(0).max_value(4));
                    for _ in 0..ops {
                        let k = tc.draw(gs::integers::<u64>().min_value(0).max_value(15));
                        let v = tc.draw(gs::integers::<u64>().min_value(0).max_value(1000));
                        table.insert(k, v).unwrap();
                        entry.insert(k, v);
                    }
                }
                wtxn.commit().unwrap();
            }
            // Delete a drawn table; returns whether it existed.
            2 => {
                let name = draw_name(tc);
                let existed = wtxn.delete_table(def(name)).unwrap();
                wtxn.commit().unwrap();
                assert_eq!(
                    existed,
                    catalog.remove(name).is_some(),
                    "delete_table({name}) existence"
                );
            }
            // Rename src -> dst, with the full error contract.
            _ => {
                let src = draw_name(tc);
                let dst = draw_name(tc);
                let result = wtxn.rename_table(def(src), def(dst));
                match (catalog.contains_key(src), catalog.contains_key(dst)) {
                    (false, _) => {
                        assert!(
                            matches!(result, Err(TableError::TableDoesNotExist(ref n)) if n == src),
                            "rename missing {src}: {result:?}"
                        );
                        wtxn.abort().unwrap();
                    }
                    // NB: src == dst with src present falls here (dst present too).
                    (true, true) => {
                        assert!(
                            matches!(result, Err(TableError::TableExists(ref n)) if n == dst),
                            "rename onto existing {dst}: {result:?}"
                        );
                        wtxn.abort().unwrap();
                    }
                    (true, false) => {
                        result.unwrap();
                        wtxn.commit().unwrap();
                        let contents = catalog.remove(src).unwrap();
                        catalog.insert(dst, contents);
                    }
                }
            }
        }

        // Catalog oracle: list_tables == model names; each table's contents match.
        let rtxn = db.begin_read().unwrap();
        let mut listed: Vec<String> = rtxn
            .list_tables()
            .unwrap()
            .map(|h| h.name().to_string())
            .collect();
        listed.sort();
        let want: Vec<String> = catalog.keys().map(|n| n.to_string()).collect();
        assert_eq!(listed, want, "list_tables");
        for (name, entries) in &catalog {
            let table = rtxn.open_table(def(name)).unwrap();
            let mut got = BTreeMap::new();
            for row in table.iter().unwrap() {
                let (k, v) = row.unwrap();
                got.insert(k.value(), v.value());
            }
            assert_eq!(&got, entries, "contents of table {name}");
        }
    }
}

#[hegel::test(test_cases = 400)]
fn table_catalog_matches_model(tc: hegel::TestCase) {
    drive_tables(&tc, 30);
}
