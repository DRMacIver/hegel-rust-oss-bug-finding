# `try_finish` panics on a second empty frame

`FrameEncoder` writes lz4 frames and supports more than one per stream: `try_finish` ends the current frame and a further `write` starts a new one (a non-empty `write`/`try_finish`/`write`/`try_finish` works). The same sequence with empty writes:

```rust
use lz4_flex::frame::FrameEncoder;
use std::io::Write;

fn main() {
    let mut enc = FrameEncoder::new(Vec::new());
    enc.write_all(b"").unwrap();
    enc.try_finish().unwrap();
    enc.write_all(b"").unwrap();
    enc.try_finish().unwrap();
}
```

On a debug build this panics:

```
thread 'main' panicked at src/frame/compress.rs:210:9:
assertion failed: self.is_frame_open
```

The first `try_finish` on an empty frame succeeds, so the second — in the same state — shouldn't panic.

On a release build there is no panic, but the second `try_finish` appends four bytes that a single `write_all(b"")` + `try_finish()` doesn't. One call gives:

```
[4, 34, 77, 24, 96, 64, 130, 0, 0, 0, 0, 0, 0, 0, 0]
```

Two calls (as in the reproduction above) give:

```
[4, 34, 77, 24, 96, 64, 130, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
```

Tested on lz4_flex 0.14.0.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
