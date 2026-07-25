# 2×2 SVD is ~7 digits less accurate through `DMatrix` than through `Matrix2`

```rust
use nalgebra::{Matrix2, DMatrix};

let vals = [2.27, -2.5e-6, 0.0, 9.008];
let a = Matrix2::new(vals[0], vals[1], vals[2], vals[3]);
let ad = DMatrix::from_row_slice(2, 2, &vals);

let e_static = (a.svd(true, true).recompose().unwrap() - a).norm();
let e_dynamic = (ad.clone().svd(true, true).recompose().unwrap() - ad).norm();
println!("static  {e_static:.3e}");
println!("dynamic {e_dynamic:.3e}");
```

prints

```
static  2.051e-15
dynamic 8.966e-9
```

Reconstructing the same matrix from its own SVD (`U * Σ * Vᵀ`) is accurate to ~2e-15 through the static `Matrix2` path but only ~9e-9 through the dynamic `DMatrix` path — about seven digits worse for an ordinary, well-conditioned matrix.

Tested on nalgebra 0.35.0.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
