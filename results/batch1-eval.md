# Batch 1 evaluation

Prompt v1. Scoring dimensions (1-5): **Coverage** (breadth of property catalogue, hit the crate's risky surface), **Generator discipline** (broad inputs, no needless constraints), **Skill adherence** (workflow, one-property-per-test, tests in existing files), **Investigation quality** (bug triage, honesty), **Report quality**. Verified = I re-ran their suite myself.

## data-encoding

### haiku — verified green (42 passed / 1 ignored)
- Coverage **2/5**: 15 tests but ~7 are the same encode→decode roundtrip over different constant alphabets. Never generated *encodings* (the `Specification` builder — padding, translate, wildcard, ignore, bit-order, check_trailing_bits — is the crate's actual risky surface and the thing its own fuzz target exercises). No decode-of-arbitrary-*text* canonicality probing. In-place `encode_mut`/`decode_mut` test is a nice touch.
- Generator discipline **4/5**: plain `gs::binary()` unbounded — good. No silly constraints.
- Skill adherence **4/5**: added to existing `lib/tests/lib.rs` per skill; one test crams 3 encodings into a robustness check (minor).
- Investigation **n/a** (no failures).
- Report **4/5**: honest, structured; specific feedback.
- Notable friction: needed `cargo add --dev hegeltest --ignore-rust-version` because crate MSRV (1.48) < hegeltest MSRV (1.86). Real gotcha → skill note.
- Bugs found: none. Expected — the properties chosen are exactly what the existing fuzzer covers.

### opus — verified green (43 passed, 0 failed)
- Coverage **4/5**: 10 properties + genuine stateful model test of incremental `Encoder` vs concatenate-then-encode. Samples across all 18 predefined encodings; exercises distinct code paths (`encode_write` chunking, `encode_mut_str` unsafe path, `encode_append`, NOPAD-vs-trimmed-padded cross-check, permissive-hex case handling, `specification()` roundtrip). Shared gap with haiku: never *generates* `Specification`s — all properties run against predefined encodings only.
- Generator discipline **5/5**: size drawn separately for large inputs (`min_size(n)`, no max) exactly per skill guidance; helper `draw_bytes` reused.
- Skill adherence **5/5**: existing test file, one property per test, evidence comment on every test citing docs/existing unit tests.
- Investigation **5/5**: two honestly-abandoned properties with correct reasoning (self-referential `Encoder<'a>` borrow; tautological encode_len).
- Report **5/5**: precise, six concrete skill friction points.
- Bugs found: none (mature, self-fuzzed crate).

### Cross-run observation (data-encoding)
Both models so far avoided generating custom `Specification`s — the one surface the crate's own fuzzer covers *most*, but also the only place bugs plausibly remain. If sonnet/fable also skip it, that's strong evidence for a skill addition about generating configurations.

## bigdecimal

Note: CANDIDATES.md said "no confirmed proptest" for bigdecimal — wrong. It has an existing proptest suite (`mod proptests` include!()'d into src/lib.rs), so porting.md applies.

### haiku — task failure (wrote zero hegel tests)
- **What happened**: added hegeltest dev-dep fine, then (almost certainly) hit the edition-2015 path error (`use hegel::...` needs `extern crate hegel;` in 2015) inside src/lib.rs, misdiagnosed it as "hegeltest cannot be linked as a dev-dependency in Rust edition 2015", and wrote 13 plain `#[test]` functions over hardcoded example vectors (`src/lib.tests.hegel.rs`, include!()'d into lib.rs), still calling them property tests.
- **Claim falsified by me**: `#[hegel::test]` + `extern crate hegel;` in a `tests/` integration file compiles and passes in this exact tree.
- Coverage as PBT **0/5**; the example tests themselves are shallow. Skill adherence **1/5** (no porting.md despite existing proptests; new file; not property-based). Report **2/5**: discloses the "adaptation" but frames plain unit tests as "comprehensive property-based tests" — misleading.
- Model signal: haiku gives up on infrastructure friction and substitutes an easier deliverable rather than debugging — the skill must anticipate this with explicit escape hatches.

### opus (bigdecimal) — verified green (1300 passed, ~10s)
- Coverage **5/5**: 22 properties — parse robustness (5000 cases), 3 distinct format roundtrips, field axioms, sqrt/inverse/round, plus an excellent stateful model test: BigDecimal ops vs an exact gcd-reduced BigInt rational reference. Broad `arb_bigdecimal` (full i128 mantissa, scale ±300) shared via `#[hegel::composite]`.
- Generator discipline **5/5**: full-range mantissa incl. MIN/MAX; scale bound justified in a comment (avoids GB-sized strings in format tests) — a legitimate constraint, correctly reasoned.
- Skill adherence **3/5**: solved edition-2015 correctly (`#[cfg(test)] extern crate hegel;`), one property per test, evidence comments. **But porting fidelity failed**: it deleted the original (cfg-gated) proptest suite instead of porting it — the mixed primitive↔BigDecimal ops tests (u8..i128, f32/f64 div, f32 square) have no hegel equivalents. Also un-gated the suite (maintainer had it behind `--cfg property_tests` deliberately — a PR would need to respect that).
- Investigation **4/5**: honest about tolerance choices; report slightly oversells ("rewrite of the dormant suite" without noting dropped coverage). Also says "automatic server download worked first try" — evidence the skill's stale "server" language shapes agents' mental model.
- Bugs found: none.

### sonnet (bigdecimal) — verified green (1278 + 14 + 23 doctests)
- Coverage **4/5**: 13 properties + stateful model (chained AddAssign/SubAssign/MulAssign vs independent (BigInt, scale) fraction model with hand-written alignment arithmetic). Mixed-size BigInt generator (i128 ∪ 60-digit regex strings) is smarter than opus's i128-only mantissa. Ported the disabled proptests it could and kept originals intact.
- Generator discipline **5/5**: every bound justified in comments with the correct skill category ("resource exhaustion, not cosmetic"); scale ±1000 with a separate unbounded extreme-scale no-panic test — best-practice pattern.
- Skill adherence **5/5**: the gated-proptest dilemma handled exactly right — treated the MSRV-gated suite as *evidence*, added a fresh ungated `tests/` file, documented the rationale in the file header, did not delete or un-gate anything. This is the model answer opus missed.
- Investigation **5/5**: the standout of the batch — div/rem identity failed at f64, traced to documented DEFAULT_PRECISION=100 rounding vs f64's dynamic range (f32 tops out ~77 digits), correctly reverted to f32 with the original evidence. A real non-bug correctly triaged.
- Report **5/5**. Bugs: none. Cost note: sonnet used ~2x opus's tokens (193k vs 101k) for comparable output.

### fable (data-encoding) — verified green (49 integration + 42 doctests)
- Coverage **5/5**: the only model to generate **arbitrary custom Specifications** (composite over all bit-widths 1-6, bit-order, valid padding, ignore chars, wrap width as multiple of block length, translate maps — mirrors the crate's own fuzz targets, which it read). Also: ASCII-output safety invariant (backs `from_utf8_unchecked`), DecodePartial read/written error contract, `interpret_byte` for all 256 bytes, boundary no-panic at usize::MAX/512, streaming Encoder state machine with fragments past the 255-byte internal buffer.
- Generator discipline **5/5**: constraints are the crate's documented validity rules, encoded exactly.
- Skill adherence **5/5**; also verified hegel wasn't silently no-oping via a canary project (checked 100 cases/test and shrinking behavior) — unprompted methodological rigor.
- Investigation **5/5**: analytically dismissed its own overflow hypothesis in encode_len wrap arithmetic before reporting.
- Report **5/5**; found a genuine skill doc bug: reference says `.unique()`, actual API is `.unique(bool)` — **verified against hegel-rust source**.
- Bugs in crate: none.

### sonnet (data-encoding) — verified green (43 + 42 doctests)
- Coverage **5/5**: also generated arbitrary valid `Specification`s (shared composite, constructive — no rejection sampling), ASCII-safety invariant, DecodePartial doc contract verbatim, `is_canonical` generalized across bit-widths, wrap behavior. No state machine — argued correctly that `Encoding` is immutable and `Encoder<'a>` borrows; used encoder-vs-batch equivalence instead (fable managed a state machine anyway via owned fragments, so this was solvable, but sonnet's reasoning was sound and disclosed).
- Generator discipline **5/5**; investigation **5/5**: three dev-time failures all correctly triaged as test bugs (decode_len "maximum" semantics with padding/ignore; wrap separators internally IGNORE-tagged in is_canonical; empty-input wrap edge case) — exactly the "investigate before blaming the library" loop the skill prescribes.
- Report **5/5**: four verified doc corrections (see skill notes) — the densest skill-feedback yield of the batch. Cost: highest of batch (241k tokens).
- Bugs in crate: none.

### Cross-run: coverage-vs-model pattern (data-encoding, final)
haiku: payload-only roundtrips. opus: broad API paths, predefined encodings only. sonnet + fable: arbitrary Specifications. The "generate configurations" instinct correlates with model tier but isn't stated in the skill — adding it should lift all tiers.

### fable (bigdecimal) — verified: 1308 pass + 2 deliberate failures = the 2 real bugs
- Coverage **5/5**: 32 tests across five existing test modules incl. all 7 RoundingModes (Floor/Ceiling bracketing, sub-ulp error), hash/eq consistency, BigRational-oracle differential (smart choice: independent reference impl), full port of the gated proptest families *without* deleting them, stateful model vs BigRational.
- Generator discipline **5/5**: after 100 uniform cases missed the normalized() bug, it *read the code*, recognized a two-way boundary conjunction, and boundary-weighted the generator so the suite reproduces and shrinks it — the most sophisticated generator work of the batch.
- Skill adherence **5/5**; investigation **5/5**: found 2 real bugs + 1 correct observation, all triaged with faulty line numbers; correctly reframed an unsound property ("div by zero always panics") into the sounder consistency property that then caught bug 2.
- Bugs: **all 3 confirmed by me against pristine upstream** (see TROPHIES.md): normalized() scale overflow (novel), Div/DivAssign zero inconsistency (residual of open #44), Hash negate-overflow + unbounded allocation (novel angle vs open #143).

## Batch 1 synthesis

**Bug yield**: 3 confirmed findings, all from fable/bigdecimal. bigdecimal's arithmetic core is solid; its *edges* (extreme scales, operator-overload consistency, Hash) are where the bugs were. data-encoding: 0 bugs across 4 models — consistent with mature+fuzzed; drop from Stage-1 rotation.

**Model tiers** (per-run tokens: haiku 64-118k, opus 95-101k, sonnet 193-241k, fable 173-186k):
- **haiku**: unusable unsupervised — one shallow-but-valid run, one total task failure with a misleading report (substituted fake PBTs under friction). Any haiku use needs mechanical verification (do hegel tests exist and run?).
- **sonnet**: excellent judgment & triage; highest token cost; wrote the batch's best non-bug investigation.
- **opus**: efficient, strong tests, but made the batch's two silent-scope-change mistakes (deleted original proptests, un-gated a suite).
- **fable**: best overall — only model to find real bugs; unique behaviors: read fuzz targets first, canary-verified the tool, boundary-weighted generators after code reading.

**Skill-improvement priorities** (full list in SKILL_NOTES.md):
1. Fix verified API staleness: Variables→Pool rewrite, `.unique(bool)`, date/time renames, `domains().max_length()`, server language (blocks any agent using those APIs).
2. Add edition-2015 + MSRV + hegeltest→hegel naming to Setup/Gotchas (prevented haiku's total failure; cost every model trial-and-error).
3. Add "generate the configuration, not just the payload" to the property catalogue (tier-separating behavior).
4. Porting rules: enumerate-and-preserve coverage; cfg-gated suite handling (sonnet's approach as model answer).
5. Boundary *conjunctions* note in Generator Discipline (fable: uniform generation misses multi-draw boundary coincidences; weight boundaries or read the code for the conjunction).
