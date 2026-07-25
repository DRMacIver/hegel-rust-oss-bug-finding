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

### Dropped during triage (not reported)
- **ron** (`Value` f32 round-trip drift) — already open as [ron-rs/ron#613](https://github.com/ron-rs/ron/issues/613); also opt-in-fixable via `number_suffixes`.
- **geohash** (north pole encodes as south pole) — already has an open fixing PR (#63).
- **simd-json** (lone surrogate → NUL) — area already covered by closed #228 and advisory #457.
