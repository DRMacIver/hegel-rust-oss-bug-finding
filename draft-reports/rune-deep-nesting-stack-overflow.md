<!-- FILED 2026-08-03 as https://github.com/rune-rs/rune/issues/1040 -->

# Deeply nested expressions abort the process with a stack overflow at compile time

Compiling a deeply nested expression overflows the stack and aborts the process instead of returning a compile error:

```rust
use rune::{Context, Diagnostics, Source, Sources};

fn main() {
    let depth = 100_000;
    let src = format!("pub fn main() {{ {}0{} }}", "-(".repeat(depth), ")".repeat(depth));

    let context = Context::with_default_modules().unwrap();
    let mut sources = Sources::new();
    sources.insert(Source::memory(&src).unwrap()).unwrap();
    let mut diagnostics = Diagnostics::new();
    let _ = rune::prepare(&mut sources)
        .with_context(&context)
        .with_diagnostics(&mut diagnostics)
        .build();
}
```

Output:

```
thread 'main' has overflowed its stack
fatal runtime error: stack overflow, aborting
```

The process is killed with SIGABRT, which a host embedding rune cannot catch, so script or REPL input with enough nesting takes the host process down. I'd expect a recursion-depth/nesting-limit error instead. Parenthesized groups `(((...0...)))` and other nested shapes abort the same way; the depth required depends on the available stack.

Tested on rune 0.14.2 and on current `main`.

This was another bug found with Hegel while preparing the follow-on PR discussed in #1030.
