---
name: stateful-model-based-testing
description: >
  Build a stateful, model-based property test for a stateful Rust API (a data
  structure, store, engine, or VM) with hegel — drive a random sequence of
  operations against both the real system and a simpler reference model, and
  assert equivalence plus invariants after every step. Use when a system has
  observable state that a simpler model can mirror, and you want to test it to
  an unreasonable degree rather than write a handful of examples.
---

# Stateful model-based testing (with hegel)

The idea: a complex stateful system usually has a much simpler *specification* you
can run alongside it. Drive the same randomly-generated sequence of operations into
both, and after **every** operation assert they still agree. The reference model is
the oracle; hegel generates and shrinks the sequences. Bugs surface as a divergence,
and hegel shrinks it to a minimal failing sequence.

This is the highest-leverage PBT technique for stateful code, and the place to develop
reusable harness skill. The running example below is a hecs (ECS) harness, but the
method is the same for a KV store (model = `BTreeMap`), a rope (model = `String`), a
CRDT (model = the abstract value), etc.

## The shape

1. **Reference model.** The smallest data structure that captures what a caller can
   observe. For hecs: `HashMap<Entity, {components...}>`. For a KV store: `BTreeMap`.
   If the model needs the full internal state to stay in sync, it's too weak an oracle —
   pick observations, not internals.
2. **Operation loop.** In one `#[hegel::test]`, draw operations one at a time and apply
   each to *both* the system and the model:
   ```rust
   for _ in 0..tc.draw(gs::integers::<u32>().min_value(0).max_value(MAX_STEPS)) {
       match tc.draw(gs::integers::<u8>().min_value(0).max_value(N)) {
           0 => { /* op: apply to world AND model */ }
           ...
       }
       check(&world, &model);   // after EVERY op, not just at the end
   }
   ```
   Draw-in-the-loop (not a pre-generated `Vec<Op>`) is the idiomatic hegel form and
   shrinks well.
3. **Check after every op.** Cheap per-step checks localise a failure to the exact
   operation that broke the invariant — far more useful than a single end check.

## Oracle families — layer several; each catches a different bug class

- **Bidirectional equivalence + size.** Every modelled entry is present in the system
  with the right value; every entry the system reports is in the model; sizes match.
  One direction alone misses "extra" or "missing" bugs.
- **Per-operation result.** Check the op's own return (ok/err, removed/not-removed)
  against the model, not just the resulting state. Catches wrong error reporting.
- **Structural / internal invariants the system exposes.** Use the system's own
  introspection: hecs `archetypes()` must *partition* the live entities (each id in
  exactly one; `Σ archetype.len() == world.len()`); a B-tree's balance bounds; a store's
  `check_integrity()`. These are independent of the model and catch corruption the
  model can't see.
- **Resource / Drop oracle.** Add a non-`Copy` component/value whose `Drop` and
  constructor bump a per-thread counter; assert `live_count == modelled_count` after
  every op. Catches leaks (missed drop on migration/despawn) and double-drops in unsafe
  code — exactly the bugs a value-only model can't. (Reset/assert the counter is 0 at
  the *start* of each test case: thread-locals persist across hegel's many cases.)
- **Derived-view sub-oracles.** For each way to read the state (each query shape:
  `&A`, `(&A,&B)`, `With<&A,&B>`, `Without<...>`), assert the result set equals exactly
  the modelled entries satisfying it, with correct values and no duplicates.
- **Round-trip identities.** `deserialize(serialize(x)) == x`, `from_bits(to_bits(e)) == e`.
  Great as their own property over an arbitrary generated state.
- **Tagged-snapshot handles (MVCC / snapshot isolation).** If the system hands out
  snapshot views (read transactions, iterators over a version, savepoints), keep a
  pool of up to K OPEN handles, each tagged with the index into the linear committed
  `history` at which it was opened; interleave {open handle, mutate+commit, close
  handle} and after EVERY step assert each open handle still observes exactly
  `history[its tag]`. Deterministic single-threaded transaction-lifetime overlap is
  the concurrency that MVCC must get right — no threads needed, and it shrinks well
  (proved out on redb read transactions; the same shape fits any versioned store).
- **Miri as an oracle (for unsafe code).** Run the *same* harness under `cargo miri test`.
  Generated operation sequences drive the unsafe machinery over inputs a human would
  never hand-write, and Miri reports UB the model checks can't. See "Running under Miri".

## Design decisions that make or break it

- **Fix a small universe.** When the API is type-level (queries are types, not runtime
  values; components are heterogeneous), you cannot generate arbitrary types. Fix a
  handful of component/key types — include a **ZST** and a **Drop-tracked** one — and an
  enumerated set of read shapes. Generate operation *sequences* over that universe.
- **Keep a handle pool that includes invalid handles.** Retain despawned / stale-generation
  / reserved handles in the pool you pick targets from, so error paths
  (`NoSuchEntity`, generation-mismatch) are exercised, not just the happy path. Generation
  reuse (despawn frees an id, a later spawn reuses it with a new generation) falls out for
  free if the model keys on the exact handle.
- **Model lazy/deferred operations explicitly.** If the system defers work (hecs
  `reserve_entity` is invisible to `len`/iter/queries until a `flush`, which any
  spawn/insert/despawn triggers implicitly), keep those in a separate `reserved` list and
  `flush_model()` them into the model at exactly the points the real system flushes. Read
  the source to learn the precise visibility rules (`len` vs `contains` may disagree) —
  guessing produces false failures.
- **Read the real API from source, not memory.** Signatures bite: hecs `query::<Q>().iter()`
  yields `Q::Item` (not `(Entity, Item)`) — you include `Entity` *in* the query;
  `world.iter()` yields `EntityRef` (use `.entity()`); `Archetype::ids()` is `&[u32]`.

## hegel idioms

- `#[hegel::test] fn t(tc: hegel::TestCase) { ... }`; `#[hegel::test(test_cases = 400)]`
  sets the example count (default 100).
- Generators: `hegel::generators::{integers, floats, booleans, optional, sampled_from,
  one_of, vecs, just}`; `integers::<T>().min_value(a).max_value(b)`; draw with `tc.draw(g)`.
- Extract the loop body into `fn drive(tc: &hegel::TestCase, max_steps: u32)` and give two
  cfg-gated entry points so Miri runs a smaller configuration:
  ```rust
  #[cfg(not(miri))] #[hegel::test(test_cases = 400)]
  fn t(tc: hegel::TestCase) { drive(&tc, 250); }
  #[cfg(miri)] #[hegel::test(test_cases = 12)]
  fn t(tc: hegel::TestCase) { drive(&tc, 25); }
  ```

## Running under Miri

- `MIRIFLAGS="-Zmiri-permissive-provenance -Zmiri-disable-isolation" cargo +nightly miri test --lib`.
- `-Zmiri-disable-isolation` is **required with hegel**: hegel reads its test-case database
  from disk, and Miri's isolation otherwise aborts on `opendir` (that abort is
  infrastructure, not a finding).
- `-Zmiri-permissive-provenance` matches how crates like hecs run Miri in their own CI.
- Miri is ~50–100× slower, so keep the `cfg(miri)` case/step counts small (a dozen cases
  of a couple dozen steps still exercises the machinery thoroughly).

## What "passing" means

A mature crate will pass — that's expected, and it's still valuable: you now have a
reusable, high-coverage harness and a validated technique, and any future regression
(or the same technique aimed at a less-tested crate) has a ready oracle. Don't weaken a
check to make it pass; a divergence is either a real bug or a wrong model, and both are
worth the dig.

## See also

`metamorphic-and-differential-testing` — model-free oracles for the same kind of
system: metamorphic relations (commutation, do/undo identities, batch≡loop,
reset≡fresh) and N-way differential construction paths compared through an
observational fingerprint. Cheaper to write than a model, and it catches
cross-path/order/history bugs this harness structurally cannot; the two share
their component universe and Drop oracle.

`coverage-guided-property-testing` — once this harness plateaus, measure the
DEPENDENCY's per-file region coverage, read why the cold regions are cold, and
aim new property-shaped tests (with real oracles) at them; run each under Miri
immediately. The least-covered files are where the untested-code bugs live.
