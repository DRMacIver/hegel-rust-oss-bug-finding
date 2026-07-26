# `Gamma::pdf` returns `NaN` for ordinary, finite, in-range parameters

```rust
use statrs::distribution::{Continuous, Gamma};

fn main() {
    let g = Gamma::new(80.0, 1e-5).unwrap();
    let x = 8e6_f64;
    println!("pdf({}) = {}", x, g.pdf(x));
    println!("ln_pdf({}) = {}", x, g.ln_pdf(x));
    println!("exp(ln_pdf) = {}", g.ln_pdf(x).exp());
}
```

```
pdf(8000000) = NaN
ln_pdf(8000000) = -14.623918976753487
exp(ln_pdf) = 0.0000004455666577034725
```

`shape = 80`, `rate = 1e-5`, and `x = 8e6` (the mean of the distribution) are all finite and within the domain `Gamma::new` accepts. `pdf` returns `NaN`, while `ln_pdf` on the same distribution and the same `x` gives a finite value whose exponential is a normal, non-zero density.

Tested on statrs 0.19.0, with both a debug and a release build.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
