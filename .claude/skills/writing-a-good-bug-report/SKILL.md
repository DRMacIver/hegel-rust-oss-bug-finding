---
name: writing-a-good-bug-report
description: >
  Write an upstream bug report or GitHub issue draft for a bug found in someone
  else's project. Use when drafting bug reports, issue text, or reproductions to
  send to a crate's or library's maintainers. Produces a short, plainly-written
  report led by a runnable reproduction — not an LLM-styled writeup with headings
  and diagnosis.
---

# Writing a good bug report

A maintainer should be able to read the whole thing in one pass and reproduce it immediately. Describe what you observed and let them diagnose it. The reproduction is the report; everything else is context around it.

## Shape

In this order, and nothing else:

1. A one-line title stating the observed problem, e.g. `make_contiguous on a full Deque can lead to an out-of-bounds write`. No `Bug:` prefix, no severity label. Keep it short and observed-only: name the API and the symptom, then stop. Cut any justification clause (`because…`, `despite being documented…`, `breaking its own guarantee`), any internal mechanism (`writes its raw u32 into…`), and noise like full generic type parameters or a pasted panic-message string — those go in the body or the output, not the title.
2. A minimal, self-contained, paste-runnable reproduction. This goes near the top — it is the most important part of the report, so don't bury it.
3. The actual output, pasted verbatim from a real run: the panic, error, or wrong value. Not a paraphrase of it. When there's no panic — a wrong return value — make the reproduction `println!` the value and paste the real stdout as its own block below the code. Do not annotate expected output as inline `// like this` comments in the code; that's a paraphrase, not real output. Paste real output, but strip machine-specific noise from it: shorten an absolute dependency path (`/home/.../registry/.../crate-1.2.3/src/x.rs`) to the crate-relative `src/x.rs`, and drop the thread id and the boilerplate `note: run with RUST_BACKTRACE` line from a panic.
4. One or two sentences of observed behaviour, and — where the correct result isn't obvious — what you expected instead. Watch for the "accepts X, then fails later on X" shape (a constructor takes a value, and a later call panics or aborts on it): the reader can't tell whether X should have been rejected up front or supported, so say which. If you're unsure yourself, at least state that accept-then-fail-later is the wrong outcome — that's the part that isn't in doubt.
5. The version or commit you tested on, in one line (`Tested on foo 1.2.3.`). If reproducing needs a non-default feature, note it there (`Tested on foo 1.2.3 (\`serde\` feature).`). Don't add a `[dependencies]` / Cargo.toml block just to repeat the version.

## Style

- No subheadings. No `Summary` / `Steps to reproduce` / `Expected` / `Actual` / `Root cause` / `Impact` scaffolding. Just prose and code blocks.
- Sparing formatting. Avoid bold sprinkled through the text, bullet lists for a single point, and emoji.
- The reproduction must be complete and runnable — full setup, no `// ...` elisions or "assume a Foo here". Keep comments in the code to a minimum; the prose says what happens. A single comment marking the line that fails is fine.
- Describe the observed behaviour; do not diagnose. Don't name the internal field, branch, or source line you think is responsible, don't speculate about the cause, don't rank severity. Maintainers know their own code — a wrong guess wastes their time and even a right one reads as presumptuous.
- Cut the *why*, keep the *when*. Delete sentences that explain the internal cause (`X is implemented in terms of Y`, `the same rounding hits it`, `inherits this`). Keep sentences that narrow the trigger — what input sets it off, and what nearby input does *not* (`a topic of 65535 bytes round-trips fine`). Those help reproduction; the mechanism doesn't.
- Don't grade the bug. Drop adjectives that rate it — `silently`, `corrupt`, `unbounded`, `serious`, `critical`. If a call returned `Ok` and did the wrong thing, the pasted output already shows that; you don't need to say it out loud.
- No speculative cross-references. Don't add "looks related to #1234" — a genuine near-duplicate is a triage decision, not report prose.
- One plain lead-in for an output block: `Output:`, or a short clause like `On a debug build this panics:`. Not `Actual output:` or `Running this program prints the following:`.
- Be concise. If a sentence isn't helping the maintainer reproduce or understand what you saw, cut it.

## Attribution

If you're disclosing the tool used to find the bug, make it a single line at the very end, after the report. It must not be the first thing a reader sees, and it must not be in the title or opening sentence.

## Worked example

````markdown
# `make_contiguous` on a full `Deque` can lead to an out-of-bounds write

The following program aborts on a debug build and writes out of bounds on a release build:

```rust
use heapless::Deque;

let mut q: Deque<i32, 4> = Deque::new();
for i in 0..4 { q.push_back(i).unwrap(); }
for i in 4..7 { q.pop_front(); q.push_back(i).unwrap(); }
q.make_contiguous();
q.pop_front();
q.push_back(99).unwrap();
```

On debug it aborts with:

```
thread 'main' panicked at src/deque.rs:672:35:
unsafe precondition(s) violated: slice::get_unchecked_mut requires that the index is within the slice
thread caused non-unwinding panic. aborting.
```

On release there is no error; the final `push_back` writes past the end of the backing array.

The deque is full (4/4) and wrapped when `make_contiguous` is called. After that call, `is_full()` returns `false` even though the deque still holds 4 elements:

```rust
use heapless::Deque;

let mut q: Deque<i32, 4> = Deque::new();
for i in 0..4 { q.push_back(i).unwrap(); }
for i in 4..7 { q.pop_front(); q.push_back(i).unwrap(); }
q.make_contiguous();
q.pop_front();
q.push_front(3).unwrap();
assert_eq!(q.len(), 4);
assert!(q.is_full()); // fails: returns false
```

The `push_back` in the first example goes ahead only because `is_full()` wrongly reports `false`.

Tested on current `main` and on 0.9.3.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
````

Note what the example does *not* do: it never says which field is corrupted or why, never uses a heading, and puts the tool attribution in a single closing line.
