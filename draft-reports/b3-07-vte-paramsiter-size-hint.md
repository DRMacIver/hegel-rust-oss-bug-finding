# `ParamsIter::size_hint()` disagrees with the number of items actually yielded

```rust
use vte::{Params, Parser, Perform};

struct Handler;

impl Perform for Handler {
    fn csi_dispatch(&mut self, params: &Params, _intermediates: &[u8], _ignore: bool, action: char) {
        let count = params.iter().count();
        let (lower, upper) = params.iter().size_hint();
        println!(
            "CSI {:?}: iter().count() = {}, size_hint() = ({}, {:?})",
            action, count, lower, upper
        );
    }
}

fn main() {
    let mut statemachine = Parser::new();
    let mut handler = Handler;

    // CSI sequence: ESC [ 0:0 m
    let input = b"\x1b[0:0m";
    statemachine.advance(&mut handler, input);
}
```

Output:

```
CSI 'm': iter().count() = 1, size_hint() = (2, Some(2))
```

`params.iter()` actually yields 1 item for this CSI sequence, but `size_hint()` reports a lower bound of 2 and an exact upper bound of `Some(2)`, so the iterator's own `size_hint` disagrees with what iterating it actually produces.

Tested on vte 0.15.0.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
