# `Stack::clear_snapshot` drops elements belonging to an outer snapshot

```rust
use pest::Stack;

let mut stack = Stack::new();
stack.push(0);
stack.snapshot();
stack.snapshot();
stack.pop();
stack.clear_snapshot();
stack.restore();
assert_eq!(stack.peek(), Some(&0));
```

The final assertion fails:

```
assertion `left == right` failed
  left: None
 right: Some(0)
```

The outer `snapshot()` was taken when the stack was `[0]`, so after `restore()` the stack should be `[0]` again. Instead it is empty.

This is reachable from a real grammar. With `pest_derive`:

```
main = { PUSH("a") ~ ( POP? ~ "X" )? ~ POP ~ EOI }
```

parsing the input `"aa"` panics:

```
thread 'main' panicked at src/parser_state.rs:1577:14:
pop was called on empty stack
```

The optional `( POP? ~ "X" )?` group fails at `"X"` and is rolled back, which should also roll the stack back to the `"a"` pushed by `PUSH`, leaving it for the final `POP`. Instead the `"a"` is gone and the final `POP` runs against an empty stack.

Tested on pest 2.8.8.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
