# `TableReference` with an empty part does not round-trip through `Display`/`parse_str`

```rust
use datafusion_common::TableReference;

fn main() {
    let tr = TableReference::partial("a", "");
    let s = tr.to_string();
    println!("original: {:?}", tr);
    println!("displayed: {:?}", s);

    let parsed = TableReference::parse_str(&s);
    println!("parsed: {:?}", parsed);

    assert_eq!(tr, parsed, "roundtrip through Display/parse_str should preserve the reference");
}
```

```
original: Partial { schema: "a", table: "" }
displayed: "a."
parsed: Bare { table: "a" }

thread 'main' panicked at src/main.rs:12:5:
assertion `left == right` failed: roundtrip through Display/parse_str should preserve the reference
  left: Partial { schema: "a", table: "" }
 right: Bare { table: "a" }
```

`TableReference::partial("a", "")` displays as `"a."`, and parsing that string back gives `Bare { table: "a" }` instead of the original `Partial { schema: "a", table: "" }` — the empty table part is dropped and the schema is reinterpreted as the table name.

Tested on `datafusion-common` 54.1.0.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
