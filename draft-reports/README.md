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

## Batch 3 — filed (2026-07-26, as @DRMacIver)

50 trophy candidates built by subagents, triaged, style-swept, and re-checked for duplicates + AI policy right before filing. **37 filed, 12 dropped, 1 held.** The pre-file re-check pulled 3 into the dropped list (ciborium, hickory, datafusion). Filed:

- bigdecimal — [akubera/bigdecimal-rs#165](https://github.com/akubera/bigdecimal-rs/issues/165)
- sqlparser — [apache/datafusion-sqlparser-rs#2409](https://github.com/apache/datafusion-sqlparser-rs/issues/2409)
- rpds `new_with_bits` — [orium/rpds#112](https://github.com/orium/rpds/issues/112)
- rpds `new_with_degree(1)` — [orium/rpds#113](https://github.com/orium/rpds/issues/113)
- taffy — [DioxusLabs/taffy#998](https://github.com/DioxusLabs/taffy/issues/998)
- vte C1 split — [alacritty/vte#156](https://github.com/alacritty/vte/issues/156)
- vte size_hint — [alacritty/vte#157](https://github.com/alacritty/vte/issues/157)
- lz4_flex Max8MB — [PSeitz/lz4_flex#232](https://github.com/PSeitz/lz4_flex/issues/232)
- lz4_flex try_finish — [PSeitz/lz4_flex#233](https://github.com/PSeitz/lz4_flex/issues/233)
- ipnet — [krisprice/ipnet#70](https://github.com/krisprice/ipnet/issues/70)
- iri-string — [lo48576/iri-string#62](https://github.com/lo48576/iri-string/issues/62)
- geo — [georust/geo#1566](https://github.com/georust/geo/issues/1566)
- jj-lib — [jj-vcs/jj#9868](https://github.com/jj-vcs/jj/issues/9868) (with AI-disclosure line)
- rustybuzz — [harfbuzz/rustybuzz#168](https://github.com/harfbuzz/rustybuzz/issues/168) — **repo unmaintained** (maintainer redirected to `harfrust`); should not have been filed. See postmortem below.
- yaml-rust2 — [Ethiraric/yaml-rust2#78](https://github.com/Ethiraric/yaml-rust2/issues/78)
- quick-xml — [tafia/quick-xml#984](https://github.com/tafia/quick-xml/issues/984)
- euclid — [servo/euclid#555](https://github.com/servo/euclid/issues/555)
- hifitime — [nyx-space/hifitime#494](https://github.com/nyx-space/hifitime/issues/494)
- native_db — [vincent-herlemont/native_db#469](https://github.com/vincent-herlemont/native_db/issues/469)
- sled — [spacejam/sled#1540](https://github.com/spacejam/sled/issues/1540)
- diamond-types — [josephg/diamond-types#50](https://github.com/josephg/diamond-types/issues/50)
- starlark — [facebook/starlark-rust#221](https://github.com/facebook/starlark-rust/issues/221)
- rune — [rune-rs/rune#1030](https://github.com/rune-rs/rune/issues/1030)
- rumqtt Publish — [bytebeamio/rumqtt#1064](https://github.com/bytebeamio/rumqtt/issues/1064)
- openraft — [databendlabs/openraft#1872](https://github.com/databendlabs/openraft/issues/1872)
- parry — [dimforge/parry#431](https://github.com/dimforge/parry/issues/431)
- brotli — [dropbox/rust-brotli#258](https://github.com/dropbox/rust-brotli/issues/258)
- swash — [dfrg/swash#133](https://github.com/dfrg/swash/issues/133)
- boa — [boa-dev/boa#5461](https://github.com/boa-dev/boa/issues/5461)
- rkyv — [rkyv/rkyv#684](https://github.com/rkyv/rkyv/issues/684)
- kurbo — [linebender/kurbo#598](https://github.com/linebender/kurbo/issues/598)
- gltf — [gltf-rs/gltf#475](https://github.com/gltf-rs/gltf/issues/475)
- palette — [Ogeon/palette#473](https://github.com/Ogeon/palette/issues/473)
- similar — [mitsuhiko/similar#99](https://github.com/mitsuhiko/similar/issues/99)
- statrs — [statrs-dev/statrs#422](https://github.com/statrs-dev/statrs/issues/422)
- bson — [mongodb/bson-rust#680](https://github.com/mongodb/bson-rust/issues/680)
- pulldown-cmark — [pulldown-cmark/pulldown-cmark#1115](https://github.com/pulldown-cmark/pulldown-cmark/issues/1115)

Five carry a "still reproduces after #N" reference (quick-xml #755, native_db #214, bson #385, rkyv #301, geo #912); jj-lib carries an AI-agent disclosure line (its PR template expects LLM disclosure). The categorisation below records which were strong (Tier A) vs lower-value (Tier B).

**Postmortem — rustybuzz (#168) was filed to an unmaintained repo.** The maintainer replied that rustybuzz is no longer maintained (use `harfrust`). It wasn't archived and had no README deprecation notice, but it was ~13 months stale with a named successor crate — signals our checks (repro + duplicate + policy) never looked at. Fix: a maintenance/staleness check (`archived` + `pushed_at` + successor-crate glance) is now a filing-gate step in `prioritising-bugs-to-report`. A sweep of the other 33 filed repos found **none** stale (oldest ~7 months, none archived), so rustybuzz was the only miss. The bug does reproduce in the active successor `harfrust` (as a clean invariant violation — it dropped rustybuzz's `debug_assert`, so no panic), re-filed there as [harfbuzz/harfrust#409](https://github.com/harfbuzz/harfrust/issues/409) (draft `b3-51`). rustybuzz#168 to be closed.

**Tier A — strong, oracle-independent, clean (32):**
`b3-01` bigdecimal `normalized()` scale overflow · `b3-02` sqlparser bracket-ident roundtrip · `b3-05` taffy stale layout after `remove` · `b3-06` vte C1 split dispatch · `b3-08` lz4_flex `Max8MB` self-reject · `b3-13` ipnet subnets past `end` · `b3-14` iri-string empty fragment · `b3-15` geo non-convex hull · `b3-20` yaml-rust2 decode infinite loop · `b3-21` quick-xml `trim_text_end` empty Text · `b3-24` hifitime `ZERO-MIN` vs `-MIN` · `b3-27` native_db `range(..=end)` exclusive · `b3-28` sled DoubleEnded re-yield · `b3-31` starlark AST Display roundtrip · `b3-33` rune `-i64::MIN` · `b3-36` openraft `Vote` PartialOrd panic · `b3-37` parry `distance` asymmetric · `b3-39` brotli BroCatli non-decodable · `b3-40` swash `advance_width` underflow · `b3-42` rkyv `access` release stack-overflow DoS · `b3-43` kurbo `eval` NaN endpoints · `b3-44` gltf slice-vs-reader disagree · `b3-45` palette hue out of `[0,360)` · `b3-47` similar empty-input panic · `b3-48` statrs `Gamma::pdf` NaN · `b3-49` bson decode release stack-overflow DoS · `b3-50` pulldown-cmark tasklist-in-heading · `b3-17` jj-lib conflict `\r` data loss · `b3-22` euclid angle range · `b3-29` diamond-types `load_from` panic · `b3-35` rumqtt `Publish` topic≥64KiB corruption

**Tier B — real but lower severity / niche / heavier repro (8):**
`b3-03` rpds `new_with_bits` panic & `b3-04` `new_with_degree(1)` abort (misconfig-adjacent) · `b3-07` vte `size_hint` (Iterator-contract, minor) · `b3-09` lz4_flex `try_finish` empty-frame · `b3-18` rustybuzz glyph_id · `b3-41` boa loop off-by-one (approximate-limit semantics)

**Human-file + deprioritised (1):**
`b3-11` quinn-proto — zero-length STREAM frames bypass the `TooManyChunks` memory cap and grow the unordered-read assembler's dedup set (resource-exhaustion). Two marks against it: (1) quinn-rs/quinn's CONTRIBUTING AI policy requires issues be described by a human in their own words, so @DRMacIver would have to file it personally (like jiff); (2) it only reproduces through internal (`pub(crate)`) APIs — the production path (a peer sending empty STREAM frames in unordered-read mode) is argued but not shown via the public API. Low priority; file only if a public-API repro can be built, otherwise leave it.

**Numerical / extreme-input group — filed as a probe:** geo, kurbo, parry, statrs trigger at extreme magnitudes / edge parameters and are more arguable than the corruption bugs. They're filed; **hold further extreme-input numerical findings — especially additional ones in geo / kurbo / parry / statrs — until maintainers respond** to this first set.

**Per-owner filing cap — max 2 issues per GitHub *owner* (not per repo), until a relationship exists.** These owners are now at the cap of 2 across all batches; do **not** file more to them without an established relationship: **servo** (rust-url✓ + euclid), **dimforge** (nalgebra✓ + parry), **bytebeamio** (rumqtt✓ + rumqtt/Publish), **orium** (rpds ×2), **alacritty** (vte ×2), **PSeitz** (lz4_flex ×2). (✓ = filed in an earlier batch.) apache is at 1 (sqlparser; datafusion was dropped). Every other owner stays ≤ 1.

**Dropped from batch 3 (12):**
- ciborium #43 (`CanonicalValue` `Ord` intransitive) — the exact bug plus a fix and regression tests were already submitted as closed [enarx/ciborium#176](https://github.com/enarx/ciborium/pull/176) by an independent contributor; known, not novel.
- hickory-proto #17 (`to_ascii`/`from_ascii` NUL-label roundtrip) — Tier-B, low value; same round-trip class already handled in closed [hickory-dns#2353](https://github.com/hickory-dns/hickory-dns/issues/2353).
- datafusion #94 (`TableReference` empty-part roundtrip) — instance of the open umbrella [apache/datafusion#6853](https://github.com/apache/datafusion/issues/6853); low value.
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
