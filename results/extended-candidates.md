
## Batch 14 — new-domain probe (testing the convergence hypothesis)
Resumed per goal: the criterion "no further skill improvements found" is only met when a genuinely-new-domain batch yields no new methodology. Batch 13 showed new domains revive the lesson stream, so this batch deliberately spans domains untouched by batches 1-13.

| Crate | domain (new?) | last | fuzz/ | PBT | note |
|---|---|---|---|---|---|
| simd-json | SIMD JSON | 2026-07-14 | yes | yes | SIMD-vs-scalar + vs serde_json differential |
| symphonia | audio decode | 2026-07-23 | no | no | NEW domain; untrusted-input decoders |
| gltf | 3D scene format | 2026-05-11 | no | no | NEW domain; roundtrip + untrusted |
| palette | color-space math | 2026-05-15 | no | no | NEW domain; conversion roundtrips + precision |
| bitvec | bit container | 2023-04-12 | no | no | NEW domain; model vs Vec<bool>; stale (may not build) |
| fancy-regex | backtracking regex | 2026-07-05 | yes | yes | NEW domain; vs regex crate oracle |
| rangemap | interval map | 2025-12-19 | yes | yes | NEW domain; interval algebra model |
| instant-distance | approximate NN | 2026-07-21 | no | no | NEW domain (kiddo was exact); ANN-vs-brute-force |
