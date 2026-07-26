# `Algorithm::Lcs` and `Algorithm::Hunt` panic on `utils::diff_chars` (and `diff_words`/`diff_lines`) for two empty inputs

```rust
use similar::{Algorithm, TextDiff};
use similar::utils::diff_chars;

fn main() {
    let diff = TextDiff::configure().algorithm(Algorithm::Lcs).diff_chars("", "");
    println!("ops: {:?}", diff.ops());

    let result = diff_chars(Algorithm::Lcs, "", "");
    println!("{:?}", result);
}
```

```
ops: [Delete { old_index: 0, old_len: 0, new_index: 0 }]
thread 'main' panicked at src/utils.rs:211:45:
slice out of bounds
```

For two empty strings, `TextDiff::configure().algorithm(Algorithm::Lcs).diff_chars("", "")` produces a single `Delete { old_index: 0, old_len: 0, new_index: 0 }` op, whereas `Algorithm::Myers` and `Algorithm::Patience` produce no ops at all for the same input. `utils::diff_chars` (and `diff_words`/`diff_lines`) then panics with "slice out of bounds". `Algorithm::Hunt` shows the same op and the same panic; `Algorithm::Myers` and `Algorithm::Patience` return `Ok([])`.

Tested on similar 3.1.1.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
