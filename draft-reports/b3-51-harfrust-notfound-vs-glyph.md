# `set_not_found_variation_selector_glyph` produces a `glyph_id` above `u16::MAX`

```rust
use harfrust::font::{Font, FontInstance};
use harfrust::{shape, SerializeFlags, ShapeOptions, UnicodeBuffer};

fn main() {
    // small published font with no UVS entry for A + U+FE00
    let font = Font::new(font_test_data::CMAP14_FONT1.to_vec(), 0).unwrap();
    let instance = FontInstance::builder(&font).build();

    let mut buffer = UnicodeBuffer::new();
    buffer.push_str("A\u{FE00}");
    buffer.guess_segment_properties();
    buffer.set_not_found_variation_selector_glyph(0x1_0000);

    let glyphs = shape(&instance, buffer, ShapeOptions::new());
    for info in glyphs.glyph_infos() {
        println!("glyph_id = {}", info.glyph_id);
    }
    println!("serialized = {}", glyphs.serialize(&instance, SerializeFlags::empty()));
}
```

with `harfrust = { version = "0.12", features = ["experimental_font_api"] }` and `font-test-data = "0.8"`. It prints (debug and release alike):

```
glyph_id = 0
glyph_id = 65536
serialized = [gid0=0+1500|gid65536=0+0]
```

`GlyphInfo::glyph_id` is documented as "Guarantee to be <= `u16::MAX`". Here `set_not_found_variation_selector_glyph(0x1_0000)`, when the variation selector (`A` + `U+FE00`) isn't a registered sequence in the font, gives a `glyph_id` of 65536 — above the documented maximum. The value surfaces in `glyph_infos()` and in the serialized output.

(Originally reported to rustybuzz as harfbuzz/rustybuzz#168, where the maintainer pointed here.)

Tested on harfrust 0.12.0.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
