# A UTF-8 encoded C1 control byte is dispatched as `print` or `execute` depending on how the input is chunked

```rust
use vte::{Params, Parser, Perform};

#[derive(Default)]
struct Log(Vec<String>);

impl Perform for Log {
    fn print(&mut self, c: char) {
        self.0.push(format!("print:{:?}", c));
    }
    fn execute(&mut self, byte: u8) {
        self.0.push(format!("exec:{:#x}", byte));
    }
    fn hook(&mut self, _: &Params, _: &[u8], _: bool, _: char) {}
    fn put(&mut self, _: u8) {}
    fn unhook(&mut self) {}
    fn osc_dispatch(&mut self, _: &[&[u8]], _: bool) {}
    fn csi_dispatch(&mut self, _: &Params, _: &[u8], _: bool, _: char) {}
    fn esc_dispatch(&mut self, _: &[u8], _: bool, _: u8) {}
}

fn main() {
    // Whole input in one advance() call.
    let mut parser = Parser::new();
    let mut performer = Log::default();
    parser.advance(&mut performer, &[0xC2, 0x80]);
    println!("whole:   {:?}", performer.0);

    // Same bytes, split across two advance() calls.
    let mut parser = Parser::new();
    let mut performer = Log::default();
    parser.advance(&mut performer, &[0xC2]);
    parser.advance(&mut performer, &[0x80]);
    println!("chunked: {:?}", performer.0);
}
```

Output:

```
whole:   ["exec:0x80"]
chunked: ["print:'\\u{80}'"]
```

`[0xC2, 0x80]` is the UTF-8 encoding of U+0080. Fed to `advance` in a single call it produces an `execute` event; fed as `[0xC2]` then `[0x80]` in two separate `advance` calls it produces a `print` event instead, for the identical byte sequence. This happens for the whole U+0080..=U+009F range whenever the two-byte encoding is split across `advance` calls.

Tested on vte 0.15.0.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
