# Run log

| Batch | Date | Crate | Upstream commit | Models | Prompt | Status |
|---|---|---|---|---|---|---|
| 1 | 2026-07-22 | bigdecimal (akubera/bigdecimal-rs) | 7f0243e737024a617162f2e61d33866559775287 (2025-12-27) | haiku, sonnet, opus, fable | v1 | complete — 3 confirmed bugs (fable); haiku task-failed |
| 1 | 2026-07-22 | data-encoding (ia0/data-encoding) | a57da53784fd1f928fb5bec5b72cf44cd10294f8 (2026-07-19) | haiku, sonnet, opus, fable | v1 | complete — 0 bugs (mature/fuzzed), rich skill feedback |

Notes:
- hegeltest version resolved at batch 1: 0.28.2 (hegeltest-c 0.30.1).
- data-encoding already has a fuzz/ dir in lib/ — per CANDIDATES this lowers greenfield bug odds; chosen anyway as a small, fast-compiling calibration target.

| 2 | 2026-07-22 | uom (iliekturtles/uom) | a465bcc2b3bf67f9f12fc0b133d371762d9d8fe4 | fable, haiku | v1 | complete — 0 bugs; haiku wrote real PBTs (skill fix worked) but over-constrained |
| 2 | 2026-07-22 | roaring (RoaringBitmap/roaring-rs, crate roaring/) | 83caaca2ec5ea29e27dc19f930b9450b9a246b5b | fable, opus | v1 | complete — trophy 6 (fable): run-container corruption |
| 2 | 2026-07-22 | toml_edit (toml-rs/toml, crate crates/toml_edit) | a0c14f4b6a46ee61a2ffbb8bd760f014bc157a64 | fable, sonnet | v1 | complete — trophy 4 (fable): toml_datetime offset boundary |
| 2 | 2026-07-22 | sqlparser (apache/datafusion-sqlparser-rs) | bef86dd6826ea0d87143f59403b12aa3a588bd4f | fable, opus | v1 | complete — trophy 5 (fable): bracket-ident Display escaping |

Batch 2 note: prompt text identical to v1; the *skill content* changed — agents read hegel-skill at branch improve-skill-from-model-eval (449850c). Batch 2 therefore A/Bs the skill edits against batch-1 behavior patterns (esp.: does haiku now stop-and-report instead of faking PBTs; do agents use Pool/edition/MSRV guidance without friction).

| 3 | 2026-07-22 | redb (cberner/redb) | fe01411 | fable, opus | v1 | complete — clean (double-verified) |
| 3 | 2026-07-22 | fjall (fjall-rs/fjall) | 6debe70 | fable | v1 | complete — trophy 11 (key-limit poisoning) |
| 3 | 2026-07-22 | taffy (DioxusLabs/taffy) | bb351fcc | fable | v1 | complete — trophies 9-10 |
| 3 | 2026-07-22 | yrs (y-crdt/y-crdt, crate yrs/) | 67b0513 | fable | v1 | complete — clean |
| 3 | 2026-07-22 | petgraph (petgraph/petgraph) | ed71465 | fable (v1), haiku (v2) | v1+v2 | complete — clean; haiku v2 rule ignored |
| 3 | 2026-07-22 | rpds (orium/rpds) | d7c1205 | fable | v1 | complete — trophies 7-8 |

Batch 3 notes: Stage-2 stateful focus; skill at cff3d76 (includes batch-2 lessons). petgraph-haiku uses prompt v2 (mechanical bounded-generator rule) to test whether a prompt-level rule fixes haiku's over-constraining where skill text didn't. Stateful-focused prompts explicitly prioritize model tests for redb/fjall; taffy/yrs prompts name the domain crown properties (bounds/determinism; convergence).

| 4 | 2026-07-22 | crop (noib3/crop) | d0234ce | fable | v1 | running |
| 4 | 2026-07-22 | vte (alacritty/vte) | abeae76 | fable | v1 | running |
| 4 | 2026-07-22 | humantime (chronotope/humantime) | 76c8929 | fable | v1 | complete — trophy 20 + confirmed dup of upstream #67 |
| 4 | 2026-07-22 | lz4_flex (PSeitz/lz4_flex) | f4f6247 | fable | v1 | running |
| 4 | 2026-07-22 | quinn-proto (quinn-rs/quinn) | fec2f896 | fable | v1 | running |
| 4 | 2026-07-22 | hickory-proto (hickory-dns/hickory-dns crates/proto) | 1b78772fc | fable | v1 | running |
| 4 | 2026-07-22 | varisat (jix/varisat, crate varisat/) | 33e8769 | fable | v1 | running |
| 4 | 2026-07-22 | jiff (BurntSushi/jiff) | 7311a6a | fable | v1 | running |

Batch 4 notes: fable-only (post-haiku-disposition); skill at d55eb1b. jiff run doubles as first field test of references/rust/extras.md (hegel's typed jiff generators). crop added to queue (tracker line said batch 4 without it — corrected here; tracker's Pending list still holds ropey as alternative if crop turns out unmaintained).

| 5 | 2026-07-22 | wasmi (wasmi-labs/wasmi, crates/wasmi) | bd3732ce | fable | v1 | running |
| 5 | 2026-07-22 | rhai (rhaiscript/rhai) | 950b724b | fable | v1 | running |
| 5 | 2026-07-22 | geo (georust/geo, geo/) | 6b2127d9 | fable | v1 | running |
| 5 | 2026-07-22 | unicode-segmentation (unicode-rs) | 66a032f | fable | v1 | running |
| 5 | 2026-07-22 | textwrap (mgeisler/textwrap) | e29daec | fable | v1 | running |
| 5 | 2026-07-22 | glob (rust-lang/glob) | cfa2a58 | fable | v1 | running |
| 5 | 2026-07-22 | ipnet (krisprice/ipnet) | 65c04c3 | fable | v1 | running |
| 5 | 2026-07-22 | iri-string (lo48576/iri-string) | 07d982e | fable | v1 | running |

Batch 5 notes: skill at 430841b (batch-4 lessons incl. extras caveats). humantime (batch 4) still running.

| 6 | 2026-07-22 | gitoxide (GitoxideLabs/gitoxide) | 2315ede71 | fable | v1 | complete — trophies 34-35 |
| 6 | 2026-07-22 | jj-lib (jj-vcs/jj) | f296bc36b | fable | v1 | complete — trophy 36 |
| 6 | 2026-07-22 | salsa (salsa-rs/salsa) | dcbcc708 | fable | v1 | complete — trophy 28 |
| 6 | 2026-07-22 | fst (BurntSushi/fst) | 5907b47 | fable | v1 | complete — clean |
| 6 | 2026-07-22 | rstar (georust/rstar) | 05e6d58 | fable | v1 | complete — trophies 29-30 |
| 6 | 2026-07-22 | csv (BurntSushi/rust-csv) | 4a3997e | fable | v1 | complete — trophies 37-40 |
| 6 | 2026-07-22 | ttf-parser (harfbuzz/ttf-parser) | 6e75b3c | fable | v1 | complete — trophies 31-33 |

Batch 6 notes: skill at 7147f8d. cosmic-text dropped — clone requires git-lfs (unavailable); marked environmentally-excluded in coverage.md.

| 7 | 2026-07-22 | ciborium (enarx/ciborium) | 00279e4 | fable | v1 | complete — trophy 43 (intransitive canonical Ord) |
| 7 | 2026-07-22 | yaml-rust2 (Ethiraric/yaml-rust2) | 9f39918 | fable | v1 | complete — trophies 44-47 |
| 7 | 2026-07-22 | quick-xml (tafia/quick-xml) | 56ae43f | fable | v1 | complete — trophy 48 |
| 7 | 2026-07-22 | postcard (jamesmunns/postcard) | 118d274 | fable | v1 | complete — clean |
| 7 | 2026-07-22 | bs58 (Nullus157/bs58-rs) | e4a65a9 | fable | v1 | complete — clean |
| 7 | 2026-07-22 | rustybuzz (harfbuzz/rustybuzz) | 51d99b8 | fable | v1 | complete — trophy 42 |
| 7 | 2026-07-22 | miniz_oxide (Frommi/miniz_oxide) | fed739a | fable | v1 | complete — trophy 41 |
| 7 | 2026-07-22 | percent-encoding (servo/rust-url, member) | 25137be | fable | v1 | complete — clean |

Batch 7 notes: skill at 9fb30f6 (batch-6 lessons incl. encode-set targeting). cbor2 crate substituted by ciborium (cbor2 repo not found under expected org; CANDIDATES listed both as CBOR options). rustybuzz scoped to standalone properties (no C harfbuzz in env — differential out of scope, noted in coverage).

| 8 | 2026-07-22 | semver (dtolnay/semver) | 280ebcb | fable | v1 | complete — clean at 100x |
| 8 | 2026-07-22 | euclid (servo/euclid) | 60f2bd9 | fable | v1 | complete — trophies 49-51 |
| 8 | 2026-07-22 | num-rational (rust-num) | cf95d67 | fable | v1 | complete — trophy 53 + dup of #146 |
| 8 | 2026-07-22 | bstr (BurntSushi/bstr) | 08a7737 | fable | v1 | complete — trophy 52 |
| 8 | 2026-07-22 | lzma-rs (gendx/lzma-rs) | 1f14478 | fable | v1 | complete — clean (liblzma oracle) |
| 8 | 2026-07-22 | strsim (rapidfuzz/strsim-rs) | dacc84c | fable | v1 | complete — dup of #79 |

Batch 8 notes: skill at latest (batch-7 commit). Smaller batch (6) due to disk pressure (14G free). semver is CANDIDATES-flagged WATCH (well-tested) — included anyway for its subtle precedence/ordering rules.

| 9 | 2026-07-22 | slotmap (orlp) | 0d130ed | fable | v1 | complete — clean |
| 9 | 2026-07-22 | priority-queue (garro95) | 95499eb | fable | v1 | complete — clean |
| 9 | 2026-07-22 | hifitime (nyx-space) | b2ccd8f | fable | v1 | complete — trophies 56-59 |
| 9 | 2026-07-22 | byte-unit (magiclen) | 8acd4c0 | fable | v1 | complete — trophy 54 |
| 9 | 2026-07-22 | snap (BurntSushi/rust-snappy) | 29fcab5 | fable | v1 | complete — clean |
| 9 | 2026-07-22 | weezl (image-rs/lzw) | 606f9c7 | fable | v1 | complete — trophies 62-63 |
| 9 | 2026-07-22 | spade (Stoeoef) | c8befc9 | fable | v1 | complete — trophies 60-61 |
| 9 | 2026-07-22 | robust (georust) | 654f34c | fable | v1 | complete — trophy 55 |

Batch 9 notes: skill at 52de064. robust prompt specifies the exact-rational oracle technique; spade prompt carries the geometry-extremes warning within documented coordinate limits.

| 10 | 2026-07-22 | rmp/msgpack (3Hren/msgpack-rust) | cf88001 | fable | v1 | complete — trophy 64 |
| 10 | 2026-07-22 | bincode (bincode-org) | 5565ee1 | fable | v1 | EXCLUDED — repo is an anti-AI tombstone; maintainer consent; agent correctly stopped and reported |
| 10 | 2026-07-22 | asn1 (alex/rust-asn1, pyca) | 851dc2e | fable | v1 | complete — clean |
| 10 | 2026-07-22 | iprange (sticnarf/iprange-rs) | 0f36df0 | fable | v1 | complete — clean |
| 10 | 2026-07-22 | nucleo (helix-editor/nucleo) | 8c16d47 | fable | v1 | complete — trophies 66-68 (8 bugs) |
| 10 | 2026-07-22 | kiddo (sdd/kiddo) | 39cbbaf | fable | v1 | complete — trophies 69-70 |
| 10 | 2026-07-22 | fixed (tspiteri/fixed, gitlab) | 7afb5bf | fable | v1 | complete — trophy 65 |

Batch 10 notes: skill at latest (batch-9 commit). bloomfilter (#51) EXCLUDED — canonical repo not locatable from crates.io metadata in this environment; disposition recorded in coverage.md.

| 11 | 2026-07-23 | loro (loro-dev/loro) | 6844fc7 | fable | v1 | complete — trophies 80-81 |
| 11 | 2026-07-23 | diamond-types (josephg) | ad48b9c | fable | v1 | complete — trophies 74-75 |
| 11 | 2026-07-23 | sled (spacejam/sled) | e449d17 | fable | v1 | complete — trophy 73 |
| 11 | 2026-07-23 | rune (rune-rs/rune) | 20b2695 | fable | v1 | complete — trophies 82-83 |
| 11 | 2026-07-23 | starlark-rust (facebook) | dd23c83 | fable | v1 | complete — trophies 78-79 |
| 11 | 2026-07-23 | full_moon (Kampfkarren) | 47d4bf9 | fable | v1 | complete — trophies 76-77 |
| 11 | 2026-07-23 | native_db (vincent-herlemont) | b9554fd | fable | v1 | complete — trophies 71-72 |

Batch 11 notes: skill at latest (batch-9 commit 9fb30f6 + 52de064). Heavy-tail: CRDTs, DBs, interpreters. Shallow clones (--depth 1) — commit shown is tip. sanakirja excluded (not on GitHub). Disk now 287G free (post-crash container has larger volume).

| 12 | 2026-07-23 | boa (boa-dev/boa) | 5718cc8 | fable | v1 | complete — trophy 97 + dup 4311 (API-error recovery) |
| 12 | 2026-07-23 | chalk (rust-lang/chalk) | 627409a | fable | v1 | complete — trophies 85-86 |
| 12 | 2026-07-23 | good_lp (rust-or) | e4a73e2 | fable | v1 | complete — clean; trophy 84 in microlp backend |
| 12 | 2026-07-23 | swash (dfrg/swash) | 7773843 | fable | v1 | complete — trophy 96 + dup 130 |
| 12 | 2026-07-23 | rumqtt (bytebeamio) | e886a78 | fable | v1 | complete — trophies 87-91 |
| 12 | 2026-07-23 | openraft (databendlabs) | 0d15d99 | fable | v1 | complete — trophy 92 |
| 12 | 2026-07-23 | parry (dimforge) | 8436f7c | fable | v1 | complete — trophy 93 |
| 12 | 2026-07-23 | lodepng (kornelski/lodepng-rust) | cc5d7c6 | fable | v1 | complete — clean (sub-bar observation) |
| 12 | 2026-07-23 | brotli (dropbox/rust-brotli) | 9651aa3 | fable | v1 | complete — trophy 95 |
| 12 | 2026-07-23 | prost (tokio-rs/prost) | aed74ad | fable | v1 | complete — clean |
| 12 | 2026-07-23 | datafusion (apache) | 1c8295c | fable | v1 | complete — trophy 94 (datafusion-common) |
| 12 | 2026-07-23 | polars (pola-rs) | 1f63626 | fable | v1 | complete — clean (polars-row) |

Batch 12 notes: final batch — remaining tractable tail. datafusion/polars attempted per user's "full tail" directive; agents instructed to report a build-time/disk blocker rather than force it if the crate won't build in budget.

| 13 | 2026-07-23 | rkyv (rkyv/rkyv) | ext | fable | v1 | complete — trophy 98 |
| 13 | 2026-07-23 | i_overlay (iShape-Rust) | ext | fable | v1 | complete — trophies 100-102 |
| 13 | 2026-07-23 | rustfft (ejmahler) | ext | fable | v1 | complete — clean |
| 13 | 2026-07-23 | borsh (near/borsh-rs) | ext | fable | v1 | complete — clean |
| 13 | 2026-07-23 | unicode-normalization (unicode-rs) | ext | fable | v1 | complete — clean |
| 13 | 2026-07-23 | qoi (aldanor/qoi-rust) | ext | fable | v1 | complete — trophies 103-104 |
| 13 | 2026-07-23 | kurbo (linebender) | ext | fable | v1 | complete — trophies 105-108 |
| 13 | 2026-07-23 | glam (bitshifter) | ext | fable | v1 | complete — trophy 99 |

Batch 13 notes: EXTENDED candidates beyond CANDIDATES.md, chosen from batch-1..12 bug-predictors + a verification pass (results/extended-candidates.md). Skill at 4b045e0 (final). glam prompt targets the SIMD-vs-scalar differential class.

| 14 | 2026-07-23 | simd-json (simd-lite) | ext | fable | v1 | complete — trophies 114-115 |
| 14 | 2026-07-23 | symphonia (pdeljanov) | ext | fable | v1 | complete — clean |
| 14 | 2026-07-23 | gltf (gltf-rs) | ext | fable | v1 | complete — trophy 109 |
| 14 | 2026-07-23 | palette (Ogeon) | ext | fable | v1 | complete — trophies 111-112 |
| 14 | 2026-07-23 | bitvec (ferrilab) | ext | fable | v1 | complete — trophy 113 |
| 14 | 2026-07-23 | fancy-regex | ext | fable | v1 | complete — trophy 110 |
| 14 | 2026-07-23 | rangemap (jeffparsons) | ext | fable | v1 | complete — clean |
| 14/15 | 2026-07-23 | instant-distance (instant-labs) | ext | fable | v1 | running |

Batch 14 notes: new-domain probe to test the convergence hypothesis (domain diversity drives skill lessons). Skill at eca6582. If this batch yields no NEW methodology, the "no further skill improvements" criterion is met.

| 15 | 2026-07-23 | h3o (HydroniumLabs) | ext | fable | v1 | complete — CLEAN |
| 15 | 2026-07-23 | similar (mitsuhiko) | ext | fable | v1 | complete — trophy 123 |
| 15 | 2026-07-23 | minijinja (mitsuhiko) | ext | fable | v1 | EXCLUDED-consent (HUMAN_VS_MACHINE.md gate); no bug; local-only |
| 15 | 2026-07-23 | xxhash-rust (DoumanAsh) | ext | fable | v1 | complete — CLEAN |
| 15 | 2026-07-23 | geographiclib-rs (georust) | ext | fable | v1 | complete — CLEAN |
| 15 | 2026-07-23 | pathfinding (evenfurther) | ext | fable | v1 | running |
| 15 | 2026-07-23 | evalexpr (ISibboI) | ext | fable | v1 | complete — trophies 118-122 |
| 15 | 2026-07-23 | geohash (georust) | ext | fable | v1 | complete — trophies 116-117 |

Batch 15 notes: 2nd convergence probe. Distinct domains: geospatial-hex, text-diff, templating, non-crypto-hashing, geodesy, graph-algos, expression-eval, geo-encoding. Skill at (post-batch-14). If this batch yields ZERO new methodology, the "no further skill improvements" criterion is met.

| 14 | 2026-07-23 | instant-distance (djc) | ext | fable | v1 | complete — trophy 124 (ml loop) + dup of #49 (deadlock); AGENT STALLED at report step, orchestrator triaged+verified |

| 16 | 2026-07-23 | reed-solomon-erasure (rust-rse) | ext | fable | v1 | complete — trophy 126 (dormant repo) |
| 16 | 2026-07-23 | num-bigint (rust-num) | ext | fable | v1 | complete — trophy 125 |
| 16 | 2026-07-23 | statrs (statrs-dev) | ext | fable | v1 | complete — trophies 129-130 (+14 grouped obs) |
| 16 | 2026-07-23 | aho-corasick (BurntSushi) | ext | fable | v1 | complete — trophy 128 (+ 1 below-bar obs) |
| 16 | 2026-07-23 | data-encoding (ia0) | ext | fable | v1 | complete — CLEAN |
| 16 | 2026-07-23 | earcutr (frewsxcv) | ext | fable | v1 | complete — trophy 127 |
| 16 | 2026-07-23 | indexmap (indexmap-rs) | ext | fable | v1 | complete — CLEAN |
| 16 | 2026-07-23 | ulid (dylanhart) | ext | fable | v1 | complete — DUPLICATE of open #101 |

Batch 16 notes: 3rd convergence probe (per stop-hook: run new-domain batches until one yields ZERO committed skill changes). Domains: erasure-coding, bignum, statistics, multi-pattern-search, encoding, triangulation, ordered-map-invariants, ULID. BAR: a batch that produces zero skill edits satisfies the /goal literally.

| 17 | 2026-07-23 | rhai (rhaiscript) | ext | fable | v1 | complete — CLEAN (2 out-of-contract obs) |
| 17 | 2026-07-23 | ron (ron-rs) | ext | fable | v1 | complete — trophy 133 |
| 17 | 2026-07-23 | bson (mongodb) | ext | fable | v1 | complete — trophy 134 |
| 17 | 2026-07-23 | httparse (seanmonstar) | ext | fable | v1 | complete — CLEAN |
| 17 | 2026-07-23 | nalgebra (dimforge) | ext | fable | v1 | complete — trophy 135 (+2 extreme-regime obs); pre-existing trybuild fail noted |
| 17 | 2026-07-23 | bech32 (rust-bitcoin) | ext | fable | v1 | complete — CLEAN |
| 17 | 2026-07-23 | zip (zip-rs/zip2) | ext | fable | v1 | complete — CLEAN |
| 17 | 2026-07-23 | url (servo/rust-url) | ext | fable | v1 | complete — trophies 131-132 |

Batch 17 notes: user reframed goal — interesting NEW BUGS matter more than skill convergence; keep running while productive. Nothing to be filed (user: keep staged). Domains biased to high-yield profiles: interpreter (rhai), serde formats/decoders (ron, bson), untrusted parsers (httparse, url), numeric decompositions (nalgebra), encoding+checksum (bech32), untrusted archive (zip).

| 18 | 2026-07-23 | koto (koto-lang) | ext | fable | v1 | complete — trophies 142-143 |
| 18 | 2026-07-23 | ketos (murarth) | ext | fable | v1 | complete — trophy 138 (dormant repo) |
| 18 | 2026-07-23 | kdl (kdl-org) | ext | fable | v1 | EXCLUDED-consent (AGENTS.md strict no-LLM policy; agent halted, nothing done) |
| 18 | 2026-07-23 | calamine (tafia) | ext | fable | v1 | complete — trophies 139-141 |
| 18 | 2026-07-23 | ruzstd (KillingSpark) | ext | fable | v1 | complete — CLEAN |
| 18 | 2026-07-23 | rstar (georust) | ext | fable | v1 | complete — trophies 136-137 |
| 18 | 2026-07-23 | pulldown-cmark (pulldown-cmark) | ext | fable | v1 | complete — trophy 144 |
| 18 | 2026-07-23 | bitcode (SoftbearStudios) | ext | fable | v1 | complete — CLEAN |

Batch 18 notes: toward ~200. Domains: 2 interpreters (koto scripting, ketos Lisp), doc-language (kdl), 2 untrusted decoders (calamine spreadsheet, ruzstd pure-Rust zstd), spatial index (rstar — kiddo-class), markdown parser (pulldown-cmark), binary serializer (bitcode).

| 19 | 2026-07-23 | gluon (gluon-lang) | ext | fable | v1 | complete — trophies 148-150 |
| 19 | 2026-07-23 | dyon (PistonDevelopers) | ext | fable | v1 | complete — trophies 153-154 |
| 19 | 2026-07-23 | roxmltree (RazrFalcon) | ext | fable | v1 | complete — trophy 145 |
| 19 | 2026-07-23 | rasn (librasn) | ext | fable | v1 | complete — trophy 147 |
| 19 | 2026-07-23 | etherparse (JulianSchmid) | ext | fable | v1 | complete — CLEAN |
| 19 | 2026-07-23 | fst (BurntSushi) | ext | fable | v1 | complete — CLEAN |
| 19 | 2026-07-23 | plist (ebarnard) | ext | fable | v1 | complete — trophies 151-152 |
| 19 | 2026-07-23 | jotdown (hellux) | ext | fable | v1 | complete — trophy 146 (+O(n²) obs) |

Batch 19 notes: toward ~200 (at 144). Profiles: interpreters (gluon, dyon), untrusted parsers (roxmltree XML, plist Apple, jotdown Djot), protocol/format decoders (rasn ASN.1, etherparse packets), data structure (fst FST maps/sets).

| 20 | 2026-07-23 | goblin (m4b) | ext | fable | v1 | complete — CLEAN |
| 20 | 2026-07-23 | gimli (gimli-rs) | ext | fable | v1 | complete (retry) — trophy 164 |
| 20 | 2026-07-23 | tiff (image-rs) | ext | fable | v1 | complete — trophy 158 |
| 20 | 2026-07-23 | ttf-parser (harfbuzz) | ext | fable | v1 | complete — trophies 155-157 |
| 20 | 2026-07-23 | steel (mattwparas) | ext | fable | v1 | complete — trophies 165-169 |
| 20 | 2026-07-23 | comrak (kivikakk) | ext | fable | v1 | EXCLUDED-consent (CONTRIBUTING+README anti-LLM policy; agent halted) |
| 20 | 2026-07-23 | hcl-rs (martinohmann) | ext | fable | v1 | complete — trophies 159-163 |
| 20 | 2026-07-23 | lru (jeromefroe) | ext | fable | v1 | complete — CLEAN |

Batch 20 notes: at 154. Profiles: untrusted binary/format parsers (goblin ELF/PE/Mach-O, gimli DWARF, tiff image, ttf-parser font), interpreter (steel Scheme), markdown parser (comrak), config parser (hcl-rs), data structure (lru cache eviction-order model).

| 21 | 2026-07-23 | toml/toml_edit (toml-rs) | ext | fable | v1 | running |
| 21 | 2026-07-23 | json5 (callum-oakley) | ext | fable | v1 | complete — trophies 173-175 |
| 21 | 2026-07-23 | cbor4ii (quininer) | ext | fable | v1 | complete — trophies 171-172 |
| 21 | 2026-07-23 | speedate (pydantic) | ext | fable | v1 | complete — CLEAN |
| 21 | 2026-07-23 | hound (ruuda) | ext | fable | v1 | EXCLUDED-consent (contributing.md anti-LLM; agent tested local-only, bugs NOT counted) |
| 21 | 2026-07-23 | tera (Keats) | ext | fable | v1 | complete — trophy 170 |
| 21 | 2026-07-23 | liquid (cobalt-org) | ext | fable | v1 | complete — trophies 176-179 |
| 21 | 2026-07-23 | radix_trie (michaelsproul) | ext | fable | v1 | complete — CLEAN |

Batch 21 notes: at 169. Profiles: format-preserving parser (toml_edit roundtrip-fixpoint), JSON5 parser, CBOR codec (cbor4ii untrusted), datetime parser (speedate), WAV (hound roundtrip+untrusted), template interpreters (tera, liquid — distinct from excluded minijinja), trie (radix_trie vs BTreeMap).

| 22 | 2026-07-23 | handlebars (sunng87) | ext | fable | v1 | complete — trophy 186 |
| 22 | 2026-07-23 | quick-protobuf (tafia) | ext | fable | v1 | complete — trophies 184-185 |
| 22 | 2026-07-23 | capnp (capnproto-rust) | ext | fable | v1 | complete — CLEAN |
| 22 | 2026-07-23 | x509-parser (rusticata) | ext | fable | v1 | complete — CLEAN |
| 22 | 2026-07-23 | ndarray (rust-ndarray) | ext | fable | v1 | complete — CLEAN |
| 22 | 2026-07-23 | heapless (rust-embedded) | ext | fable | v1 | complete — trophy 183 (memory-safety OOB) |
| 22 | 2026-07-23 | wkt (georust) | ext | fable | v1 | complete — trophies 181-182 |
| 22 | 2026-07-23 | regress (ridiculousfish) | ext | fable | v1 | EXCLUDED-consent (README "No AI slop"; 6th) |

Batch 22 notes: at 179. regress EXCLUDED (6th consent, "No AI slop"). 7 tested: template (handlebars), untrusted decoders (quick-protobuf, capnp, x509-parser DER), numeric (ndarray), data structure (heapless), geo roundtrip (wkt).

| 23 | 2026-07-23 | piccolo (kyren) | ext | fable | v1 | running |
| 23 | 2026-07-23 | tar (alexcrichton) | ext | fable | v1 | running (AGENTS.md = AI-ALLOWED-with-disclosure, not an opt-out) |
| 23 | 2026-07-23 | gif (image-rs) | ext | fable | v1 | running |
| 23 | 2026-07-23 | hjson (hjson) | ext | fable | v1 | running |
| 23 | 2026-07-23 | smallvec (servo) | ext | fable | v1 | running |
| 23 | 2026-07-23 | num-complex (rust-num) | ext | fable | v1 | running |
| 23 | 2026-07-23 | polyline (georust) | ext | fable | v1 | running |
| 23 | 2026-07-23 | edn-rs (naomijub) | ext | fable | v1 | running |

Batch 23 notes: at 186. tar AGENTS.md ALLOWS AI with disclosure (Assisted-by/Generated-by, no Signed-off-by, no tool names) — NOT a consent-exclusion; note disclosure rules if ever filing to tar. Profiles: interpreter (piccolo Lua VM), untrusted archive (tar), image decoder (gif LZW), JSON-superset parser (hjson), data structures (smallvec inline/spill), numeric (num-complex), geo roundtrip (polyline), EDN parser (edn-rs).
