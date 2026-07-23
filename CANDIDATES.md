# Candidate Rust Libraries for Property-Based Testing with hegel-rust

## TL;DR
- I recommend ~100 actively maintained, production-grade Rust libraries as hegel-rust targets, split into "clean-property" (roundtrip/algebraic/invariant) and "boundary-pushing" (databases, runtimes, layout, VCS, CRDTs) buckets, with a Top 20 shortlist led by taffy, redb, fjall, sqlparser, rustybuzz, jiff, uom, bigdecimal, roaring, gix, wasmi, salsa, yrs, and quinn-proto.
- The strongest targets combine real usage with thin existing PBT: taffy, bigdecimal, uom, roaring (pure-Rust), sqlparser, rustybuzz, fjall, crop/ropey, petgraph, and yrs have limited/unconfirmed property testing; deprioritize over-hardened targets like rasn (AFL++-fuzzed), zune-jpeg (fuzz corpus), and malachite (built-in generative testing) except for differential or stateful work.
- Avoid duplicating the antithesis blog's examples (fraction, rust_decimal, heck, im-rs) and avoid mega-hardened crates (serde, regex, url, base64, aho-corasick, rustls, std); the sweet spot is "serious, real users, but not continuously OSS-Fuzzed."

## Key Findings
- The Hegel blog (David MacIver, March 24, 2026) explicitly used `fraction` (`from_str("0/0")` panic), `rust_decimal` (scientific-notation zero bug, issue #784), `heck` (ß title-case idempotence, issue #70), and `im-rs` (`OrdMap::get_prev`, issue #215) as examples — off-limits for duplication. MacIver acknowledged many were unmaintained ("it's just way easier to find nice illustrative bugs in such libraries"), which is exactly the "punching down" the user wants to avoid.
- MacIver's bug taxonomy is a useful test-design lens: (1) "you forgot about zero," (2) "this data type is cursed and you fell afoul of the curse" (Unicode, floats, time), (3) "you made an error in a complicated structural invariant" — the model-based-testing category Antithesis is "most excited about."
- The most fertile clean-property targets are roundtrip laws (serialize/parse), algebraic laws (arithmetic/units/decimals), and data-structure invariants (ordered maps, bitmaps, ropes) — all easily expressed against a reference model such as std's `BTreeMap`.
- The most fertile boundary-pushing targets are stateful systems where a random operation sequence can be diffed against a simple in-memory model: embedded databases, LSM engines, layout engines, terminal parsers, WASM interpreters, VCS object stores, and CRDTs.

## Details

Fifteen crates were metrics-verified against crates.io/lib.rs/GitHub in a dedicated pass; all are confirmed pure Rust and actively maintained in 2025–2026. Metrics elsewhere are best-effort; unverified figures are flagged.

### Category A — Clean, easily-stated properties

#### Arbitrary-precision arithmetic, decimals, units, rationals
1. **bigdecimal** — github.com/akubera/bigdecimal-rs — Arbitrary-precision decimal built on num-bigint. Properties: algebraic laws (associativity/commutativity/distributivity), parse/format roundtrip, `a/b*b ≈ a` within scale, comparison total-order consistency. ~87M downloads all-time, v0.4.10 (2025), pure Rust, no confirmed proptest. **Strong (a).**
2. **malachite** — github.com/mhogrefe/malachite — Arbitrary-precision integers/rationals (algorithms derived from GMP/FLINT/MPFR, reimplemented in pure Rust; LGPL-3.0). Properties: full ring/field axioms, conversions, string roundtrip. ~6.8M downloads, MSRV 1.90, but **already has extensive built-in generative testing** — lower priority except for differential testing vs num-bigint/rug.
3. **uom** — github.com/iliekturtles/uom — Type-safe units of measurement / dimensional analysis. Properties: unit-conversion roundtrip (m→ft→m), dimensional-consistency invariants, `x+y` across units equals canonical. ~11M downloads, v0.38.0 (2025), pure Rust, no_std. Thin PBT. **Strong (a).**
4. **num-rational** (rust-num) — Rational numbers. Properties: lowest-terms reduction invariant, field axioms, float conversion. Widely used, pure Rust.
5. **fixed** — Fixed-point numbers. Properties: saturation/wrap laws, string parse roundtrip, float conversion.
6. **rust_decimal** — CALIBRATION-ADJACENT: blog found a bug here (issue #784); use only for *new* properties, not scientific-notation.

#### Serialization / format roundtrips
7. **cbor2** — Full RFC 8949 CBOR (canonical/deterministic encoding). Properties: encode→decode roundtrip, canonical-form idempotence, tag preservation. Newer/less-hardened than the archived serde_cbor. **Good (a).**
8. **ciborium** — CBOR for serde (maintained). Properties: value roundtrip, self-describing tag handling.
9. **toml_edit** — github.com/toml-rs/toml — Format-preserving TOML parser. Properties: parse→serialize preserves formatting/comments, edit-then-reparse idempotence, semantic equality after roundtrip. ~355M downloads, actively maintained. **Strong (a).**
10. **yaml-rust2** — github.com/Ethiraric/yaml-rust2 — Pure-Rust YAML 1.2. Properties: load→emit→load roundtrip, YAML test-suite conformance.
11. **rmp / rmp-serde** — MessagePack. Properties: encode/decode roundtrip, type preservation.
12. **quick-xml** — XML reader/writer. Properties: parse→write roundtrip, escaping/unescaping inverse.
13. **prost / quick-protobuf** — Protobuf. Properties: message encode/decode roundtrip, unknown-field preservation.
14. **csv** (BurntSushi) — Properties: write→read roundtrip with quoting/escaping edge cases (CSV quoting is "cursed").
15. **bincode** — Properties: roundtrip, length-prefix invariants.
16. **postcard** — no_std serde format. Properties: roundtrip, varint encode/decode inverse.
17. **serde_json** — WATCH: heavily fuzzed; deprioritize.

#### Encoding / crypto-adjacent (not primitives)
18. **rasn** — github.com/librasn/rasn — ASN.1 BER/CER/DER/PER codec framework. Properties: encode→decode roundtrip, DER canonical-form. ~331 stars, v0.28.1 (Nov 2025), but **already fuzzed with AFL++** — deprioritize or do BER-vs-DER differential.
19. **asn1** (pyca) — DER parser/writer. Properties: parse→write roundtrip minimality.
20. **data-encoding** — base32/base64/hex with many variants. Properties: encode→decode roundtrip across all alphabets, padding invariants; strong "forgot about zero/empty" surface. **Good (a).**
21. **bs58** — Base58. Properties: encode/decode roundtrip, checksum validation.
22. **percent-encoding** — URL percent-encoding. Properties: encode/decode inverse, idempotence.
23. **der** (RustCrypto), **base64** — WATCH: heavily fuzzed; deprioritize.

#### Parsing / identifiers / networking address types
24. **sqlparser** — github.com/apache/datafusion-sqlparser-rs — SQL lexer/parser (foundation for DataFusion, Polars, GreptimeDB, GlueSQL, PRQL). Properties: parse→unparse→parse AST-equality, no-panic on arbitrary input. Pure Rust, maintained (v0.59+), no confirmed proptest. **Strong (a).**
25. **semver** — github.com/dtolnay/semver — Properties: parse→display roundtrip, ordering total-order axioms, comparator matching. WATCH: well-tested but precedence rules are subtle.
26. **iri-string** — RFC 3987 IRI/URI. Properties: parse→serialize roundtrip, normalization idempotence, reference-resolution laws. Less hardened than `url`. **Good (a).**
27. **ipnet / cidr** — IP network/CIDR. Properties: parse→display roundtrip, subnet containment, network/broadcast computation.
28. **iprange** — IP range sets. Properties: union/intersection/difference set laws.
29. **glob / globset** — Properties: match consistency, literals match themselves, escape handling.
30. **humantime** — Human duration parsing. Properties: parse→format roundtrip, "forgot about zero."
31. **byte-unit** — Byte-size parsing. Properties: parse→format roundtrip, unit conversion.
32. **jiff** — github.com/BurntSushi/jiff — Modern datetime (Temporal-inspired). Properties: civil↔zoned roundtrip, DST-aware arithmetic inverse (add then subtract a span), ISO-8601 parse/print roundtrip. 7,631,393 all-time downloads (crates.io, v0.2.16), pure Rust, very active — but still pre-1.0: per BurntSushi's April 2026 note, "It is now April 2026 ... I don't currently have a timeline for a Jiff 1.0 release." BurntSushi favors quickcheck so some PBT likely exists, but the API surface is huge. **Strong (a/b border).**
33. **hifitime** — High-precision aerospace time. Properties: epoch conversions roundtrip, leap-second handling. Niche but serious.
34. **chrono** — WATCH: extremely widely used; deprioritize (tz edge cases persist though).

#### Text / string algorithms
35. **unicode-segmentation** — Grapheme/word/sentence boundaries. Properties: concatenation of segments equals input, UAX#29 conformance.
36. **strsim** — String similarity. Properties: metric axioms (identity/symmetry/triangle inequality), edit-distance bounds.
37. **textwrap** — Properties: unwrapping wrapped text preserves words, width invariants.
38. **bstr** (BurntSushi) — Byte-string ops. Properties: UTF-8 lossy roundtrip, split/join inverse.
39. **nucleo-matcher / fuzzy-matcher** — Fuzzy matching (nucleo powers Helix). Properties: matched indices form a valid subsequence, score monotonicity.
40. **heck** — CALIBRATION: blog found the ß bug (issue #70); off-limits for that property.

#### Data structures & collections
41. **roaring** — github.com/RoaringBitmap/roaring-rs — Pure-Rust Roaring bitmaps. Properties: set ops (union/intersection/difference/xor) vs `HashSet<u32>` model, serialize→deserialize roundtrip, cardinality correctness. ~700K downloads/month, last release Dec 2024 (verify activity). Pure Rust (not the C-FFI `croaring`). **Strong (a).**
42. **rpds** — Persistent/immutable data structures. Properties: structural-sharing correctness vs std-collection model. Less hardened than `im-rs`. **Good (a).**
43. **crop / ropey / jumprope** — Text ropes (crop/jumprope actively maintained). Properties: rope ops vs String model (insert/delete/slice), char↔byte↔line index conversions, split/append inverse. **Strong (a).**
44. **petgraph** — Graph structures/algorithms. Properties: shortest-path vs reference, topological-sort validity, SCC correctness. Invariant-rich. **Strong (a).**
45. **rstar** — R*-tree spatial index. Properties: nearest-neighbor vs brute-force, bounding-box query completeness.
46. **kiddo / kdtree** — kd-tree. Properties: k-NN vs brute-force, range-query completeness.
47. **fst** (BurntSushi) — Finite-state transducer maps/sets. Properties: build→query vs BTreeMap model, ordered iteration.
48. **indexmap** — Insertion-ordered map/set. Properties: vs std HashMap semantics + order preservation, swap_remove invariants. WATCH: widely used but order invariants testable.
49. **slotmap / generational-arena** — Properties: key stability, ABA-safety after remove/insert.
50. **priority-queue / keyed_priority_queue** — Properties: heap ordering invariant, priority-change correctness vs sorted model.
51. **bloomfilter / probabilistic-collections / cuckoofilter** — Bloom/cuckoo filters, count-min sketch. Properties: no false negatives, false-positive rate bounds, serialize roundtrip. **Good (a).**
52. **im-rs** — CALIBRATION: blog found OrdMap bug (issue #215); use only new properties.

#### Compression roundtrips
53. **lz4_flex** — Pure-Rust LZ4. Properties: compress→decompress identity, concatenated-block decode. **Good (a).**
54. **miniz_oxide** — Pure-Rust DEFLATE (backs flate2). Properties: compress→decompress identity, gzip/zlib header roundtrip. **Good (a).**
55. **snap** — Pure-Rust Snappy. Properties: roundtrip, framing inverse.
56. **brotli** (Rust port) — Properties: roundtrip across quality levels.
57. **lzma-rs** — Pure-Rust LZMA/XZ. Properties: decode of known-good, partial compress roundtrip. **Good (a).**
58. **weezl** — LZW (GIF/TIFF). Properties: encode→decode roundtrip across code widths.

#### Geometry / numeric predicates
59. **geo** — github.com/georust/geo — Planar geometry algorithms. Properties: boolean-op set laws (A∪A=A, De Morgan), area/centroid invariance under translation, convex-hull membership; robustness/precision bugs likely. **Strong (a/b).**
60. **robust** — Adaptive FP geometric predicates (orient2d, incircle). Properties: sign consistency vs exact arithmetic, collinearity. Small but foundational.
61. **spade** — Delaunay/Voronoi. Properties: empty-circle invariant, hull coverage, Euler's formula.
62. **parry2d/parry3d** (Dimforge) — Collision detection. Properties: symmetric intersection tests, distance non-negativity, bounding-volume containment.
63. **euclid** (servo) — 2D/3D geometry types. Properties: transform inverse composition (T·T⁻¹=I), point/vector algebra.

### Category B — Boundary-pushing

#### Embedded databases & storage engines
64. **redb** — github.com/cberner/redb — Per its README, "A simple, portable, high-performance, ACID, embedded key-value store. redb is written in pure Rust and is loosely inspired by lmdb. Data is stored in a collection of copy-on-write B-trees," and is described as "Stable and maintained." Properties: model-based testing of insert/delete/range vs BTreeMap under random transaction+crash sequences, MVCC snapshot isolation, savepoint/rollback correctness. ~4,486 stars, v4.1.0 (2025). Already has a cargo-fuzz harness — still excellent for hegel-style stateful model tests. **Top target (b).** (All-time download figure ~7.7M is still unverified.)
65. **fjall** — github.com/fjall-rs/fjall — Pure-Rust LSM-tree KV storage engine. Properties: get/put/delete vs BTreeMap model across compaction, WAL crash-recovery consistency, snapshot isolation. ~174K downloads/month, "Fjall 3.0" announced (a stated plan), 100% safe Rust, PBT unconfirmed. **Top target (b).**
66. **sled** — "champagne of beta embedded databases." Properties: same model-based approach — but CHECK maintenance (development has slowed; verify recent commits first).
67. **sanakirja** — Copy-on-write B-tree DB backing Pijul. Properties: transactional B-tree invariants, fork/clone correctness.
68. **native_db / persy** — Higher-level embedded stores. Properties: model-based CRUD vs HashMap, ACID transaction model, index roundtrip.

#### Query / dataframe / analytics engines
69. **datafusion** — Apache SQL query engine. Properties: result equivalence under logical-plan rewrites, aggregation correctness vs naive implementation. **Boundary (b).**
70. **polars** (polars-core) — DataFrame engine. Properties: operation equivalence (filter∘filter = filter with AND), join correctness vs reference, sort stability. Pick under-tested corners.

#### WASM / language runtimes & compilers
71. **wasmi** — github.com/wasmi-labs/wasmi — Pure-Rust WASM interpreter. Properties: differential execution vs wasmtime/spec on generated modules, deterministic re-execution, fuel/gas metering monotonicity. ~2k stars, very active; likely some fuzzing exists. **Top target (b).** (All-time download figure ~13.6M is still unverified.)
72. **boa** — JavaScript engine in Rust. Properties: test262 conformance, parse roundtrip, spec-compliant coercions. Serious, active.
73. **rhai** — Embedded scripting language. Properties: expression evaluation vs reference, parse roundtrip, no-panic on arbitrary scripts.
74. **rune** — Embeddable dynamic language. Properties: parse→pretty→parse roundtrip, VM determinism, GC safety.
75. **starlark-rust** (Meta) — Starlark interpreter. Properties: deterministic evaluation, parse roundtrip, frozen-value immutability.
76. **full_moon** — Lua parser. Properties: parse→print roundtrip, AST invariants.
77. **cranelift** — WATCH: heavily fuzzed with differential verification as part of wasmtime; deprioritize.

#### Incremental computation / type systems / solvers
78. **salsa** — github.com/salsa-rs/salsa — Incremental computation framework (powers rust-analyzer). Properties: incremental recomputation equals from-scratch computation (core correctness invariant), memoization consistency under random input-mutation sequences. **Boundary (b), model-based.**
79. **chalk** — Rust trait-system solver. Properties: solver determinism, solution stability. Research-grade but serious.
80. **varisat / splr / batsat** — SAT solvers. Properties: returned model satisfies the formula (cheap to verify — the classic "check the witness" property), UNSAT-certificate checking, cross-solver satisfiability agreement. **Excellent (b).**
81. **good_lp / russcip** — LP/MILP modeling. Properties: solution feasibility, objective-bound consistency.

#### Layout, text shaping, terminal
82. **taffy** — github.com/DioxusLabs/taffy — Flexbox/CSS-Grid/Block layout engine. Per its README it "is designed to be used as a dependency for other UI and GUI libraries. Right now, it powers: Dioxus..."; it is also used by Blitz/Dioxus-Native (blitz-dom pairs Stylo + Taffy + Parley) and has a zed-industries fork. Properties: children fit within parent bounds, layout determinism, no-overlap invariants, comparison vs Chrome reference. ~137K downloads/month, ~3.3k stars, v0.8.1 (Apr 2025), uses rstest but no proptest/cargo-fuzz confirmed. **Top target (b).**
83. **rustybuzz** — github.com/harfbuzz/rustybuzz — Pure-Rust HarfBuzz shaping port (matches harfbuzz v10.1.0). Its README states "rustybuzz passes nearly all of harfbuzz shaping tests (2221 out of 2252 to be more precise)," and explicitly frames a differential fuzzer as future work: "One potential way of addressing this issue could be to create a fuzzer that takes random fonts, and shapes them with a random set of Unicode codepoints as well as input settings. In case of a discovered discrepancy, this test case could then be investigated and once the bug has been identified, added to our custom test suite." Properties: differential shaping vs HarfBuzz, no-panic on malformed fonts, cluster-mapping invariants. **Top target (b)** — the maintainers have essentially specified the hegel contribution for you.
84. **swash** — github.com/dfrg/swash — Font introspection/shaping/rendering. Properties: shaping cluster invariants, variation-axis interpolation bounds. Pure Rust (some unsafe SIMD).
85. **cosmic-text** (System76) — Text layout/editing. Properties: edit ops vs buffer model, cursor-movement roundtrip, line-wrapping invariants. Active.
86. **vte** — github.com/alacritty/vte — ANSI/VT escape parser (Paul Williams state machine; ~876K downloads/month, used in 1,015 crates). Properties: parser never panics on arbitrary bytes, state-machine invariants, parse consistency. **Good (b).**
87. **ttf-parser** (RazrFalcon) — Font parsing. Properties: no-panic on arbitrary bytes, table roundtrip. Backs rustybuzz.

#### Version control internals
88. **gix / gitoxide** — github.com/GitoxideLabs/gitoxide — Pure-Rust Git implementation (powers jujutsu, cargo tooling). Properties: object encode→decode→hash roundtrip, pack encode/decode, ref-transaction atomicity, gix-url parse roundtrip. Very active (releases days apart); plumbing crates already have cargo-fuzz targets — target under-fuzzed corners (config, refspec, revision parsing). **Top target (b).**
89. **jj-lib** — github.com/jj-vcs/jj — Jujutsu VCS library. Properties: operation-log undo/redo roundtrip, commit-graph invariants, conflict-representation correctness, rebase idempotence. Serious, active. **Top target (b).**
90. **gix-config** — Git config parser. Properties: parse→serialize roundtrip, section/value invariants.

#### CRDTs, concurrency, protocols
91. **yrs** — github.com/y-crdt/y-crdt — Rust port of Yjs CRDT. Properties: convergence (concurrent ops on replicas converge), commutativity, state-vector encode/decode roundtrip. ~1.76M downloads, v0.27.2 (2026), pure-Rust core, PBT unconfirmed. **Top target (b)** — CRDT convergence is the calibration domain (automerge).
92. **diamond-types** — High-performance text CRDT. Properties: convergence, merge associativity, vs reference OT.
93. **loro** — CRDT framework (rich text/collab). Properties: convergence, snapshot encode/decode roundtrip, time-travel consistency. Active, serious. **Good (b).**
94. **automerge** — CALIBRATION EXAMPLE (bugs already found); use for new properties only.
95. **quinn / quinn-proto** — github.com/quinn-rs/quinn — Pure-Rust QUIC. `quinn-proto` is a deterministic sans-I/O state machine ideal for model-based testing. Properties: packet encode/decode roundtrip, stream flow-control invariants, connection-state transitions. **Boundary (b).**
96. **hickory-dns** (formerly trust-dns) — DNS implementation. Properties: message encode/decode roundtrip, name-compression inverse, DNSSEC record parsing. **Good (b).**
97. **rumqtt** — MQTT. Properties: packet encode/decode roundtrip, QoS state machine.
98. **openraft / raft-rs** (TiKV) — Raft consensus. Properties: safety invariants (election safety, log matching) under random partition/message-reorder schedules. Model-checking-adjacent. **Boundary (b).**
99. **quiche** (Cloudflare) — QUIC/HTTP-3. Properties: frame roundtrip — but heavily production-tested; slightly lower priority.
100. **rustls** — WATCH: security-critical and heavily fuzzed; deprioritize.

#### Additional codec targets (verify hardening first)
101. **zune-png / zune-jpeg** — github.com/etemesi254/zune-image — Fast pure-Rust image decoders. Properties: decode→encode→decode roundtrip, no-panic on malformed input. **Already fuzz-tested with a fuzz corpus** — viable only for new (e.g., encode-side) properties. zune-jpeg ~44.8M downloads all-time.
102. **lodepng-rust** — Pure-Rust PNG encoder/decoder. Properties: encode→decode roundtrip, color-format conversions.

## Recommendations

**Stage 1 — Prove the skill on high-yield clean-property targets (weeks 1–2).** Start with clean-property crates where a reference model is trivial and existing PBT is thin: **bigdecimal, uom, roaring, toml_edit, sqlparser, rpds, crop/ropey, petgraph, data-encoding, cbor2**. These maximize the chance of quick, showcase-worthy bugs (MacIver's categories 1 and 2) and let you iterate the hegel-skill's generator authoring rapidly. Benchmark to change course: if you are not finding at least one panic or roundtrip discrepancy per ~3 crates, revisit generator quality (bias toward empty/zero/huge/Unicode-cursed inputs).

**Stage 2 — Move to stateful model-based targets (weeks 3–5).** Tackle **redb, fjall, taffy, salsa, wasmi, quinn-proto, yrs, jj-lib** with operation-sequence generators diffed against in-memory models. These produce the "complicated structural invariant" bugs Antithesis prizes and make the best showcase. Prioritize crates whose maintainers visibly welcome PRs (redb, taffy, jiff, gitoxide, sqlparser are all responsive), so upstreaming test contributions is smooth.

**Stage 3 — Differential-testing showpieces (weeks 5+).** Do **rustybuzz vs HarfBuzz** (its README explicitly specifies this fuzzer as future work), **wasmi vs wasmtime/spec**, **geo boolean-ops vs a reference**, and **SAT-solver witness verification (varisat/splr/batsat)**. These are the most compelling "if you can test it, you can property-test it" demonstrations for a showcase blog post.

**What would change the plan:** If a target turns out to have a mature `fuzz/` directory with OSS-Fuzz coverage (rasn, zune-jpeg, cranelift, rustls, base64, url, aho-corasick, serde_json), drop it down the list — the marginal greenfield bug yield is low. If a candidate fails to build on current stable/nightly, exclude it immediately per the hard criterion. If roaring-rs or sled show no 2025+ activity on inspection, substitute a livelier alternative (e.g., another bitmap/embedded-DB crate) rather than risk an unmaintained target.

**Maintenance/licensing:** All primary recommendations are OSS (mostly MIT/Apache-2.0; malachite is LGPL-3.0, acceptable) and confirmed active in 2025–2026 except **roaring-rs** (last release Dec 2024 — verify) and **sled** (slowed development — verify before investing).

## Caveats
- **Metrics precision:** GitHub star counts were firmly verified only for redb (~4,486), wasmi (~2k), rasn (~331), and taffy (~3,285, possibly dated). Several exact all-time download figures remain unverified — notably wasmi (~13.6M) and redb (~7.7M), plus rustybuzz, the gix main crate, sqlparser, and zune-png. Verify before publishing any specific number.
- **Pure-Rust look-alikes:** Use **roaring** not `croaring` (C FFI), **rustybuzz** not `harfbuzz-sys`, and note **wasmi/yrs** offer optional C-APIs but their cores are pure Rust. `zstd`, `z3.rs`, and stb-image bindings are C-FFI (acceptable per criteria but heavier).
- **Existing hardening:** rasn (AFL++), zune-jpeg/zune-png (fuzz corpus), malachite (built-in exhaustive/random generators), and likely redb, gitoxide, jiff, and wasmi already have fuzzing/PBT — they remain viable for *new* property types (especially stateful/model-based) but are not greenfield.
- **Blog overlap:** fraction, rust_decimal, heck, and im-rs already have published Hegel-found bugs; do not re-report those specific bugs.
- **Speculative language:** some descriptions reflect stated plans, not shipped features — e.g., "Fjall 3.0" is announced, rustybuzz's differential fuzzer is explicitly framed as a potential future addition, and jiff remains pre-1.0 with no committed 1.0 date as of April 2026.
- **Coverage confidence:** I could not independently confirm current commit dates for every one of the ~100 crates within budget. Treat the ~15 metrics-verified crates (bigdecimal, malachite, uom, redb, fjall, jiff, rustybuzz, taffy, gix, rasn, roaring, sqlparser, wasmi, zune-*, yrs) as high-confidence and the remainder as strong-but-verify candidates.
