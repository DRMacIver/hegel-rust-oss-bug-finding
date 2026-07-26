# Draft bug reports

Drafts for human review before filing. Each file is a ready-to-paste GitHub issue; edit freely. Before filing any of these, re-check the target repo's current issue tracker for duplicates and its contribution/AI policy.

## Batch 1 — filed (all 2026-07-24, as @DRMacIver)

- #1 heapless — [rust-embedded/heapless#676](https://github.com/rust-embedded/heapless/issues/676)
- #2 roaring — [RoaringBitmap/roaring-rs#359](https://github.com/RoaringBitmap/roaring-rs/issues/359)
- #3 jiff — [BurntSushi/jiff#613](https://github.com/BurntSushi/jiff/issues/613) (filed by @DRMacIver personally, per jiff's AI_POLICY.md — autonomous agents may not contribute)
- #4 gix-url — [GitoxideLabs/gitoxide#2827](https://github.com/GitoxideLabs/gitoxide/issues/2827) (gitoxide requires AI-agent identification, so the issue body carries a disclosure line)
- #5 pest — [pest-parser/pest#1183](https://github.com/pest-parser/pest/issues/1183)

## Batch 2 — filed (all 2026-07-25, as @DRMacIver)

Each verified to still reproduce against the latest pristine crates.io release, duplicate-checked, and no repo-level AI policy (all filed directly by the agent).

6. **[url](6-url-set-host-empty-unparseable.md)** — `set_host(Some(""))` returns `Ok` but yields a URL that fails to re-parse (`EmptyHost`). [servo/rust-url#1144](https://github.com/servo/rust-url/issues/1144)
7. **[rasn](7-rasn-oid-second-arc-corruption.md)** — an `ObjectIdentifier` with second arc > 39 encodes/decodes to a *different* OID (`0.40` → `1.0`). [librasn/rasn#565](https://github.com/librasn/rasn/issues/565)
8. **[nalgebra](8-nalgebra-svd-dynamic-precision.md)** — 2×2 SVD reconstructs ~7 digits worse through `DMatrix` than through `Matrix2`. [dimforge/nalgebra#1612](https://github.com/dimforge/nalgebra/issues/1612)
9. **[num-complex](9-num-complex-sqrt-underflow-zero.md)** — `sqrt(0 + 5e-324i)` returns exactly `0` for a nonzero input. [rust-num/num-complex#162](https://github.com/rust-num/num-complex/issues/162)
10. **[rumqtt](10-rumqtt-connect-write-nonempty-buffer.md)** — writing a v4 `Connect` into a non-empty buffer corrupts the frame. [bytebeamio/rumqtt#1063](https://github.com/bytebeamio/rumqtt/issues/1063)

Each ends with the requested hegel attribution line.

## Batch 3 — verified, awaiting review (not yet filed)

50 trophy candidates each built by a subagent (verify-on-latest → dup-check → policy-check → draft), then triaged by me (read every captured output; re-ran the risky release/DoS and duplicate checks). **9 dropped, 41 kept** — 40 directly fileable by the agent + 1 human-file (quinn-proto's CONTRIBUTING requires the issue be written by a human in their own words).

**Tier A — strong, oracle-independent, clean (32):**
`b3-01` bigdecimal `normalized()` scale overflow · `b3-02` sqlparser bracket-ident roundtrip · `b3-05` taffy stale layout after `remove` · `b3-06` vte C1 split dispatch · `b3-08` lz4_flex `Max8MB` self-reject · `b3-13` ipnet subnets past `end` · `b3-14` iri-string empty fragment · `b3-15` geo non-convex hull · `b3-19` ciborium `Ord` intransitive · `b3-20` yaml-rust2 decode infinite loop · `b3-21` quick-xml `trim_text_end` empty Text · `b3-24` hifitime `ZERO-MIN` vs `-MIN` · `b3-27` native_db `range(..=end)` exclusive · `b3-28` sled DoubleEnded re-yield · `b3-31` starlark AST Display roundtrip · `b3-33` rune `-i64::MIN` · `b3-36` openraft `Vote` PartialOrd panic · `b3-37` parry `distance` asymmetric · `b3-39` brotli BroCatli non-decodable · `b3-40` swash `advance_width` underflow · `b3-42` rkyv `access` release stack-overflow DoS · `b3-43` kurbo `eval` NaN endpoints · `b3-44` gltf slice-vs-reader disagree · `b3-45` palette hue out of `[0,360)` · `b3-47` similar empty-input panic · `b3-48` statrs `Gamma::pdf` NaN · `b3-49` bson decode release stack-overflow DoS · `b3-50` pulldown-cmark tasklist-in-heading · `b3-17` jj-lib conflict `\r` data loss · `b3-22` euclid angle range · `b3-29` diamond-types `load_from` panic · `b3-35` rumqtt `Publish` topic≥64KiB corruption

**Tier B — real but lower severity / niche / heavier repro (8):**
`b3-03` rpds `new_with_bits` panic & `b3-04` `new_with_degree(1)` abort (misconfig-adjacent) · `b3-07` vte `size_hint` (Iterator-contract, minor) · `b3-09` lz4_flex `try_finish` empty-frame · `b3-10` hickory `to_ascii` NUL-label roundtrip · `b3-18` rustybuzz glyph_id · `b3-38` datafusion empty-part roundtrip (low severity) · `b3-41` boa loop off-by-one (approximate-limit semantics)

**Human-file + deprioritised (1):**
`b3-11` quinn-proto — zero-length STREAM frames bypass the `TooManyChunks` memory cap and grow the unordered-read assembler's dedup set (resource-exhaustion). Two marks against it: (1) quinn-rs/quinn's CONTRIBUTING AI policy requires issues be described by a human in their own words, so @DRMacIver would have to file it personally (like jiff); (2) it only reproduces through internal (`pub(crate)`) APIs — the production path (a peer sending empty STREAM frames in unordered-read mode) is argued but not shown via the public API. Low priority; file only if a public-API repro can be built, otherwise leave it.

**Numerical / extreme-input group — probe, then hold:** `b3-15` geo, `b3-43` kurbo, `b3-37` parry, `b3-48` statrs trigger at extreme magnitudes / edge parameters and are more arguable than the corruption bugs. Plan (per DRMacIver): file these, then **hold further extreme-input numerical findings — especially additional ones in geo / kurbo / parry / statrs — until we see how maintainers respond** to this first set.

**Per-owner filing cap — max 2 issues per GitHub *owner* (not per repo), until a relationship exists.** Filing the 40 batch-3 reports brings these owners to the cap of 2 across all batches; do **not** file more to them without an established relationship: **apache** (sqlparser + datafusion), **servo** (rust-url✓ + euclid), **dimforge** (nalgebra✓ + parry), **bytebeamio** (rumqtt✓ + rumqtt/Publish), **orium** (rpds ×2), **alacritty** (vte ×2), **PSeitz** (lz4_flex ×2). (✓ = filed in an earlier batch.) Every other owner stays ≤ 1.

**Dropped from batch 3 (9):**
- chalk-ir #85 (`Subst::apply` panic) — repo is sunset (README deprecation banner, Chalk replaced by rustc's next-gen trait solver); also a ~100-line repro and arguable doc reading.
- loro #80 (import-batching representation difference) — dropped on review: the difference (`{}` vs `{"counter": 0.0}`) is plausibly semantically equivalent (a net-zero counter ≡ absent, which loro's own fuzzer assumes), so there's no clear user-visible problem.
- textwrap #21 — did not reproduce on latest (already fixed).
- geo #26 (convex-hull panic) — dup of open [georust/geo#531](https://github.com/georust/geo/issues/531).
- full_moon #76 (backtick panic) — dup of open [Kampfkarren/full-moon#359](https://github.com/Kampfkarren/full-moon/issues/359).
- bitvec #113 (`last_one` overflow) — dup of open [ferrilab/bitvec#166](https://github.com/ferrilab/bitvec/issues/166).
- num-rational #53 (`from_str` `i64::MIN` denom) — dup of open [rust-num/num-rational#6](https://github.com/rust-num/num-rational/issues/6) (same `reduce()` overflow).
- hifitime #58 (TAI↔UTC leap) — dup of open [nyx-space/hifitime#255](https://github.com/nyx-space/hifitime/issues/255).
- rmpv #64 (stack overflow) — debug-only; release enforces `MAX_DEPTH` correctly.

---

### Dropped during triage (not reported) — batches 1–2
- **ron** (`Value` f32 round-trip drift) — already open as [ron-rs/ron#613](https://github.com/ron-rs/ron/issues/613); also opt-in-fixable via `number_suffixes`.
- **geohash** (north pole encodes as south pole) — already has an open fixing PR (#63).
- **simd-json** (lone surrogate → NUL) — area already covered by closed #228 and advisory #457.
