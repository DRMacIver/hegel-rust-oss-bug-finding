---
name: prioritising-bugs-to-report
description: >
  Decide whether a bug found in someone else's crate is actually worth reporting,
  and how strong a finding it is, before writing it up. Use when triaging a
  candidate bug, selecting which findings to file upstream, or sanity-checking a
  severity claim. Guards against reporting documented preconditions, doc-only
  off-by-ones, and consequences you asserted but never tested.
---

# Prioritising bugs to report

Before writing a report, decide whether the finding is real and how strong it is. A weak or overstated report costs a maintainer's goodwill and makes the next one easier to ignore. The bar is not "did something go wrong" — it is "would the maintainer agree this is a bug, and is it as bad as I'm about to say."

## Test the consequence before you claim it

The single most common way to overstate a bug is to assert a consequence you reasoned about but never ran. If you're going to say "corrupts the database", "bricks", "leaks", "loses data", "crashes the server" — reproduce that exact outcome end to end first.

The fjall case: an oversized key panicked mid-write and left the keyspace returning `Err(Poisoned)`, so the draft said it "bricks the whole database". Actually reopening the database succeeded fine — the poison was in-memory and cleared on restart. The reopen took one small test to check, and it removed the entire headline. Run that test *before* the claim goes in, not after someone challenges it.

Corollary: don't count the downstream, expected consequence of a failure as a second, separate severity. A panic while holding a lock poisoning that lock is what Rust does — it is the same bug, not a cascade.

## Signals a finding is strong (worth reporting)

- It violates a contract the crate states about **itself**: its docs, or better, its own property tests / fuzz targets / `debug_assert`s / `sanity_check`. "Your own fuzz target asserts this roundtrip and here's an input that breaks it" is close to unarguable.
- It is **oracle-independent** — you don't need a second implementation to agree it's wrong: memory unsafety (OOB, UB, `get_unchecked` violations), silent data corruption, a self-roundtrip failure (`parse(print(x)) != x`, `decode(encode(x)) != x`), or a violated invariant the crate itself checks elsewhere.
- It is reachable from the **safe, public, documented API** with input the docs treat as **valid**.
- The crate is maintained and prominent, and hasn't opted out of AI-found reports.

jiff `-PT0.5S` parsing as `+0.5s` is strong on every axis: valid ISO 8601 input, silent sign loss, breaks jiff's own roundtrip fuzz target, public API, no precondition to hide behind.

## Signals a finding is weak (downgrade, reframe narrowly, or drop)

- **Documented precondition.** A panic on input the crate documents as a caller error (a `# Panics` section, "must be", "the caller must ensure") is usually intended, like indexing out of bounds. Not a bug on its own. The exception: if the crate documents that same input as *valid* and it still panics, that inconsistency is the real, and much narrower, finding.
- **Doc-only ambiguity / off-by-one.** "Up to N" that's really N−1, or behaviour that's self-consistent but mis-described, is a documentation fix. Report it as that, small, not as a correctness catastrophe. (fjall's limit is 65535, documented as 65536 — genuine, but a doc/validation nit.)
- **A `Result`-returning API panicking instead of returning `Err`** on invalid input is a real but minor API-quality issue — file it as exactly that, without inflating the blast radius.
- **Differential disagreement with no authority.** "Crate A and crate B disagree" is only a bug if a spec, RFC, or the crate's own stated guarantee says which is right. Absent that, it's an observation, and weak — especially against a hardened, widely-used crate where the behaviour is likely deliberate.
- **An accepted, opt-in-mitigated limitation.** Some imperfections are a known, tolerated class the ecosystem treats as fine, often with an opt-in switch to fix them. serde_json's default float parse is an ULP off unless you enable `float_roundtrip`, and that's considered acceptable, not a bug. Before reporting one, look for the tells: a feature flag or config that fixes it (ron's `number_suffixes` makes lossy `Value` f32 round-trips exact), a documented "not guaranteed" stance, or maintainers saying so on the tracker. If the fix is already a switch the user is expected to flip, it's a docs/ergonomics point at best.
- **Unmaintained / deprecated / dormant** crate, or one whose policy declines AI-found reports.

## Search the tracker for the behaviour, not just your exact repro

Duplicate-checking means searching for the *behaviour*, in open and closed issues **and** PRs — not just pasting your repro's error string. What you find reclassifies the finding:

- **An open issue describing the same behaviour** (even filed by someone else, even hedged) — it's a duplicate. Don't file. (ron's f32 `Value` round-trip was already open as #613, with the same repro and analysis.)
- **An open PR fixing it** — also a duplicate; the maintainers already know. (geohash's north-pole-encodes-as-south-pole already had an open "fix maximum coordinate wrapping" PR.)
- **A closed/fixed precedent for the same class** — two signals at once: (a) *positive* — the maintainers treat this class as a real bug and fix it, so a genuinely-new instance is welcome (nalgebra fixed an SVD-consistency bug in #1089, so an SVD-accuracy report won't be waved off); and (b) *caution* — your instance may already be fixed, so **re-verify on the latest release**, especially when the tracker shows recent work in that exact code area.
- **A wontfix / "not planned as intended" precedent** — a strong downgrade. If they closed the same class saying it's by design, yours will land the same way unless you can show it's materially different.

## Can you even file it? — the maintainer's AI policy is an effort signal

Before you invest in a finding, check whether the repo's contribution rules let *you* file it. Look **in the repo only** — `AI_POLICY.md`, `AGENTS.md`, `CONTRIBUTING.md`, a contribution section in the README, issue/PR templates. If there's no policy in the repo, assume filing is fine; don't go hunting org-level `.github` repos or external pages.

Three outcomes, in rough order of effort:

- **No policy → file directly.** Lowest effort.
- **AI allowed with disclosure/identification → file directly, plus one disclosure line.** e.g. gitoxide's CONTRIBUTING requires an AI agent acting through an account to identify itself in the issue body. Cheap, but don't forget the line.
- **Autonomous agents prohibited / issue must be written by a human in their own words → you can't file it; a person must.** e.g. jiff's (and other BurntSushi repos') `AI_POLICY.md` bans autonomous-agent contributions and will hide AI-written comments. The finding may be excellent, but shipping it needs a human to write and post it.

When prioritising a shortlist, this is a real cost axis: prefer the file-directly findings first, and batch the human-must-file ones for when that person is available. A strong bug in a jiff-style repo isn't lower *quality* — it's higher *effort to land*, so it sits lower when you're picking what to do next. Note which bucket each candidate is in when you present the shortlist.

## Before promoting a finding to "report this"

Ask, in order:

1. Have I reproduced the *actual* worst consequence I'm claiming, on a pristine upstream checkout, from the public API?
2. Is the triggering input genuinely valid per the docs — or is there a `# Panics` / precondition clause that makes this intended?
3. Would a fair maintainer call this a bug, or would they say "working as documented / that's a caller error / that's just the docs"?
4. Am I stating one bug, or have I bundled its inevitable side effects into a scarier-sounding cascade?
5. Is the severity word I'm using ("corrupts", "bricks", "unsafe") one I have direct evidence for?
6. Can I file it directly, or does the repo's AI policy mean a human has to? (Affects effort and who does it, not whether it's a bug.)

If a finding only survives as "a Result API panics instead of erroring on the documented-max input, and the docs are off by one" — that's honest, but it's not top-tier. Rank it accordingly, or leave it out of a curated shortlist.
