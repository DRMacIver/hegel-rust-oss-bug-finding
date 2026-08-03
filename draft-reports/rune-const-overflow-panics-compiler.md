# Compiling a `const` with overflowing integer arithmetic panics the compiler

Compiling a `const` whose initializer overflows `i64` panics the compiler on a debug build, and accepts the wrapped value on a release build:

```rust
use rune::{Context, Diagnostics, Source, Sources};

fn main() {
    let context = Context::with_default_modules().unwrap();
    let mut sources = Sources::new();
    sources
        .insert(Source::memory("pub const VALUE = 9223372036854775807 + 1;").unwrap())
        .unwrap();
    let mut diagnostics = Diagnostics::new();
    let _ = rune::prepare(&mut sources)
        .with_context(&context)
        .with_diagnostics(&mut diagnostics)
        .build();
}
```

On a debug build this panics:

```
thread 'main' panicked at src/compile/ir/eval.rs:84:57:
attempt to add with overflow
```

On a release build `build()` returns `Ok` and the constant takes the wrapped value.

The same overflowing arithmetic evaluated at runtime (e.g. `9223372036854775807 + 1` in a function body) returns an error rather than panicking, so I'd expect a `const` initializer to report an error here too instead of panicking (debug) or wrapping (release).

Tested on rune 0.14.2 and on current `main`.

This was another bug found with Hegel while preparing the follow-on PR discussed in #1030.
