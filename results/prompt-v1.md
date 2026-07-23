# Subagent prompt template v1

Used for batch 1 (2026-07-22): bigdecimal + data-encoding across haiku/sonnet/opus/fable.
Substitutions: {CRATE}, {DIR}, {CRATE_PATH} (path to the Cargo package to test, may equal {DIR} or a subdir).

---

You are adding property-based tests to the Rust crate `{CRATE}`, checked out at {DIR}, using the hegel property-based testing library (crate name: `hegeltest`). Your work will be reviewed both to find real bugs in the crate and to improve the hegel skill documentation, so follow the skill faithfully and report honestly.

MANDATORY FIRST STEP — load the skill exactly as a skill invocation would:
1. Read /home/dev/investigation/hegel-skill/skills/hegel/SKILL.md
2. Follow its workflow, which will tell you to read /home/dev/investigation/hegel-skill/skills/hegel/references/rust/reference.md and possibly other reference files (porting.md if the project already has proptest/quickcheck tests, extras.md if the code under test uses chrono/jiff/serde_json/rand).

Ground rules:
- Work ONLY inside {DIR}. Do not modify any files outside it (reading the skill files above is fine). Do not push to any git remote.
- The Cargo package to test is at {CRATE_PATH}. Add the dependency with `cargo add --dev hegeltest`.
- Do NOT fix bugs you find in the library, and do NOT weaken a test to make a real failure go away. When a test fails, follow the skill's "Run and Reflect" step: investigate whether it is a real bug, an unsound property, or an over-broad generator. If you conclude it is a real library bug, keep the test as written and document the failure instead of making it pass.

Deliverables:
1. Property-based tests added per the skill's workflow. Aim for breadth across the property catalogue where the code provides evidence: as a guideline 6-10 distinct properties, one property per test. If the crate exposes a data structure or stateful API suitable for it, include a stateful model test.
2. Run the tests (`cargo test`) and iterate until everything compiles and the suite runs. Tests failing due to suspected genuine library bugs stay in place, failing.
3. Write {DIR}/HEGEL_REPORT.md with exactly these sections:
   - **Properties tested** — each property, the test name, and the evidence (docs/signature/existing test/usage) it is grounded in.
   - **Test results** — pass/fail for every test you added, with cargo test output summarized.
   - **Suspected bugs** — for each: minimal counterexample as reported by hegel, why you believe it is a library bug rather than a test bug, and the exact code path in the library at fault if you found it.
   - **Unsound properties abandoned** — properties you tried and dropped, and why.
   - **Skill feedback** — the most important section. Anything in SKILL.md or the rust reference that was confusing, missing, wrong, or that you only got right after trial and error: compile errors you hit and how you resolved them, API surprises, generator gaps, unclear guidance. Be specific (quote the doc text or the compiler error). If everything just worked, say so.
4. `git add -A && git commit -m "Add hegel property-based tests"` inside {DIR} when done.

Your final response must be a concise structured summary with the same five sections as HEGEL_REPORT.md.
