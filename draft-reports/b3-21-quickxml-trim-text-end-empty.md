# `trim_text_end` still emits an empty `Text` event for whitespace-only text before a tag

```rust
use quick_xml::events::Event;
use quick_xml::reader::Reader;

fn main() {
    let mut reader = Reader::from_str(" <a/>");
    reader.config_mut().trim_text_end = true;

    loop {
        match reader.read_event() {
            Ok(Event::Eof) => break,
            Ok(e) => println!("{:?}", e),
            Err(e) => {
                println!("Error: {:?}", e);
                break;
            }
        }
    }
}
```

Output:

```
Text(BytesText { content: Borrowed("") })
Empty(BytesStart { buf: Borrowed("a"), name_len: 1 })
```

`Config::trim_text_end`'s documentation says: "When set to `true`, trailing whitespace is trimmed in `Text` events. If after that the event is empty it will not be pushed." Here the leading space is whitespace-only text preceding `<a/>`, and `trim_text_end` is set, but the reader still emits a `Text("")` event before the `Empty` event.

(#755 fixed a related all-space-trimming case in 0.33.0; this leading-space-before-a-self-closing-tag case still reproduces.)

Tested on quick-xml 0.41.0.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
