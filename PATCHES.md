# Patches: hegel-rust property-based tests per crate

Each `patches/<crate>.patch` is the diff that adds the hegel property-based tests (and the `hegeltest` dev-dependency) to that crate, generated from the crate's pristine upstream checkout at the **base commit** recorded below (i.e. the parent of the "Add hegel property-based tests" commit). `Cargo.lock` changes are excluded.

## Applying a patch

```sh
git clone <upstream repo> && cd <crate>
git checkout <base commit>
git apply /path/to/patches/<crate>.patch   # or: patch -p1 < ...
cargo test
```

The base commit is the exact upstream revision each patch was written and verified against; the crate's `Cargo.toml` in the patch adds `hegeltest` as a dev-dependency (some crates need `--ignore-rust-version` due to hegeltest's MSRV).

Crates whose maintainers have opted out of AI contributions are intentionally omitted, as are crates still under active testing at the time of this snapshot.

## Index

| Crate | Upstream repo | Base commit | Base date | Base commit subject |
|---|---|---|---|---|
| `aho-corasick` | https://github.com/BurntSushi/aho-corasick | `c82178696b9d` | 2026-04-21 | build(deps): bump actions/checkout in the actions group (#171) |
| `asn1` | https://github.com/alex/rust-asn1 | `851dc2e18e90` | 2026-07-20 | Upgrade to syn 3 (#622) |
| `bech32` | https://github.com/rust-bitcoin/rust-bech32 | `162cff91760a` | 2026-07-22 | Merge rust-bitcoin/rust-bech32#283: Automated daily update to rustc... |
| `bigdecimal` | https://github.com/akubera/bigdecimal-rs | `7f0243e73702` | 2025-12-27 | Begin v0.4.11 development |
| `bitcode` | https://github.com/SoftbearStudios/bitcode | `7ae88076943b` | 2026-06-27 | Use core::error::Error instead of std (and remove some nightly poly... |
| `bitvec` | https://github.com/ferrilab/bitvec | `5fb855073acc` | 2023-04-12 | Merge pull request #201 from connorskees/feat/add-track-caller |
| `boa` | https://github.com/boa-dev/boa | `5718cc81111c` | 2026-07-22 | Add console.exception() as alias for console.error() (#5425) |
| `borsh` | https://github.com/near/borsh-rs | `7fc21fe52d39` | 2026-07-16 | chore: release v1.8.0 (#375) |
| `brotli` | https://github.com/dropbox/rust-brotli | `9651aa3ebfd2` | 2026-06-14 | Fix version bump |
| `bs58` | https://github.com/Nullus157/bs58-rs | `e4a65a9ec641` | 2024-03-19 | Merge pull request #118 from Nemo157/nightly-update |
| `bson` | https://github.com/mongodb/bson-rust | `27166baaee68` | 2026-07-21 | minor: add .zed to gitignore (#679) |
| `bstr` | https://github.com/BurntSushi/bstr | `08a77375dfa8` | 2026-07-15 | doc: add AI Policy (#232) |
| `byte-unit` | https://github.com/magiclen/byte-unit | `8acd4c0cd85e` | 2026-06-28 | update docs cfg |
| `calamine` | https://github.com/tafia/calamine | `c53aff3d81a7` | 2026-07-16 | support OOXML format (#681) |
| `capnp` | https://github.com/dwrensha/capnproto-rust | `d1616946b6a5` | 2026-07-23 | prepare for capnp-rpc-v0.26.2 release |
| `cbor4ii` | https://github.com/quininer/cbor4ii | `7a496cb5186d` | 2025-11-30 | release 1.2.2 |
| `chalk` | https://github.com/rust-lang/chalk | `627409a4735b` | 2026-02-08 | Merge pull request #833 from Noratrieb/patch-1 |
| `ciborium` | https://github.com/enarx/ciborium | `00279e48e75e` | 2026-06-18 | build(deps): bump actions/checkout from 6 to 7 |
| `crop` | https://github.com/noib3/crop | `d0234ce772eb` | 2026-03-02 | Fix `offset` in `UnitsBackward::remainder()` |
| `csv` | https://github.com/BurntSushi/rust-csv | `4a3997e91d66` | 2025-10-17 | 1.4.0 |
| `data-encoding` | https://github.com/ia0/data-encoding | `a57da53784fd` | 2026-07-19 | Remove deprecated authors field from Cargo.toml (#158) |
| `datafusion` | https://github.com/apache/datafusion | `1c8295c992c5` | 2026-07-23 | feat: Support Union type in approx_distinct (#23714) |
| `diamond-types` | https://github.com/josephg/diamond-types | `ad48b9cced1d` | 2026-05-29 | More cleanups |
| `dyon` | https://github.com/pistondevelopers/dyon.git | `3fb34a313a37` | 2025-12-23 | Merge pull request #797 from bvssvni/master |
| `earcutr` | https://github.com/frewsxcv/earcutr/ | `a201db6b6ec1` | 2026-05-04 | Update README with deprecation and new repository link |
| `etherparse` | https://github.com/JulianSchmid/etherparse | `70f72bee542a` | 2026-07-21 | Merge pull request #162 from JulianSchmid/migrate-coverage-to-gist-... |
| `euclid` | https://github.com/servo/euclid | `60f2bd96deec` | 2026-03-17 | Avoid NaNs in Rotation::get_angle (#552) |
| `evalexpr` | https://github.com/ISibboI/evalexpr.git | `45e9634dab34` | 2025-11-26 | Release (#195) |
| `fancy-regex` | https://github.com/fancy-regex/fancy-regex | `d00f0f7a2382` | 2026-07-05 | Merge pull request #263 from fancy-regex/anchored_search |
| `fixed` | https://gitlab.com/tspiteri/fixed | `7afb5bf760e7` | 2026-03-20 | version 1.31.0 |
| `fjall` | https://github.com/fjall-rs/fjall | `6debe706dbc5` | 2026-07-18 | 3.1.8 |
| `fst` | https://github.com/BurntSushi/fst | `5907b4739793` | 2024-09-25 | github: add FUNDING |
| `full_moon` | https://github.com/Kampfkarren/full-moon | `47d4bf94104c` | 2026-04-15 | 2.2.0 |
| `geo` | https://github.com/georust/geo | `6b2127d9ad99` | 2026-07-22 | Use total_cmp in sweep line interval ordering (#1554) |
| `geographiclib-rs` | https://github.com/georust/geographiclib-rs | `c5e906d94a46` | 2026-02-17 | return result rather than panic |
| `geohash` | https://github.com/georust/geohash.rs | `50a4b2a35ee0` | 2026-06-14 | Add encode_iter variant of encode to enable avoiding allocations (#62) |
| `gimli` | https://github.com/gimli-rs/gimli | `843c38e886f5` | 2026-07-06 | read/cfi: validate eh_frame_hdr fde_count against table length (#897) |
| `gitoxide` | https://github.com/GitoxideLabs/gitoxide | `2315ede714da` | 2026-07-22 | Merge pull request #2737 from GitoxideLabs/encoding-fallback-pony |
| `glam` | https://github.com/bitshifter/glam-rs | `6feed7d50ee7` | 2026-07-23 | Consolodate some common test code into macros where possible (#756) |
| `glob` | https://github.com/rust-lang/glob | `cfa2a58f2e44` | 2026-07-21 | chore: release v0.3.4 |
| `gltf` | https://github.com/gltf-rs/gltf | `50d65229477f` | 2026-05-11 | Merge pull request #471 from alteous/fix-panics |
| `gluon` | https://github.com/gluon-lang/gluon | `418c6b7de22b` | 2026-07-10 | Merge pull request #978 from Marwes/more |
| `goblin` | https://github.com/m4b/goblin | `dca2e753b2ab` | 2026-06-13 | Fix Actions badge (#540) |
| `good_lp` | https://github.com/rust-or/good_lp | `e4a73e22ee00` | 2026-07-18 | bump microlp add mip gap and initial solution (#129) |
| `h3o` | https://github.com/HydroniumLabs/h3o | `287e4b26b5b5` | 2026-06-08 | little cleanup |
| `handlebars` | https://github.com/sunng87/handlebars-rust | `2802ada08ad8` | 2026-07-16 | chore(deps-dev): bump websocket-driver in /playground/www (#772) |
| `hcl-rs` | https://github.com/martinohmann/hcl-rs | `2f0b1f87fbb4` | 2026-07-02 | chore(deps): update dtolnay/rust-toolchain digest to 4be7066 (#546) |
| `heapless` | https://github.com/rust-embedded/heapless | `fbe9aeb4db17` | 2026-07-19 | Merge pull request #652 from sgued/rem-perf |
| `hickory` | https://github.com/hickory-dns/hickory-dns | `1b78772fcad0` | 2026-07-22 | Include CNAME records when calculating minimum TTL for caching purp... |
| `hifitime` | https://github.com/nyx-space/hifitime | `b2ccd8f1163f` | 2026-07-21 | Merge pull request #493 from nyx-space/derive-partial-eq-duration-1... |
| `httparse` | https://github.com/seanmonstar/httparse | `a0fa552e4e0f` | 2026-06-30 | refactor: share invalid header handling (#221) |
| `humantime` | https://github.com/chronotope/humantime | `76c8929b4cc2` | 2026-07-13 | Apply suggestions from Clippy 1.97 |
| `i_overlay` | https://github.com/iShape-Rust/iOverlay | `eeb4a9acfd1a` | 2026-07-05 | update contribution rules |
| `indexmap` | https://github.com/indexmap-rs/indexmap | `571943c5b3ec` | 2026-07-10 | Merge pull request #445 from maxtaran2010/fix/typos |
| `instant-distance` | https://github.com/InstantDomain/instant-distance | `13ea89ac1ca0` | 2026-07-21 | Bump actions/setup-python from 6 to 7 |
| `ipnet` | https://github.com/krisprice/ipnet | `65c04c355668` | 2026-03-03 | Update version number. |
| `iprange` | https://github.com/sticnarf/iprange-rs | `0f36df090902` | 2026-04-20 | Merge pull request #31 from pronebird/rust-edition-2024 |
| `iri-string` | https://github.com/lo48576/iri-string | `07d982ed76ec` | 2026-07-22 | doc: fix harmless typo |
| `jiff` | https://github.com/BurntSushi/jiff | `7311a6ac67cf` | 2026-07-19 | 0.2.34 |
| `jj` | https://github.com/jj-vcs/jj | `f296bc36b18d` | 2026-07-21 | cli: show workspace roots in workspace list |
| `jotdown` | https://github.com/hellux/jotdown | `56d6d1b3d707` | 2026-07-06 | .gitignore: add afl output dir |
| `json5` | https://github.com/callum-oakley/json5-rs | `6905ad2ea7b0` | 2026-02-07 | expose char |
| `ketos` | https://github.com/murarth/ketos | `011287590ebe` | 2020-01-17 | Merge pull request #66 from murarth/github-ci |
| `kiddo` | https://github.com/sdd/kiddo | `39cbbaf99876` | 2026-07-22 | ci: cap benchmark trees at 2^25 |
| `koto` | https://github.com/koto-lang/koto | `4b433e7a7ce1` | 2026-07-05 | Merge pull request #552 from koto-lang/koto-derive-improvements |
| `kurbo` | https://github.com/linebender/kurbo | `ca273499e3e4` | 2026-07-23 | ci: Update to stable Rust 1.97.1, typos 1.48.0 (#596) |
| `liquid` | https://github.com/cobalt-org/liquid-rust | `cd1e5ac838ad` | 2026-07-10 | chore(deps): Update Rust Stable to v1.97 (#625) |
| `lodepng` | https://github.com/kornelski/lodepng-rust.git | `cc5d7c6feb89` | 2026-02-16 | Drop cf-zlib |
| `loro` | https://github.com/loro-dev/loro/ | `6844fc7c9d8f` | 2026-07-22 | chore: version packages (#1042) |
| `lru` | https://github.com/jeromefroe/lru-rs.git | `c6620d1165dd` | 2026-07-09 | Merge pull request #237 from jeromefroe/jerome/prepare-0-18-1-release |
| `lz4_flex` | https://github.com/pseitz/lz4_flex | `f4f624772f13` | 2026-07-14 | stricter lints |
| `lzma-rs` | https://github.com/gendx/lzma-rs | `1f14478def43` | 2024-05-06 | Remove CARGO_UNSTABLE_SPARSE_REGISTRY from GitHub actions. |
| `miniz_oxide` | https://github.com/Frommi/miniz_oxide | `fed739a8c7fe` | 2026-07-20 | try fuzz again |
| `nalgebra` | https://github.com/dimforge/nalgebra | `3320ecca21dc` | 2026-06-30 | fix: Cholesky::new returns None non-positive-definite complex matri... |
| `native_db` | https://github.com/vincent-herlemont/native_db | `b9554fdabbdd` | 2025-10-10 | chore(deps): update rust crate cc to 1.2.41 |
| `ndarray` | https://github.com/rust-ndarray/ndarray | `bd3ade99c1f6` | 2026-06-19 | Make ArrayViewMut::into_view public to allow lifetime preservation ... |
| `nucleo` | https://github.com/helix-editor/nucleo | `8c16d47cdfa9` | 2026-06-23 | doc: Fix a typo |
| `num-bigint` | https://github.com/rust-num/num-bigint | `9ec740f8c162` | 2026-07-07 | Merge pull request #351 from cuviper/rename-head |
| `num-rational` | https://github.com/rust-num/num-rational | `cf95d6719c58` | 2026-07-07 | Merge pull request #154 from cuviper/modules |
| `openraft` | https://github.com/databendlabs/openraft | `0d15d99844e8` | 2026-07-23 | feat: add heartbeat_min_interval to suppress redundant heartbeats |
| `palette` | https://github.com/Ogeon/palette | `9aa1ac21a7da` | 2026-05-15 | Merge pull request #469 from Ogeon/phf_0.13 |
| `parry` | https://github.com/dimforge/parry | `8436f7c21875` | 2026-07-04 | Release v0.29.0 (#427) |
| `pathfinding` | https://github.com/evenfurther/pathfinding | `16ce0bc5d60b` | 2026-07-21 | chore(deps): update actions/checkout action to v7 |
| `percent` | https://github.com/servo/rust-url | `25137be1fc1d` | 2026-07-08 | fix percent-encode of caret in path (#1140) (#1141) |
| `petgraph` | https://github.com/petgraph/petgraph | `ed714652ab45` | 2026-03-08 | chore: Bump hashbrown to ^0.16 (#967) |
| `plist` | https://github.com/ebarnard/rust-plist/ | `2881e175b61f` | 2026-07-04 | Release v1.10.0 |
| `polars` | https://github.com/pola-rs/polars | `1f6362635a59` | 2026-07-23 | fix: Do not CSE non-column height expr on streaming engine (#28480) |
| `postcard` | https://github.com/jamesmunns/postcard | `118d274cf46e` | 2026-07-20 | Merge pull request #300 from sugar700/enum-map-v2_0 |
| `priority-queue` | https://github.com/garro95/priority-queue | `95499ebb38f2` | 2025-10-15 | Prepare version |
| `prost` | https://github.com/tokio-rs/prost | `aed74ad0e844` | 2026-07-05 | fix: Prevent panic for service generator in empty module (#1442) |
| `pulldown-cmark` | https://github.com/raphlinus/pulldown-cmark | `68afb08c9014` | 2026-07-08 | Merge pull request #1111 from teddytennant/fix-wikilink-overflow |
| `qoi` | https://github.com/aldanor/qoi-rust | `81c14c4b637c` | 2025-07-28 | Merge pull request #22 from aldanor/fuzz-fix |
| `quick-protobuf` | https://github.com/tafia/quick-protobuf | `54e7d6c5d981` | 2024-02-14 | Merge pull request #259 from ghpr-asia/mr-config-msrv-re |
| `quick-xml` | https://github.com/tafia/quick-xml | `56ae43f82792` | 2026-07-20 | Bound NamespaceResolver nesting depth |
| `quinn` | https://github.com/quinn-rs/quinn | `fec2f8960df4` | 2026-07-20 | build(deps): bump rustls from 0.23.41 to 0.23.42 |
| `radix_trie` | https://github.com/michaelsproul/rust_radix_trie | `a89f789e1267` | 2025-09-16 | Release v0.3.0 (#79) |
| `rangemap` | https://github.com/jeffparsons/rangemap | `414e9c7c10af` | 2025-12-19 | Merge pull request #117 from jeffparsons/prepare_v1.7.1 |
| `rasn` | https://github.com/librasn/rasn.git | `0e45728195d9` | 2026-05-04 | chore: fmt |
| `redb` | https://github.com/cberner/redb | `fe0141159c73` | 2026-07-17 | Sandbox just bench target |
| `reed-solomon-erasure` | https://github.com/darrenldl/reed-solomon-erasure | `ac2b561e406f` | 2022-11-11 | Change recommended benchmarking to `cargo bench` |
| `rhai` | https://github.com/rhaiscript/rhai | `950b724b8f1d` | 2026-07-18 | Merge pull request #1106 from yuvalrakavy/fix-compact-script-operat... |
| `rkyv` | https://github.com/rkyv/rkyv | `46e143d6e4c8` | 2026-07-02 | Release 0.8.17 |
| `rmp` | https://github.com/3Hren/msgpack-rust | `cf880019f72e` | 2025-12-23 | Drop byteorder dep |
| `roaring` | https://github.com/RoaringBitmap/roaring-rs | `83caaca2ec5e` | 2026-04-24 | Merge pull request #353 from RoaringBitmap/fix-fuzz-ab533c242f8db0b... |
| `robust` | https://github.com/georust/robust | `654f34cb8cdb` | 2025-05-09 | Prepare for 1.2.0 release |
| `ron` | https://github.com/ron-rs/ron | `31529b8b8d8c` | 2026-07-16 | Fix quadratic escaped string parsing in escaped_byte_buf (#610) |
| `roxmltree` | https://github.com/RazrFalcon/roxmltree | `e8a27a70867b` | 2026-05-23 | Fix a typo in and reword for clarity the docstring for `EntityResol... |
| `rpds` | https://github.com/orium/rpds | `d7c1205c81f1` | 2026-07-19 | Fix new clippy warnings. |
| `rstar` | https://github.com/georust/rstar | `05e6d58c5e03` | 2026-06-22 | Bump actions/checkout from 6 to 7 (#234) |
| `rumqtt` | https://github.com/bytebeamio/rumqtt | `e886a788935d` | 2026-05-01 | feat: add support for binding outgoing TCP connections to a specifi... |
| `rune` | https://github.com/rune-rs/rune | `20b26957f18e` | 2026-07-20 | Make indentation configurable and honor LSP formatting options |
| `rustfft` | https://github.com/ejmahler/RustFFT | `4758ab0dd6f2` | 2025-09-17 | Release v6.4.1 (#165) |
| `rustybuzz` | https://github.com/harfbuzz/rustybuzz | `51d99b83ae78` | 2025-06-09 | [buffer] Fix buffer size enlargement (harfruzz PR #62) |
| `ruzstd` | https://github.com/KillingSpark/zstd-rs | `e7cc3b92895f` | 2026-07-22 | Remove `compiler-builtins` from `rustc-dep-of-std` dependencies (#113) |
| `salsa` | https://github.com/salsa-rs/salsa | `dcbcc7082c3b` | 2026-07-22 | chore: release v0.28.1 (#1248) |
| `semver` | https://github.com/dtolnay/semver | `280ebcb6edac` | 2026-06-23 | Update actions/upload-artifact@v6 -> v7 |
| `simd-json` | https://github.com/simd-lite/simd-json | `c8cece05a69a` | 2026-07-14 | Add approx integer parsing error-path test coverage (#466) |
| `similar` | https://github.com/mitsuhiko/similar | `0210f53830cc` | 2026-05-24 | chore(release): prepare 3.1.1 |
| `sled` | https://github.com/spacejam/sled | `e449d17111f4` | 2026-04-04 | Add benchmark for memory and throughput of a fanout=3 data set for ... |
| `slotmap` | https://github.com/orlp/slotmap | `0d130ed5bbd6` | 2026-05-09 | Add MSRV-compatible lockfiles (#151) |
| `snap` | https://github.com/BurntSushi/rust-snappy | `29fcab53647b` | 2026-07-15 | 1.1.2 |
| `spade` | https://github.com/Stoeoef/spade | `c8befc96bbbc` | 2026-03-24 | chore: Release |
| `speedate` | https://github.com/pydantic/speedate/ | `6fafc2c60b5c` | 2026-04-15 | Bump codecov/codecov-action from 5 to 6 in the actions group (#99) |
| `sqlparser` | https://github.com/apache/datafusion-sqlparser-rs | `bef86dd6826e` | 2026-07-21 | Snowflake: parse CREATE WAREHOUSE (#2388) |
| `starlark` | https://github.com/facebook/starlark-rust | `dd23c83b49ff` | 2026-07-22 | Preserve `FrozenHeapName` across paging |
| `statrs` | https://github.com/statrs-dev/statrs | `102945824c83` | 2026-07-20 | chore: update MSRV lockfile |
| `steel` | https://github.com/mattwparas/steel | `3a418c9ea586` | 2026-07-18 | clear out the stack that isn't used on the spawned thread (#674) |
| `strsim` | https://github.com/rapidfuzz/strsim-rs | `dacc84c0dc61` | 2025-11-27 | Fix clippy warnings |
| `swash` | https://github.com/dfrg/swash | `7773843df0d6` | 2026-07-17 | Bump version number to 0.2.10 (#132) |
| `symphonia` | https://github.com/pdeljanov/Symphonia | `5f26f020b3a1` | 2026-07-23 | core (io): Clamp scan_bytes_aligned_ref to scan_len to prevent over... |
| `taffy` | https://github.com/DioxusLabs/taffy | `bb351fcc056c` | 2026-07-15 | Prepare for v0.12.2 release (#979) |
| `tera` | https://github.com/Keats/tera | `15e0c6e6f1ab` | 2026-07-23 | Fix typo |
| `textwrap` | https://github.com/mgeisler/textwrap | `e29daecac529` | 2026-06-28 | Merge pull request #619 from mgeisler/rename-master-to-main |
| `tiff` | https://github.com/image-rs/image-tiff | `f3f9ff1244e5` | 2026-07-20 | Merge pull request #398 from Shnatsel/safe-rust-zstd-2 |
| `toml` | https://github.com/toml-rs/toml | `a0c14f4b6a46` | 2026-07-16 | chore(deps): Update Prek to v0.4.10 (#1190) |
| `ttf-parser` | https://github.com/harfbuzz/ttf-parser | `6e75b3c539ef` | 2025-11-22 | chore: Deduplicate vhea parsing (#204) |
| `ulid` | https://github.com/dylanhart/ulid-rs | `6018cb8d158a` | 2026-07-15 | Add changelog |
| `unicode-normalization` | https://github.com/unicode-rs/unicode-normalization | `576ae0b1407d` | 2025-11-02 | Merge pull request #116 from musicinmybrain/license |
| `unicode-segmentation` | https://github.com/unicode-rs/unicode-segmentation | `66a032fd8d66` | 2026-06-01 | Publish 1.13.3 |
| `uom` | https://github.com/iliekturtles/uom | `a465bcc2b3bf` | 2026-04-04 | Merge pull request #543 from SombkeMaximilian/gyromagnetic_ratio |
| `url` | https://github.com/servo/rust-url | `25137be1fc1d` | 2026-07-08 | fix percent-encode of caret in path (#1140) (#1141) |
| `varisat` | https://github.com/jix/varisat | `33e876937c5d` | 2022-11-02 | Merge pull request #165 from maugier/cnfformula-clone |
| `vte` | https://github.com/alacritty/vte | `abeae765dd54` | 2026-02-28 | Add rustdoc attribute to ansi module |
| `wasmi` | https://github.com/wasmi-labs/wasmi | `bd3732cea636` | 2026-07-22 | Move `ArenaKey` and impls into its own submodule (#1989) |
| `weezl` | https://github.com/image-rs/weezl | `606f9c79b054` | 2026-05-15 | Merge pull request #82 from image-rs/release-0.2.1 |
| `wkt` | https://github.com/georust/wkt | `85088d9279e5` | 2026-01-01 | Fix doc build (#151) |
| `x509-parser` | https://github.com/rusticata/x509-parser.git | `303b80f44685` | 2026-07-23 | fix(validate): reject unsupported critical extensions per RFC 5280 |
| `xxhash-rust` | https://github.com/DoumanAsh/xxhash-rust | `f93abc7ce036` | 2026-07-21 | 0.8.18 |
| `yaml-rust2` | https://github.com/Ethiraric/yaml-rust2 | `9f39918876eb` | 2025-12-16 | tests: fix clippy warnings |
| `ycrdt` | https://github.com/y-crdt/y-crdt/ | `67b0513fe6cf` | 2026-07-13 | Merge pull request #638 from Horusiath/release-v0.27.3 |
| `zip` | https://github.com/zip-rs/zip2 | `1058f8062102` | 2026-07-23 | ci(deps): bump step-security/harden-runner from 2.19.4 to 2.20.0 (#... |
