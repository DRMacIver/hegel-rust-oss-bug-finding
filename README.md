# hegel-rust OSS bug-finding

A record of an investigation that used [hegel-rust](https://github.com/hegeldev/hegel-rust) property-based tests — written by subagents driving the [hegel-skill](https://github.com/hegeldev/hegel-skill) — to find real, upstream-reportable bugs in a wide range of open-source Rust crates, and to develop the hegel-skill from what the runs revealed.

## What's here

- **[TROPHIES.md](TROPHIES.md)** — every confirmed bug, each independently reproduced against pristine upstream (pinned commit), checked for a contract/spec violation, and dupe-checked against the crate's issue tracker before recording. Also lists confirmed duplicates (correctly triaged out), below-the-bar observations, and hegel-rust engine findings.
- **[SUMMARY.md](SUMMARY.md)** — top-level writeup: outcomes, the skill-development arc, and the convergence analysis.
- **[CANDIDATES.md](CANDIDATES.md)** — the original candidate list.
- **[results/](results/)** — the run log (`runs.md`, pinned commits per crate), full per-run evaluations (`batch2-eval.md`), per-candidate disposition (`coverage.md`), the extended-candidate selection heuristic (`extended-candidates.md`), and ready-to-file issue drafts (`draft-issues.md`).
- **[SKILL_NOTES.md](SKILL_NOTES.md)** — the skill-feedback accumulator.
- **[patches/](patches/)** + **[PATCHES.md](PATCHES.md)** — the actual property tests written for each crate, one `git apply`-able `.patch` per project. `PATCHES.md` records, for every patch, the upstream repository and the exact **base commit** (SHA, date, subject) the patch was written and verified against. Crates whose maintainers opted out of AI contributions are omitted.

## Method (triage bar)

Every suspected bug was recorded only after: an independent deterministic reproduction against pristine upstream; confirmation it violates a documented or self-evident contract (oracle-independent failures — panics, aborts, invariant violations, self-roundtrip failures — weighted above differential disagreements, especially in hardened crates); a check against the upstream issue tracker for duplicates; and exclusion of crates whose maintainers have opted out of AI contributions.

## Status

Nothing has been filed to any third-party repository — the findings are staged for review. Crates whose maintainers opted out of AI contributions were excluded and are noted as such. The actual per-crate property tests and `HEGEL_REPORT.md` files (under `targets/`, not committed here) remain local.
