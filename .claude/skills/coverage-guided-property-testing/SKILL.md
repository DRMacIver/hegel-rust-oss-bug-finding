---
name: coverage-guided-property-testing
description: >
  Close the last uncovered regions of a dependency under property test by
  measuring per-file region coverage of the DEPENDENCY (not your crate),
  reading why each cold region is cold, and writing property-shaped tests —
  with a real oracle, not mere calls — that reach it. Use after a broad
  model-based/metamorphic harness plateaus: the least-covered files are where
  the untested-code bugs live (this technique found two memory-safety bugs in
  hecs's least-covered file within one session).
---

# Coverage-guided property testing (of a dependency, with hegel)

A broad harness plateaus with the "famous" paths saturated and a long tail of
API surface never touched. That tail is exactly where bugs survive: it's the
code the crate's own tests don't exercise either. Treat dependency coverage
as a *map of where to aim the next oracle*, never as the goal itself.

## Measure the dependency, not your crate

```
cargo llvm-cov --tests --dep-coverage <crate> --summary-only   # per-file table
cargo llvm-cov report --dep-coverage <crate> --text            # annotated source, reuses last run
```

Rank files by missed regions. In the annotated output, grep for `| 0|` lines
and read each zero-count region **in the dependency's source** to classify it:

1. **API path never taken** — a public method/impl your harness never calls
   (the common case, and the actionable one).
2. **Unreachable without a feature/config** — e.g. a rayon feature that
   doesn't exist in this version. Record it as non-existent in your notes so
   nobody burns time on it again.
3. **Deprecated aliases** — covering `fn old_name()` that forwards to the new
   name buys nothing; skip deliberately.
4. **Error/Debug formatting arms** — cheap to hit (format the error) and
   occasionally reveal real bugs; batch them into an API-surface test.

## Write properties, not calls

The trap is "coverage tests" that invoke the cold function and assert
nothing. Every new call must sit inside an oracle you already trust:

- **Ground-truth against a drawn spec**: builder introspection (`has`/`get`/
  `component_types`), bundle-vs-query satisfaction — assert they agree with
  the spec that generated the value.
- **Project through an observational fingerprint**: every *read shape*
  (query combinators, views, whole-column access, prepared variants) is a
  projection of the same state — assert each returns exactly the fingerprint-
  predicted set/values, no duplicates. One fingerprint funds dozens of shape
  checks (see `metamorphic-and-differential-testing`).
- **Differential within the API**: `add_bundle(tuple) ≡` individual `add`s;
  `clone()`d builder spawns identically; batch ≡ loop.
- **Resource oracle stays on**: keep the Drop-tracked component in every new
  test; builder/buffer paths that were never covered are exactly where leaks
  and double-drops hide.

Validate each new suite once with a planted bug (wrong relation or wrong
path), watch it fail and shrink, then revert.

## Run every new suite under Miri immediately

Cold regions are disproportionately unsafe code, and some findings are
**Miri-only**: in hecs, `EntityBuilderClone::clone` on an empty builder
executes zero-size `alloc` — native tests pass silently; Miri flags the UB on
the first run. Give each test binary a small `#[cfg(miri)]` entry point
(`test_cases = 8`-ish, suppress hegel's TooSlow check) from day one, and run
`cargo +nightly miri test --test <name>` with
`MIRIFLAGS="-Zmiri-permissive-provenance -Zmiri-disable-isolation"` as part
of landing it.

## What the payoff looks like

One session on hecs 0.11.0: region coverage 81.3% → 91.2%, and the two
lowest-coverage areas yielded two unreported memory-safety bugs — a stale
internal index after `BuiltEntityClone -> EntityBuilderClone` conversion
(wrong-slot reads; OOB write path), and the zero-size `alloc` UB above. Both
were within a few dozen lines of regions that had literally never executed
under the whole prior suite.

## When a cold region turns out to be a bug you can't keep executing

UB cannot live in a green suite (Miri would stay red). Pattern: keep the
correct-contract property for the healthy part of the surface; guard the
specific UB-triggering input shape with a **loud comment** naming the draft
report; pin any safely-observable wrong behavior in a dedicated
`*_observation` test that asserts the *current* buggy output and instructs
the reader to flip it when upstream fixes it. Never silently narrow a
generator — every guard must carry the bug reference.

## See also

- `stateful-model-based-testing` — the broad harness this technique extends.
- `metamorphic-and-differential-testing` — the fingerprint and spec-ground-
  truth oracles that make the new calls meaningful.
