# Hegel trophy case + skill development

## Goal
1. Build a trophy case of real, upstream-reportable bugs found via hegel-rust property-based tests on the crates in CANDIDATES.md.
2. Use the subagent runs (varying model: haiku/sonnet/opus/fable) to find weaknesses in the hegel skill (hegel-skill repo) and improve it.

## Workflow per target crate
1. Clone upstream into `upstream-cache/<crate>` (pinned commit recorded in `results/runs.md`).
2. Copy to `targets/<crate>-<model>` per model, strip git remote.
3. Spawn one subagent per model with the prompt template in `results/prompt-vN.md`. Agent reads the skill from `hegel-skill/skills/hegel/`, adds tests, writes `HEGEL_REPORT.md`, commits.
4. Evaluate: read each HEGEL_REPORT.md + `git diff` in each target; score in `results/<crate>-batchN.md` (property quality, generator discipline, skill adherence, bugs found, friction points).
5. Triage suspected bugs myself (reproduce, check docs/upstream issues for dupes) → record confirmed ones in `TROPHIES.md`.
6. Fold skill feedback into concrete edits to `hegel-skill` (branch + PR-ready commits).

## Layout
- `CANDIDATES.md` — target shortlist (input report)
- `hegel-rust/`, `hegel-skill/` — reference clones (hegel-skill is where skill edits land)
- `upstream-cache/<crate>` — pristine clones
- `targets/<crate>-<model>` — per-run working copies
- `results/` — prompt templates, run log, per-batch evaluations
- `TROPHIES.md` — triaged/confirmed bugs
- `SKILL_NOTES.md` — accumulated skill-improvement observations, mapped to skill edits

## Status
- 2026-07-22: Batch 1 launched — bigdecimal + data-encoding × {haiku, sonnet, opus, fable}, prompt v1. Environment smoke-tested (hegeltest 0.28.2 works, Rust 1.95).
- 2026-07-22: Batch 1 complete + evaluated (results/batch1-eval.md). 3 confirmed bigdecimal bugs in TROPHIES.md (2 novel + 1 residual of open #44). Skill improvements landed on hegel-skill branch `improve-skill-from-model-eval` (pushed; PR needs human title/description). Model tiers: fable > sonnet ≈ opus >> haiku (haiku needs mechanical verification).
- 2026-07-22 (later): Batch 2 complete + evaluated (results/batch2-eval.md). 3 more confirmed bugs → 6 total in TROPHIES.md (5 novel): #4 toml_datetime offset boundary, #5 sqlparser bracket-ident Display (roundtrip contract), #6 roaring run-container silent corruption (most severe). All 6 found by fable. Skill A/B verdict: batch-1 edits measurably worked (zero API friction, haiku fallback fixed, "boundary conjunctions" directly credited for #5). Second skill commit pushed (cff3d76): recursive-composite E0283 pattern, live-test + 10x-run verification loop, porting nuances.
- Awaiting user: (a) approval to file trophies #1, #3, #4, #5, #6 upstream + comment on bigdecimal #44 for #2; (b) PR title/description for hegel-skill branch `improve-skill-from-model-eval` (2 commits, pushed).
- Next batch candidates: Stage-2 stateful targets (redb, fjall, taffy, yrs) — fable-only or fable+opus; consider prompt tweak for weak tiers (mechanical bounded-generator justification rule).
