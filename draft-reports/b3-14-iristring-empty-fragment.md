# `IriAbsoluteStr`/`UriAbsoluteStr` accept strings with an empty fragment

```rust
use iri_string::types::{IriAbsoluteStr, IriStr, UriAbsoluteStr};

fn main() {
    let a = IriAbsoluteStr::new("a:#");
    println!("IriAbsoluteStr::new(\"a:#\") = {:?}", a);

    let b = IriStr::new("a:#").unwrap();
    println!("IriStr::new(\"a:#\").fragment_str() = {:?}", b.fragment_str());

    let c = UriAbsoluteStr::new("http://example.com/#");
    println!("UriAbsoluteStr::new(\"http://example.com/#\") = {:?}", c);

    let d = IriAbsoluteStr::new("a:?#");
    println!("IriAbsoluteStr::new(\"a:?#\") = {:?}", d);
}
```

```
IriAbsoluteStr::new("a:#") = Ok(RiAbsoluteStr("a:#"))
IriStr::new("a:#").fragment_str() = Some("")
UriAbsoluteStr::new("http://example.com/#") = Ok(RiAbsoluteStr("http://example.com/#"))
IriAbsoluteStr::new("a:?#") = Ok(RiAbsoluteStr("a:?#"))
```

`RiAbsoluteStr` (and its URI alias) are documented as "absolute IRI without fragment part", but `new` accepts strings that contain a `#` followed by an empty fragment, such as `"a:#"`, `"a:?#"`, and `"http://example.com/#"`. Parsing the same string as a plain `RiStr` shows it does carry a fragment part: `fragment_str()` returns `Some("")`, not `None`.

Tested on iri-string 0.7.13.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
