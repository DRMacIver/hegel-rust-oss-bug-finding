# Negating `i64::MIN` in a Rune script panics the host in debug and wraps in release

```rust
use rune::{Context, Diagnostics, Source, Sources, Vm};
use rune::termcolor::{ColorChoice, StandardStream};
use std::sync::Arc;

fn main() -> rune::support::Result<()> {
    let context = Context::with_default_modules()?;
    let runtime = Arc::new(context.runtime()?);

    let mut sources = Sources::new();
    sources.insert(Source::memory(
        r#"
        pub fn main() {
            let a = -9223372036854775808;
            -a
        }
        "#,
    )?)?;

    let mut diagnostics = Diagnostics::new();

    let result = rune::prepare(&mut sources)
        .with_context(&context)
        .with_diagnostics(&mut diagnostics)
        .build();

    if !diagnostics.is_empty() {
        let mut writer = StandardStream::stderr(ColorChoice::Always);
        diagnostics.emit(&mut writer, &sources)?;
    }

    let unit = result?;
    let unit = Arc::new(unit);
    let mut vm = Vm::new(runtime, unit);

    let output = vm.call(["main"], ())?;
    println!("{:?}", output);
    Ok(())
}
```

On a debug build this panics:

```
thread 'main' panicked at core/src/ops/arith.rs:729:1:
attempt to negate with overflow
```

On a release build it prints the unchanged value instead:

```
-9223372036854775808
```

For comparison, an integer overflow through `+` on the same VM (e.g. `9223372036854775807 + 1`) does not panic in either profile; `vm.call` returns `Err(VmError { ... kind: Overflow ... })`. Unary negation of `i64::MIN` is the odd one out: it panics the embedding host application in debug and produces a wrong value in release.

Tested on `rune` 0.14.2.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
