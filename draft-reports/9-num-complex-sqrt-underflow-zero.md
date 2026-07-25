# `sqrt` of a tiny nonzero imaginary number returns exactly zero

```rust
use num_complex::Complex;

let z = Complex::new(0.0_f64, 5e-324);
println!("{:?}", z.sqrt());
```

prints

```
Complex { re: 0.0, im: 0.0 }
```

`z` is nonzero (`5e-324` is the smallest positive subnormal `f64`), but `z.sqrt()` is exactly `0`. The true square root is about `1.6e-162 * (1 + i)`, a perfectly normal value; squaring the returned `0` cannot recover `z`.

Tested on num-complex 0.4.6.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
