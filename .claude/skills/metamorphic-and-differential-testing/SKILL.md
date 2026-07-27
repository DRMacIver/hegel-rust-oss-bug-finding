---
name: metamorphic-and-differential-testing
description: >
  Property-test a stateful Rust API without a reference model, using hegel:
  metamorphic relations (two executions whose results must relate in a known
  way — commuting operations, do/undo identities, batch-equals-loop,
  reset-equals-fresh) and differential construction paths (build the same
  logical state through N different API routes and demand observational
  equivalence). Use when a full reference model is too costly or as a
  complement to one: these oracles are model-free, cheap to write, and catch
  cross-path inconsistencies a single-model harness structurally cannot.
---

# Metamorphic & differential testing (with hegel)

Model-based testing (see `stateful-model-based-testing`) needs a reference
model that mirrors every operation. Two model-free alternatives cover the
same ground from a different angle:

- **Metamorphic**: run *two executions from the same state* whose outcomes
  must relate by the API's own semantics — `X;Y ≡ Y;X` on disjoint targets,
  `insert;remove ≡ id`, `batch(N) ≡ N × single`, `clear() ≡ fresh()`.
  The oracle is the relation; no model tracks values.
- **Differential**: build the *same logical state* through N different API
  paths (builder vs tuples vs deferred buffer vs incremental inserts) and
  demand the results are observationally identical. Each path exercises
  different internal machinery; the paths cross-check each other.

Both were developed against hecs (an ECS); the worked examples below are
`hecs-pbt/tests/metamorphic.rs` and `hecs-pbt/tests/differential_paths.rs`
with shared infrastructure in `hecs-pbt/tests/common/mod.rs`.

## The core tool: an observational fingerprint

Everything reduces to one reusable function — a **canonical, plain-data
snapshot of what a caller can observe**:

```rust
type Fingerprint = BTreeMap<Entity, Obs>;   // exact handle -> exact components/values
fn fingerprint(world: &World) -> Fingerprint { /* iterate, read every component */ }
```

Design rules that matter:

- **Key by the exact identity the API exposes** (hecs: `Entity` = id AND
  generation, and `Entity: Ord` makes `BTreeMap` canonical). If two worlds
  assign different handles, that IS an observable difference — don't paper
  over it with position-based comparison.
- **Plain data, `==`-comparable, editable.** "Adjusted identity" relations
  compute the expected result by *editing the before-fingerprint* (e.g.
  `expected[e].b = None`) instead of re-deriving state. The assert's diff then
  reads as the exact semantic delta.
- Build defensively: assert no duplicate keys while collecting, and that the
  snapshot size equals the API's own `len()` — the fingerprint doubles as a
  consistency check on iteration itself.

## Twin states without `Clone`

Most stateful systems aren't `Clone`, but metamorphic relations need two
executions "from the same state". If construction is **deterministic**
(hecs hands out identical `Entity` values for identical spawn/despawn
histories), replay one drawn history into N fresh instances and *assert*
equality of returned handles at every step. Two payoffs: you get twins, and
determinism itself becomes a continuously-checked property (the twin builder
fails loudly the day it stops holding).

Seed the history with despawns/removals too, so twins enter the experiment
with non-trivial freelists and stale generations, not just a happy path.

## A catalog of relations that transfer to most stateful APIs

1. **Commutation on disjoint targets.** Draw two ops targeting *different*
   entities/keys; apply `X;Y` to one twin, `Y;X` to the other; fingerprints
   and each op's own result must match. Caveats found the hard way:
   - Exclude *allocating* ops (spawn): allocation order is observable through
     handles, so they legitimately don't commute.
   - Two deallocating ops commute observationally but leave different
     freelist orders — fine, unless you then allocate as a "probe". Don't.
2. **Do/undo adjusted identity.** `insert_one::<T>(v)` then
   `remove_one::<T>()` equals "original minus T on that entity" (a plain
   identity only when T was absent — the *adjusted* form covers both cases
   uniformly). Check the removed value equals the inserted one. Same pattern
   for exchange round-trips: `exchange<A,B>` then `exchange<B,A>(old)`
   restores everything *except* a pre-existing B (overwritten, then taken).
   Overwrite semantics are the classic trap: naive "round-trip = identity"
   claims are wrong whenever the forward step can clobber existing state.
3. **Batch ≡ loop.** `spawn_batch(N specs)` vs N individual `spawn`s: same
   handles, same fingerprint. Include *partial consumption* if the batch is
   lazy — hecs's `SpawnBatchIter::drop` drains the iterator, so taking only
   k of N must still spawn all N. Read the source to learn the laziness
   contract first.
4. **Reset ≡ fresh.** After `clear()`, replay one drawn op sequence into the
   cleared instance and a brand-new one in lockstep; fingerprints (and
   allocated handles!) must match after every step. This pinned down hecs's
   documented-but-subtle contract that `clear()` resets entity metadata so
   handles repeat — the strongest form is "indistinguishable under any
   subsequent program", approximated by a random suffix.
5. **Order-independence of independent writes to one target.** Insert two
   *different* components in both orders: same result, even though the two
   orders route through different intermediate representations (archetypes).

## Differential construction paths

Enumerate every API route that can produce "an entity with components S":

hecs: (1) dynamic builder, (2) concrete tuple bundles — dispatch over all 2^k
subsets to hit every static arity, (3) reserve-then-insert, (4) command
buffer replayed later, (5) empty-spawn then one `insert_one` per component
(migrating through every intermediate archetype).

Then:

- **Same drawn specs through every path; assert all fingerprints equal** —
  including handles (deterministic allocation makes this exact).
- **Ground-truth against the spec** too: N paths agreeing on a *wrong* answer
  must be caught, and the spec is already plain data — assert path 1's
  fingerprint contains exactly the drawn spec per new entity.
- **Mutation-amplification phase**: after construction, apply one drawn
  mutation sequence identically to all N instances and re-compare after each
  op. Internally the instances differ (path 5 built a chain of intermediate
  archetypes; path 2 only the final ones); this phase asserts construction
  history is unobservable — a whole bug class (state depending on how you
  got there) that a single-path model harness can never see.
- Keep the structural invariants (partition checks) and a Drop-tracked
  component running across **all** instances: a path that leaks or
  double-drops while building shows up as a live-count imbalance even when
  values compare equal.

## Validate the oracle, and what failures look like

Before trusting a suite that passes immediately, **plant a temporary bug and
watch it fail** (then revert; never commit the plant). Two experiments worth
repeating on any new harness, because they show the two failure modes:

- *Planted SUT-path bug* (incremental path silently dropped C when A was
  present): hegel shrank to zero pre-existing entities and a single spec
  `{a: 0, c: true}` — exactly the two fields the bug needed — with the assert
  message naming the offending path. Draw-in-the-loop harnesses shrink this
  well; keep per-comparison messages specific (`"construction path {i} ..."`).
- *Wrong relation/adjustment* (claiming exchange round-trip is a plain
  identity): shrank to one entity `{a: 0, b: 0}`, and the fingerprint diff
  *was* the missed semantics (actual lost B, expected kept it). A metamorphic
  failure is always ambiguous between "SUT bug" and "my relation is wrong" —
  resolve it by reading the SUT's source/docs for the documented semantics,
  never by weakening the assert until it passes.

## How this relates to model-based testing

- Cheaper to write: no per-op model bookkeeping — the fingerprint plus a
  relation replaces the model.
- Different bug surface: cross-path inconsistency, order-dependence,
  construction-history observability, reset incompleteness — things a
  single-model harness can't express.
- Weaker at value tracking over long histories — a model harness checks every
  intermediate state against independently-computed truth; metamorphic
  relations only compare executions with each other (plus spec ground truth
  where available).
- Use both. They share the component universe, Drop oracle, and structural
  invariants; in this project all of that lives in one `tests/common/mod.rs`.
