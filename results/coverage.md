# Candidate coverage tracker

Disposition per CANDIDATES.md numbered entry. Statuses: **done** (agent-evaluated + triaged), **queued**, **excluded** (with CANDIDATES.md's own rationale), **pending**.

## Evaluated
| Crate | Batch | Outcome |
|---|---|---|
| bigdecimal (#1) | 1 | 3 confirmed bugs (trophies 1-3) |
| data-encoding (#20) | 1 | 0 bugs (mature/fuzzed) |
| uom (#3) | 2 | 0 bugs |
| roaring (#41) | 2 | trophy 6 (run-container corruption) |
| toml_edit (#9) | 2 | trophy 4 (in sibling toml_datetime) |
| sqlparser (#24) | 2 | trophy 5 (bracket-ident Display) |
| redb (#64) | 3 | clean (double-verified, fable+opus) |
| fjall (#65) | 3 | trophy 11 (key-limit → DB poisoning) |
| taffy (#82) | 3 | trophies 9, 10 (cache invalidation) |
| yrs (#91) | 3 | clean (convergence held at 10x) |
| petgraph (#44) | 3 | clean (strong oracles) |
| rpds (#42) | 3 | trophies 7, 8 (constructor validation) |
| crop (#43) | 4 | trophy 16 (doc gap) |
| vte (#86) | 4 | trophies 12, 13 |
| humantime (#30) | 4 | trophy 20 + dup of #67 |
| lz4_flex (#53) | 4 | trophies 14, 15 |
| quinn-proto (#95) | 4 | trophy 18 |
| hickory-proto (#96) | 4 | trophy 17 |
| varisat (#80) | 4 | clean |
| jiff (#32) | 4 | trophy 19 |
| wasmi (#71) | 5 | clean |
| rhai (#73) | 5 | clean (2 out-of-contract observations) |
| geo (#59) | 5 | trophies 25-27 |
| unicode-segmentation (#35) | 5 | dup of open #174 |
| textwrap (#37) | 5 | trophy 21 |
| glob (#29) | 5 | clean |
| ipnet (#27) | 5 | trophies 22, 23 |
| iri-string (#26) | 5 | trophy 24 + hegel engine bug |
| gix (#88/#90) | 6 | trophies 34-35 |
| jj-lib (#89) | 6 | trophy 36 |
| salsa (#78) | 6 | trophy 28 |
| fst (#47) | 6 | clean |
| rstar (#45) | 6 | trophies 29-30 |
| csv (#14) | 6 | trophies 37-40 |
| ttf-parser (#87) | 6 | trophies 31-33 |
| ciborium (#8) | 7 | trophy 43 (intransitive Ord) |
| yaml-rust2 (#10) | 7 | trophies 44-47 |
| quick-xml (#12) | 7 | trophy 48 |
| postcard (#16) | 7 | clean |
| bs58 (#21) | 7 | clean |
| rustybuzz (#83) | 7 | trophy 42 |
| miniz_oxide (#54) | 7 | trophy 41 |
| percent-encoding (#22) | 7 | clean |
| semver (#25) | 8 | clean at 100x |
| euclid (#63) | 8 | trophies 49-51 |
| num-rational (#4) | 8 | trophy 53 + dup #146 |
| bstr (#38) | 8 | trophy 52 |
| lzma-rs (#57) | 8 | clean |
| strsim (#36) | 8 | dup of #79 |
| slotmap (#49) | 9 | clean |
| priority-queue (#50) | 9 | clean |
| hifitime (#33) | 9 | trophies 56-59 |
| byte-unit (#31) | 9 | trophy 54 |
| snap (#55) | 9 | clean |
| weezl (#58) | 9 | trophies 62-63 |
| spade (#61) | 9 | trophies 60-61 |
| robust (#60) | 9 | trophy 55 |
| loro (#93) | 11 | trophies 80-81 |
| rune (#74) | 11 | trophies 82-83 |
| polars (#70) | 12 | clean (polars-row) |
| good_lp (#81) | 12 | clean (modeling layer); trophy 84 in microlp backend |
| prost (#13) | 12 | clean |
| chalk (#79) | 12 | trophies 85-86 (86 low-sev) |
| rumqtt (#97) | 12 | trophies 87-91 |
| openraft (#98) | 12 | trophy 92 |
| lodepng (#102) | 12 | clean (1 sub-bar OOM observation) |
| parry (#62) | 12 | trophy 93 |
| datafusion (#69) | 12 | trophy 94 |
| brotli (#56) | 12 | trophy 95 |
| swash (#84) | 12 | trophy 96 + dup of #130 |
| boa (#72) | 12 | trophy 97 + dup of #4311 |
| native_db (#68) | 11 | trophies 71-72 |
| sled (#66) | 11 | trophy 73 (double-ended iter) |
| diamond-types (#92) | 11 | trophies 74-75 |
| full_moon (#76) | 11 | trophies 76-77 |
| starlark-rust (#75) | 11 | trophies 78-79 |
| rmp (#11) | 10 | trophy 64 |
| asn1 (#19) | 10 | clean |
| iprange (#28) | 10 | clean |
| nucleo (#39) | 10 | trophies 66-68 |
| kiddo (#46) | 10 | trophies 69-70 |
| fixed (#5) | 10 | trophy 65 |

## Excluded — maintainer consent
- bincode (#15): GitHub repo is a deliberate tombstone ("Goodbye Github") — maintainers removed the code specifically objecting to generative-AI use of their project and migrated to sourcehut. Testing it with an AI-driven campaign against their explicit wishes is off the table; excluded out of respect — confirmed by user 2026-07-23. Also fails CANDIDATES.md's actively-maintained criterion (the GitHub project is abandoned as far as the ecosystem's canonical host is concerned).

## Excluded — environmental
- cosmic-text (#85): repo requires git-lfs for font fixtures; unavailable in this environment.
- bloomfilter/probabilistic-collections (#51): canonical repos not locatable via github guesses in this environment; low priority (small crates).
- sanakirja (#67): not hosted on GitHub (Pijul/nest.pijul.com project); clone unavailable in this environment.

## Excluded per CANDIDATES.md's own guidance
- Calibration/off-limits (blog overlap): rust_decimal (#6), heck (#40), im-rs (#52), automerge (#94), fraction.
- Explicitly deprioritized as over-hardened: malachite (#2, built-in generative testing), serde_json (#17), der/base64 (#23), rasn (#18, AFL++), chrono (#34), cranelift (#77), zune-png/jpeg (#101, fuzz corpus), quiche (#99), rustls (#100), semver (#25, WATCH), indexmap (#48, WATCH).

## Pending (batch 11 — the remaining tail: heavy interpreters, databases, CRDTs)
Stage-2/3 large targets not yet run. Candidates for the final batch(es), roughly by tractability:
- **CRDT / collab**: loro (#93), diamond-types (#92)
- **Databases**: sled (#66), sanakirja (#67), native_db/persy (#68)
- **Interpreters / language**: rune (#74), starlark-rust (#75), full_moon (#76), boa (#72), chalk (#79)
- **Analytics**: datafusion (#69), polars (#70) — very heavy builds; may exceed disk/time budget
- **Other**: good_lp (#81), swash (#84), rumqtt (#97), openraft (#98), lodepng-rust (#102), brotli (#56), parry2d/3d (#62), prost/quick-protobuf (#13), gix-config (#90 — sibling gix crates already yielded #34-35; low marginal value)

| unicode-normalization (ext) | 13 | clean (conformance-suite-hardened) |
| borsh (ext) | 13 | clean (canonicalization held) |
| rustfft (ext) | 13 | clean (all planner families, 10x) |
| rkyv (ext) | 13 | trophy 98 (recursion-DoS in safe API) |
| glam (ext) | 13 | trophy 99 (try_normalize contract) |
| i_overlay (ext) | 13 | trophies 100-102 (100 in i_float dep) |
| qoi (ext) | 13 | trophies 103-104 (vs qoi.h reference) |
| kurbo (ext) | 13 | trophies 105-108 |
| rangemap (ext) | 14 | clean (interval-map model held) |
| symphonia (ext) | 14 | clean (symphonia-core) |
| gltf (ext) | 14 | trophy 109 (sibling-decoder disagreement) |
| fancy-regex (ext) | 14 | trophy 110 (capture_names panic) |
| palette (ext) | 14 | trophies 111-112 (color-space) |
| bitvec (ext) | 14 | trophy 113 (Msb0 last_one underflow) |
| simd-json (ext) | 14 | trophies 114-115 (surrogate + DOM recursion) |
| h3o (ext) | 15 | CLEAN (differentially fuzzed vs C H3) |
| similar (ext) | 15 | trophy 123 (Lcs/Hunt empty-input spurious Delete → panic) |
| minijinja (ext) | 15 | EXCLUDED-consent — HUMAN_VS_MACHINE.md AI-gate (mitsuhiko); no bug, local-only |
| xxhash-rust (ext) | 15 | CLEAN (C-reference differential) |
| geographiclib-rs (ext) | 15 | CLEAN |
| pathfinding (ext) | 15 | CLEAN (cross-impl + independent oracles) |
| geohash (ext) | 15 | trophies 116-117 (pole-flip encode; decode("") overflow) |
| instant-distance (ext) | 14 | trophy 124 (ml layer-sizing panic+OOM); deadlock = dup #49 |
| reed-solomon-erasure (ext) | 16 | trophy 126 (simd empty-slice panic; repo dormant) |
| num-bigint (ext) | 16 | trophy 125 (modinv \|m\|==1 contract) |
| statrs (ext) | 16 | trophies 129-130 (gamma pdf NaN, beta try_ panic) + 14 grouped obs |
| aho-corasick (ext) | 16 | trophy 128 (overlapping dup) + 1 below-bar obs |
| data-encoding (ext) | 16 | CLEAN |
| earcutr (ext) | 16 | trophy 127 (T::max_value contour panic) |
| indexmap (ext) | 16 | CLEAN |
| ulid (ext) | 16 | DUPLICATE of open #101 (8th) |
| evalexpr (ext) | 15 | trophies 118-122 (substring/shift panics, i64::MIN, len, recursion) |

## Evaluated-count summary (as of batch 10)
- **57 candidate crates evaluated** across 10 batches. 70 confirmed trophies (67 novel + 3 grouped-residual) across 33 crates; 27 crates clean-or-dup (rigorous negatives).
- Convergence: skill methodology stable since batch 7; batches 8-10 produced only import-polish + the validation/rejection catalogue pattern + operational notes. Remaining feedback is hegel-rust feature requests, not skill gaps.
