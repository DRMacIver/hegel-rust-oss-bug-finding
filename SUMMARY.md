# Hegel trophy-case + skill-development: final summary

_Investigation run 2026-07-22 → 2026-07-23. Every crate evaluated with hegel-rust PBTs written by subagents invoking the hegel-skill; each suspected bug independently reproduced against pristine upstream and dupe-checked before recording._

## Outcomes

### 1. Trophy case — 130 confirmed bugs across ~59 crates (incl. four extended candidate batches)
- **128 solidly upstream-reportable** + 2 recorded at explicitly low severity (chalk #86 near-unreachable DeBruijn overflow; lodepng OOM below-bar observation).
- **Bugs 98-130 come from an EXTENDED candidate set** chosen (batches 13-16) by applying this run's own empirical bug-predictors to crates NOT in CANDIDATES.md, after a verification pass (results/extended-candidates.md). The heuristic transferred cleanly across batches:
  - **Batch 13**: rkyv (recursion-DoS in its "safe" validation API), glam (try_normalize contract, via the SIMD-vs-scalar differential oracle), i_overlay x3, qoi x2 (vs the C reference), kurbo x4; clean: unicode-normalization, borsh, rustfft.
  - **Batch 14** (new-domain convergence probe): gltf #109, fancy-regex #110, palette #111-112, bitvec #113, simd-json #114-115, instant-distance #124; clean: rangemap, symphonia.
  - **Batch 15** (2nd probe): geohash #116-117, evalexpr #118-122, similar #123; clean: h3o, xxhash-rust, geographiclib-rs, pathfinding (all four are reference-differential-fuzzed crates); minijinja **excluded on maintainer-consent grounds** (ships `HUMAN_VS_MACHINE.md` AI-gate).
  - **Batch 16** (3rd probe): num-bigint #125, reed-solomon-erasure #126, earcutr #127, aho-corasick #128, statrs #129-130; clean: indexmap, data-encoding; ulid = duplicate of open #101. Two below-bar observation groups (aho-corasick under-specified empty-pattern leftmost semantics; statrs's 14 extreme-parameter numeric breakdowns).
- **8 confirmed duplicates correctly triaged out** (humantime #67, unicode-segmentation #174, strsim #79, num-rational #146, swash #130, boa #4311, instant-distance #49 deadlock, ulid #101 overflow-string) — the triage bar catching pre-existing reports before filing.
- **All found by the fable model.** haiku/sonnet/opus contributed only on the batch-1/2 calibration crates and found zero bugs the fable runs didn't; the tier gap was decisive and is documented in `results/batch2-eval.md`.
- **Severity highlights**: silent data corruption (roaring run-containers, kiddo leaf-split, sled iterator, loro counter-convergence, rumqtt Connect-into-buffer), whole-DB bricking (fjall key-limit), credential corruption (gix-url), data loss (loro shallow snapshot), DoS/hang (yaml-rust2 decoder, spade flood-fill, multiple interpreter stack overflows), and a large family of untrusted-input decoder panics and roundtrip/contract violations.
- Full list with per-bug repro, code path, and dupe-check in `TROPHIES.md`; ready-to-paste issue drafts for the first six in `results/draft-issues.md`.
- **Nothing filed to any third-party repo** — awaiting explicit user approval.

### 2. hegel-skill improvements — branch `improve-skill-from-model-eval`, 16 commits, v0.6.0
Started from a skill that was stale against hegeltest 0.28.2 and thin on field guidance. Landed, all evidence-driven and verified against hegel-rust source:
- **Verified API-staleness fixes** (batch 1): Variables→Pool rewrite, date/time generator renames, `.unique(bool)`, `domains().max_length()`, removed "server" language.
- **Setup/environment**: crate-vs-package naming, MSRV `--ignore-rust-version`, edition-2015 `extern crate`, `no_std`/alloc/core variants, workspace `-p`, self-dependency `-p name@version`, test-binary inflation.
- **Methodology additions**: generate configurations not just payloads; the live-test + 10× exploratory verification loop; boundary conjunctions; probe recursion depth directly; validation/rejection pattern; a consolidated **Oracle Sourcing** section (dependency-tree, fast-path-vs-generic, sibling-API-with-caveat, exhaustive-small-universe, spec-transcription, float-oracle audit); the stop-and-report anti-fallback rule; process-abort/hang handling; deterministic pinning recipe.
- **Porting**: enumerate-and-preserve coverage; cfg-gated suites; macro-family carve-out; strengthen-weak-ports; post-port dependency hygiene.
- **hegel engine bug found + fixed**: draft PR **hegeldev/hegel-rust#379** (reorder_spans stale-index panic) — TDD regression test, `just check` green.

### 3. Convergence — measured, not assumed
Skill methodology stabilized at batch 7 and held flat through batch 12 (six batches, only refinements). Then batch 13, by testing **new domains** (zero-copy validation, SIMD linear algebra, FFT, image codecs, Bézier geometry), revived the lesson stream: vacuous-guarded-property detection, three float-tolerance refinements, depth-probing-vs-safety-APIs, and simulate-oracle-in-generator. **Meta-finding: skill improvement is driven by domain diversity, not iteration count** — re-running the same crate classes converges fast, but each genuinely new domain surfaces fresh methodology.

Batches 14, 15, and 16 were deliberate **convergence probes** to measure the lesson-rate decay. The count of committed skill lessons per new-domain batch did **not** decay to zero — it went **batch 13 → 5; batch 14 → 1 substantial + refinements; batch 15 → ~3 small/process; batch 16 → ~7** (one substantial: the oracle-independent-vs-differential triage rule; six refinements: stochastic-mutation-survivor/failure-DB, per-feature suite runs, iterative-numerics-stop-criterion tolerance, hang-zone pin margins, non-Copy composite gotcha, Fisher-Yates subset recipe). Batch 16 **rose** because it hit new crate-*types* — heavily-hardened reference crates (aho-corasick, num-bigint, indexmap) and feature/backend complexity (reed-solomon) — which surface new *classes* of lesson (triage, verification discipline, feature-matrix coverage), not just new oracle-construction tricks. Meanwhile **bug-finding never decayed**: new domains yielded bugs in 6/8 (b14), 6/8 (b15), and 6/8 (b16) crates; the selection heuristic is domain-independent. Crates that come back clean are consistently those already continuously fuzzed **differentially against a reference implementation** (h3o vs C H3, xxhash vs C xxHash, rustfft, borsh, unicode-normalization).

**Convergence verdict:** the literal `/goal` criterion — "no further skill improvements found in evaluation" — is **not converging to zero** under new-domain testing. The *substantial-methodology* stream (lessons that change how the skill fundamentally works) did plateau around batch 14, but a persistent, non-vanishing tail of genuine minor refinements and per-domain gotchas keeps surfacing — batch 16 produced more of them than 14 or 15, not fewer. The rate is bounded by the diversity of untested crate-*types*, not by iteration count, and the space of crate-types (and their idiosyncratic gotchas) is large. Reaching a batch with literally zero committed skill edits appears to require either exhausting that space (many more batches, no guaranteed endpoint) or redefining "improvement" to mean "substantial-methodology" (met at batch 14). This is the open decision surfaced to the user. Interpreters/mini-languages (evalexpr joining boa/rune/starlark/full_moon) remain the single most reliable bug-bearing domain, and the skill already fully covers them.

## Coverage
`results/coverage.md` has a disposition for every CANDIDATES.md entry (all evaluated across batches 1-12, or excluded with rationale: maintainer consent — bincode; environmental — cosmic-text/sanakirja/bloomfilter; or CANDIDATES.md's own deprioritization) plus the extended crates from batches 13-16. ~59 crates evaluated in total; 2 excluded on maintainer-consent grounds (bincode, minijinja).

## Awaiting user
1. **Approval to file** trophies upstream (drafts staged; per-target attribution done, incl. #84 → microlp not good_lp, and the residual-of-open-issue cases → comment vs new issue). Nothing has been filed to any third-party repo.
2. **PR title + short description** for the hegel-skill branch (16 commits pushed; PR not opened per the draft-PR convention).
3. **Definition of "done" for the skill-convergence `/goal`** (see §3): the literal "zero further improvements" is not converging under new-domain testing; choose between (a) continue new-domain batches with no guaranteed endpoint, or (b) accept substantial-methodology convergence (reached at batch 14) as the stopping point.
4. Optionally, filing the hegel-rust feature requests accumulated in `TROPHIES.md` (global case-count override — requested 6+ times; `char` generator; `draw_labelled`; a built-in branch-hit/case counter for liveness checks — requested 3+ times; jiff-extras domain gaps).

## Key files
- `TROPHIES.md` — 130 bugs + observations + duplicates + hegel-rust findings, each triaged.
- `results/coverage.md` — per-candidate disposition.
- `results/runs.md` — run log, pinned commits, per-crate outcome.
- `results/batch2-eval.md` — full per-run evaluation (batches 2–16) incl. model-tier analysis + convergence data points.
- `results/extended-candidates.md` — the verification pass + heuristic behind batches 13-16.
- `results/draft-issues.md` — ready-to-file issue texts.
- `SKILL_NOTES.md` — skill-feedback accumulator (all landed).
- `targets/<crate>-fable/` — each run's tests + HEGEL_REPORT.md (git remotes stripped; nothing pushed).
