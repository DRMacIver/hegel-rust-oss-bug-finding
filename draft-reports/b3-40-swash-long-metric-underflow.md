# `GlyphMetrics::advance_width` panics on a minimal font accepted by `FontRef::from_index`

```rust
use swash::FontRef;

fn main() {
    // Minimal sfnt: version tag 0x00010000, numTables = 0, no table directory entries.
    // FontRef::from_index accepts this as a valid font.
    let data: [u8; 12] = [0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let font = FontRef::from_index(&data, 0).unwrap();
    let metrics = font.glyph_metrics(&[]);
    let advance = metrics.advance_width(0);
    println!("advance = {advance}");
}
```

On a debug build this panics:

```
thread 'main' panicked at src/internal/xmtx.rs:14:9:
attempt to subtract with overflow
```

On a release build there is no panic; it prints `advance = 0`.

`FontRef::from_index` returns `Some` for this data, and `glyph_metrics` also succeeds; the panic only shows up once `advance_width` is called. Since `from_index` accepts the font, `advance_width` on it shouldn't panic.

Tested on swash 0.2.10.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
