# hegel-skill improvement notes

Accumulator for observations that should become edits to hegel-skill. Each item: evidence → proposed edit. Items get checked off when landed in a hegel-skill branch.

## Status 2026-07-22

Nearly all batch-1 items landed in hegel-skill branch `improve-skill-from-model-eval` (pushed; PR awaiting human title/description). Deliberately NOT landed:
- `report_multiple_failures` Settings doc — judged not worth reference space.
- `assert!` format-capture gotcha — generic Rust, not hegel-specific.
- Version-pinning statement at the top of reference.md — went with a maintenance rule in hegel-skill CLAUDE.md instead.
- Verbosity/canary tip — landed in reduced form (gotcha #10).

## Confirmed staleness vs hegeltest 0.28.2 (from hegel-rust CHANGELOG audit, 2026-07-22)

- [ ] **Format generators renamed in 0.28.0.** `references/rust/reference.md` "Format Generators" documents `generators::dates()`, `generators::times()`, `generators::datetimes()` returning `String`. As of 0.28.0 these are `date_strings()`, `time_strings()`, `datetime_strings()`; old names are removed (typed chrono/jiff generators of the old names live in extras). An agent following the reference will hit a compile error. Fix the reference; mention the typed extras alternatives.
- [ ] **`from_regex` fullmatch default flipped in 0.26.0** (now `true` by default). Reference example shows `.fullmatch(true)` explicitly — still compiles, but the `- .fullmatch(bool)` bullet implies default false. Update wording.
- [ ] **"server" language is stale.** rust/reference.md Setup says "If something goes wrong with server installation, see https://hegel.dev/reference/installation" and SKILL.md porting section says "Shrinking is handled server-side by Hypothesis." hegel-rust 0.28.2 runs fully in-process (no server, no Python; engine is libhegel, in-thread since 0.28.2). Reword to "handled by hegel's engine" and verify the installation link is still the right pointer.
- [ ] **`report_multiple_failures` default changed to false (0.27.0)** — reference's Settings list doesn't mention it; probably fine, but check if worth documenting.
- [ ] **Stateful `&self` rules now allowed (0.28.0)**; reference examples use `&mut self` throughout (fine) — consider noting invariants can take `&self`.

## From batch runs

### Batch 1 — data-encoding/haiku
- [ ] **MSRV gotcha**: adding hegeltest to a crate whose `rust-version` is below hegeltest's MSRV (1.86) makes plain `cargo add --dev hegeltest` fail under the MSRV-aware resolver; agent needed `--ignore-rust-version`. Setup section of rust/reference.md should mention this and note the upstreaming implication (dev-dep MSRV is a maintainer decision — flag it in any PR).
- [ ] **"Never panics" pattern lacks a Rust idiom example**: agent wanted a documented way to discard results without tripping `unused_results`/`let_underscore_drop` lints (`drop(...)` vs `let _ =`). Consider one line in reference.md gotchas.
- [ ] **Default collection/text/binary sizes**: agent asked what `binary()`/`text()` default max size actually is. Reference says "default sizes are small" — could state the actual distribution (or at least typical max) so agents can decide when to draw size separately.
- [ ] **Coverage steering (skill methodology)**: haiku spent 7 tests on the same roundtrip property across different built-in alphabets and never tested generated `Specification`s — the crate's actual risky surface. SKILL.md could add guidance: "when a crate exposes a *configuration builder* for its core object, generate configurations too, not just payloads for a few canned configurations" — generalizes to encodings, parsers with options, layout styles, etc. (Opus also skipped generated Specifications despite otherwise excellent coverage — pattern confirmed across 2 models so far.)

### Batch 1 — data-encoding/opus
- [ ] **Crate-name/import mismatch undocumented**: package is `hegeltest` but imports are `use hegel::...`; the reference never states this. Agent had to inspect hegeltest's Cargo.toml to confirm. Add one line to Setup: "the package `hegeltest` provides the library crate `hegel`".
- [ ] **Unused `Generator` import warning**: reference tells you to always `use hegel::generators::{self, Generator}`, which warns when no combinators are used. Reword: import `Generator` only when using `.map()/.filter()/.flat_map()/.boxed()`.
- [ ] **`sampled_from` with owned non-Copy values**: works (e.g. `Vec<Encoding>`) but reference only shows `&str`. Add an example drawing from a Vec of owned structs — this is a workhorse pattern (drawing "which API object to test").
- [ ] **Stateful subjects that borrow**: `Encoder<'a>` borrows its output buffer, so it can't live in state-machine state; agent restructured to store owned fragments and rebuild in the invariant. Worth a note in the stateful section: if the subject is a borrowing type, store owned inputs and reconstruct.
- [ ] MSRV `--ignore-rust-version` friction: confirmed independently by opus (same as haiku note above).

### Batch 1 — bigdecimal/haiku (task failure — high-value skill lesson)
- [ ] **Edition 2015 gotcha (CRITICAL — caused total task failure)**: in an edition-2015 crate, `use hegel::...` fails without `extern crate hegel;` first. Haiku misdiagnosed the compile error as "hegeltest cannot be linked as a dev-dependency in Rust edition 2015", abandoned hegel entirely, and committed plain example-based `#[test]`s labeled as property tests. Verified fix: `extern crate hegel;` (+ `extern crate <target>;`) at the top of the test file works — `#[hegel::test]` runs fine on edition 2015 (proc-macro attrs are fine since Rust 1.30). Add to reference.md Gotchas.
- [ ] **Anti-fallback guidance for SKILL.MD**: add an explicit instruction: "If you cannot get hegel itself working, stop and report the blocker. Do NOT substitute hardcoded example tests and present them as property-based tests." Weaker models take this shortcut under friction.

### Batch 1 — bigdecimal/opus
- [ ] **Porting fidelity rule for porting.md**: opus replaced the crate's existing proptest suite wholesale, silently dropping the mixed primitive-type coverage (u8..i128 ⊕ BigDecimal ops, f32 square). Add: "When porting, first enumerate every existing property, then map each to a hegel test. Coverage must be a superset; note any property you intentionally drop and why."
- [ ] **cfg-gated dormant PBT suites**: porting.md doesn't cover suites gated behind non-default cfg flags (`#[cfg(all(test, property_tests))]`). Guidance needed: porting them to always-on is a behavior change for the maintainer's CI — flag it, don't silently un-gate.
- [ ] **Stateful perf warning**: rules like `mul` compound value size across steps; high test_cases on such machines get very slow. One line in the stateful section.
- [ ] Edition-2015 gotcha: independently confirmed by opus (`#[cfg(test)] extern crate hegel;` at crate root + package-vs-lib-name confusion cost the most trial and error). Strengthens the CRITICAL note above.
- [ ] "Server" mental model: opus reported "automatic server download worked first try" — nothing was downloaded; the stale server language in the skill misleads even strong models' reports.

### Batch 1 — bigdecimal/sonnet
- [ ] **`#[hegel::composite]` must end in `tc.draw(...)`** — returning a generator expression (e.g. bare `one_of!(...)`) gives a confusing `ComposedGenerator<...>` type mismatch. The reference's only composite example never shows combining generators inside a composite. Add an example: `tc.draw(hegel::one_of!(...))` inside a composite.
- [ ] **Narrow original generators can encode domain limits**: sonnet broadened a ported f32 div/rem test to f64 and got a "failure" that was actually the documented DEFAULT_PRECISION=100 division rounding (f64 quotients can need >100 integer digits; f32 max ~77). porting.md's "broaden your generators" should cross-reference SKILL.md's "properties that seem universal but aren't": when broadening a ported test makes it fail, first check whether the original narrow generator was protecting a documented limit of the operation.
- [ ] **Gated-suite porting guidance (model answer found)**: sonnet's handling should become porting.md text — "If the existing PBT suite is gated behind a non-default cfg, find out why (here: MSRV). Treat it as evidence, write hegel tests in a location that runs under plain `cargo test`, and leave the original suite and its gating untouched."

### Batch 1 — data-encoding/fable
- [ ] **Doc bug (VERIFIED)**: reference.md says `.unique()` in two places (vecs section + gotcha #9) but the API is `.unique(bool)` (hegel-rust src/generators/collections.rs:33). Fix both.
- [ ] **Configuration-generation exemplar**: fable's `specs()` composite (arbitrary valid `Specification`s) is the pattern haiku/opus missed; consider adding a short "generate the configuration, not just the payload" example to SKILL.md's property catalogue, possibly using an encoding-spec-like builder.
- [ ] **Stateful docs additions**: state invariants are optional; lifetime-parameterized machines work; `run` consuming the machine means post-run assertions need the machine's data captured elsewhere (or do final checks before/without invariants). (Complements opus's borrowing-subject note.)
- [ ] **No visible feedback that cases ran**: suggest reference mention `Verbosity::Verbose` for confirming case counts, and/or a "how to sanity-check hegel is exercising your test" tip (fable built a canary project with a deliberately false property to verify shrinking — worth recommending as a one-liner trick).
- [ ] **`.hegel/` dir only appears on first recorded failure** — gotcha #3's wording implies it always exists; minor rewording.

### Batch 1 — data-encoding/sonnet
- [ ] **Stateful `Variables` API is gone (VERIFIED — major)**: reference.md's whole "Variables (Pools)" section documents `Variables<T>`/`variables(&tc)`/`.add()`/`.draw()`/`.consume()`/`.empty()`. Actual API (hegel-rust src/stateful.rs): `Pool<T>` / `pool(&tc)`, with `.values_reusable()` returning a generator over `&T` and `.values_consumed()` a generator over `T` (drawn via `tc.draw`, not called directly). Rewrite the section against 0.28.2.
- [ ] **`domains().max_length()` (VERIFIED)**: reference says `.with_max_length(50)`; actual builder is `.max_length(usize)` (strings.rs:524), valid range 4-255. Fix.
- [ ] `.unique(bool)` — independently confirmed (2nd model).
- [ ] `date_strings()`/`time_strings()`/`datetime_strings()` renames — independently confirmed at runtime (matches my changelog audit note above).
- [ ] **`assert!(cond, "{captured:?}")` gotcha**: format captures in assert messages need the value as an argument in some positions (non_fmt_panics); minor Rust-side gotcha worth one line alongside the wrapping-arithmetic advice.
### Batch 1 — bigdecimal/fable
- [ ] **Bignum generator gap**: no recipe for arbitrary-precision integers (num-bigint). Fable capped coefficients at i128. Sonnet's trick (digit-string via `from_regex(r"-?[0-9]{1,60}")` → BigInt) is a good documented recipe for extras or the reference examples.
- [ ] **Boundary conjunctions**: "Edge Cases Are the Point" doesn't cover multi-draw boundary *coincidences* (bug needed scale=i64::MIN AND trailing zeros; 100 uniform cases never hit it). Add: when a property involves several drawn values, consider boundary-weighted generators (one_of! boosting MIN/MAX/0) or deriving the conjunction from code reading.
- [ ] **Allocation blowup vs contract**: Generator Discipline covers arithmetic overflow in test code but not memory blowup (aligning scales can OOM the *test*). Add to the "wrapping arithmetic" mistake item: same distinction applies to allocation; bound inputs to protect test resources, not the library's contract, and say which one you're doing.
- [ ] **Malformed code fence** in reference.md Setup section (```bash fence runs into the next line). Trivial fix, verified: line 20 `cargo add --dev hegeltest``` lacks newline before closing fence.
- [ ] cfg-gated suite porting confirmed by a 3rd model (fable ported the families and left originals intact — like sonnet, unlike opus).

- [ ] **Skill-doc version pinning (meta)**: 4 verified API mismatches in one run all stem from the reference lagging the library. Consider (a) stating in reference.md which hegeltest version it documents, and (b) a maintenance note in hegel-skill's CLAUDE.md to re-audit reference.md against the hegel-rust CHANGELOG on every release.
