# `set_host(Some(""))` produces a URL that then fails to re-parse

```rust
use url::Url;

let mut u = Url::parse("foo://user@host/").unwrap();
u.set_host(Some("")).unwrap(); // returns Ok
println!("{u}");
println!("{:?}", Url::parse(u.as_str()).err());
```

prints

```
foo://user@/
Some(EmptyHost)
```

`set_host(Some(""))` returns `Ok` on a non-special URL that has credentials (or a port), but the resulting URL no longer parses — `Url::parse` rejects its own serialization with `EmptyHost`. The port case behaves the same way: `"foo://host:1/"` becomes `"foo://:1/"`.

Tested on url 2.5.8.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
