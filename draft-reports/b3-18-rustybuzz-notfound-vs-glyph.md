# `set_not_found_variation_selector_glyph` produces a `glyph_id` above `u16::MAX`

```rust
use rustybuzz::{shape, Face, SerializeFlags, UnicodeBuffer};

fn main() {
    // small published font with no UVS entry for A + U+FE00
    let face = Face::from_slice(font_test_data::CMAP14_FONT1, 0).unwrap();

    let mut buffer = UnicodeBuffer::new();
    buffer.push_str("A\u{FE00}");
    buffer.set_not_found_variation_selector_glyph(0x1_0000);

    let output = shape(&face, &[], buffer);
    for info in output.glyph_infos() {
        println!("glyph_id = {}", info.glyph_id);
    }
    let serialized = output.serialize(&face, SerializeFlags::default());
    println!("serialized = {serialized}");
}
```

with `rustybuzz = "0.20"` and `font-test-data = "0.8"`.

On a debug build it panics in `serialize`:

```
glyph_id = 0
glyph_id = 65536
thread 'main' panicked at src/hb/buffer.rs:203:9:
assertion failed: self.glyph_id <= u32::from(u16::MAX)
```

On a release build there is no panic, and the out-of-range glyph id appears in the output:

```
glyph_id = 0
glyph_id = 65536
serialized = gid0=0+1500|gid65536=0+0
```

`glyph_id` is documented as "Guarantee to be <= `u16::MAX`". Here `set_not_found_variation_selector_glyph(0x1_0000)`, when the variation selector (`A` + `U+FE00`) isn't a registered sequence in the font, gives a `glyph_id` of 65536. On a debug build that trips the crate's own `debug_assert!`; on a release build the value 65536 surfaces in the output.

Tested on rustybuzz 0.20.1.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
