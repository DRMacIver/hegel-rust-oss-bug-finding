# Draft upstream issue texts (NOT filed — awaiting user approval)

Each draft: title + body ready to paste. All repros verified against the pinned commits in results/runs.md. Style: minimal repro first, cause analysis second, no fix prescriptions unless obvious, credit hegel discreetly at the end.

---

## Trophy 1 — akubera/bigdecimal-rs

**Title:** `normalized()` panics (debug) or silently wraps the scale (release) when scale is near i64::MIN and the value has trailing zeros

**Body:**

```rust
use bigdecimal::BigDecimal;
use num_bigint::BigInt;

let d = BigDecimal::new(BigInt::from(10), i64::MIN);
let n = d.normalized();
```

Debug: panics `attempt to subtract with overflow` at src/lib.rs:926 (`let scale = self.scale - trailing_count as i64;`).

Release: silently wraps, returning `1E-9223372036854775807` (scale `i64::MAX`) for a value equal to `10 * 10^9223372036854775808` — a wrong value rather than a panic.

`BigDecimal::new` accepts any `i64` scale and `normalized()` is documented as value-preserving trailing-zero removal, so this input is in-domain. A checked/saturating subtraction (or returning `self` unchanged when the adjustment would overflow) both seem reasonable. Related in spirit to previously fixed overflow panics #94 and #115.

Found by property-based testing with hegel (https://hegel.dev) — the property was `normalized() == self` under value equality.

---

## Trophy 2 — akubera/bigdecimal-rs (comment on existing issue #44, not a new issue)

**Comment on #44 ("Divide by zero results in zero"):**

`Div` now panics, but the primitive-RHS `DivAssign` overloads still have the old behavior — they silently set the value to zero:

```rust
use bigdecimal::BigDecimal;
use std::str::FromStr;

let mut d = BigDecimal::from_str("7").unwrap();
d /= 0i32;          // no panic
assert_eq!(d, BigDecimal::from(0));   // passes!
```

while `BigDecimal::from(7) / BigDecimal::from(0)` panics `"Division by zero"`. The zeroing branch is explicit in the `(IMPL:DIV-ASSIGN $t)` macro arm (src/impl_ops.rs:298): `if rhs.is_zero() { *self = BigDecimal::zero() }`. As noted above, `inverse()` on zero has the same issue. Whichever behavior is intended, `/` and `/=` presumably shouldn't disagree.

(Found while property-testing operator-overload consistency with hegel, https://hegel.dev.)

---

## Trophy 3 — akubera/bigdecimal-rs

**Title:** `Hash` panics on scale i64::MIN and allocates |scale| bytes for large negative scales

**Body:**

```rust
use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

let d = BigDecimal::new(BigInt::from(1), i64::MIN);
let mut h = DefaultHasher::new();
d.hash(&mut h);   // debug: panics "attempt to negate with overflow"
```

Cause: the `Hash` impl (src/lib.rs, `dec_str.push_str(&"0".repeat(self.scale.abs() as usize))`) — `scale.abs()` overflows at `i64::MIN`.

Separately, the same line means hashing any value with a moderately large negative scale materializes |scale| zero bytes: `BigDecimal::new(1.into(), -1_000_000_000)` allocates ~1 GB to compute a hash. For hashed untrusted values this is a denial-of-service vector; hashing the (int_val, scale) pair of the normalized value would avoid both problems. Possibly worth fixing together with #143 (Hash equal-suffix collisions), which is a different defect in the same impl.

Found by property-based testing with hegel (https://hegel.dev) — hash/eq consistency property.

---

## Trophy 4 — toml-rs/toml (toml_datetime)

**Title:** toml_datetime: documented `Offset::Custom` minutes range includes -1440, which Displays as unparseable `-24:00`

**Body:**

`Offset::Custom`'s field is documented as `Minutes: -1_440..1_440` (crates/toml_datetime/src/datetime.rs:175). At the documented lower bound:

```rust
use toml_datetime::*;
use std::str::FromStr;

let dt = Datetime {
    date: Some(Date { year: 2020, month: 1, day: 1 }),
    time: Some(Time { hour: 0, minute: 0, second: Some(0), nanosecond: Some(0) }),
    offset: Some(Offset::Custom { minutes: -1440 }),
};
let s = dt.to_string();               // "2020-01-01T00:00:00.0-24:00"
Datetime::from_str(&s).unwrap_err();  // "hours between 00 and 23"
```

RFC 3339 caps offset hours at 23, so only ±1439 minutes is representable in `HH:MM` form; either the doc range should be `-1439..1440` (or `-1439..=1439`), or Display/parsing should handle the boundary. Doc fix seems most likely intended.

Found by property-based testing with hegel (https://hegel.dev) — Display/FromStr roundtrip over the documented field ranges.

---

## Trophy 5 — apache/datafusion-sqlparser-rs

**Title:** Display for bracket-quoted identifiers does not escape `]`, breaking the parse↔display roundtrip

**Body:**

The tokenizer folds `]]` to `]` inside bracket-quoted identifiers (src/tokenizer.rs, `parse_quoted_ident`), but `Display for Ident`'s `Some('[')` arm writes `[{value}]` with no escaping (src/ast/mod.rs:388), so the roundtrip breaks even for ASTs produced by the parser itself:

```rust
use sqlparser::dialect::MsSqlDialect;
use sqlparser::parser::Parser;

let ast = Parser::parse_sql(&MsSqlDialect {}, "SELECT [a]]b]").unwrap();
let sql2 = ast[0].to_string();        // "SELECT [a]b]"
Parser::parse_sql(&MsSqlDialect {}, &sql2).unwrap_err();
// Expected: end of statement, found: ]
```

`Display for Word` (src/tokenizer.rs:474) has the same asymmetry. The single-quote/double-quote paths escape by doubling; the bracket path just needs the matching `]` → `]]` escape on output.

Found by property-based testing with hegel (https://hegel.dev) — display→reparse AST identity over generated quoted identifiers.

---

## Trophy 6 — RoaringBitmap/roaring-rs

**Title:** remove_smallest/remove_biggest corrupt run containers when the amount exactly consumes an interval (silent corruption in release)

**Body:**

```rust
use roaring::RoaringBitmap;

let mut b = RoaringBitmap::new();
b.insert_range(0..=2);
b.insert(4);
b.remove_smallest(3);
// debug: panic "attempt to subtract with overflow" at src/bitmap/store/interval_store.rs:966
// release: b now contains [3, 4, 5, 6, ..., ~65535] — tens of thousands of phantom values
```

Mirror case for `remove_biggest`: build `{2, 4, 5, 6, 7, 8}`, `optimize()` (run container), then `remove_biggest(5)`.

Cause: in `interval_store.rs`, `remove_smallest` (line ~249) handles the `run_len() == amount` case with `last_interval.start += amount`, leaving an interval with `start == end + 1` instead of removing it; `Interval::run_len` (`end - start + 1`, line 966) then underflows — panicking under debug assertions and wrapping to 65536 in release, which materializes a full-container phantom run. `remove_biggest` (line ~273) has the mirrored defect. `RoaringTreemap` is affected through delegation.

Found by property-based testing with hegel (https://hegel.dev) — a stateful model test against `BTreeSet<u32>` plus remove_smallest/remove_biggest iterator-oracle properties with run-length-exact amounts.
