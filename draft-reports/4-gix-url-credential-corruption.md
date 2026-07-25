# A colon in the user component is not preserved by `to_bstring()`

```rust
let url = gix_url::parse("http://a%3Ab@host/").unwrap();
assert_eq!(url.user(), Some("a:b"));

let serialized = url.to_bstring();
let reparsed = gix_url::parse(&serialized).unwrap();
assert_eq!(reparsed, url);
```

The final assertion fails. `to_bstring()` produces `http://a:b@host/` — the colon in the user is written literally rather than percent-encoded — and re-parsing that splits the user component in two: `reparsed.user()` is `Some("a")` and `reparsed.password()` is `Some("b")`.

`write_to` is documented as writing the URL "losslessly … ready to be parsed again", but here a user containing a colon does not survive the round-trip, and the trailing part silently becomes a password.

Tested on gix-url 0.37.0.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
