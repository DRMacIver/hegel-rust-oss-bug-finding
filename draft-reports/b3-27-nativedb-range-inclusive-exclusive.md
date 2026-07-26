# `range(..=end)` scan misses an item whose key equals `end`

```rust
use native_db::*;
use native_model::{native_model, Model};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[native_model(id = 1, version = 1)]
#[native_db]
struct Item {
    #[primary_key]
    id: u32,
}

static MODELS: Lazy<Models> = Lazy::new(|| {
    let mut models = Models::new();
    models.define::<Item>().unwrap();
    models
});

fn main() -> Result<(), db_type::Error> {
    let db = Builder::new().create_in_memory(&MODELS)?;

    let rw = db.rw_transaction()?;
    rw.insert(Item { id: 0 })?;
    rw.commit()?;

    let r = db.r_transaction()?;
    let mut results: Vec<Item> = Vec::new();
    for item in r.scan().primary()?.range(..=0u32)? {
        results.push(item?);
    }
    println!("range(..=0) with one item pk=0 returned: {:?}", results);

    Ok(())
}
```

Output:

```
range(..=0) with one item pk=0 returned: []
```

The database contains a single item with primary key `0`. Scanning with `range(..=0u32)` returns an empty result, as if the range were exclusive of its end bound. The same happens scanning a unique secondary key with `range(..=0u32)` over an item whose secondary key is `0`.

Tested with native_db 0.8.2 (native_model 0.4.20).

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
