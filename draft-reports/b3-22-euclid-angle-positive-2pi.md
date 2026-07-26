# `Angle::positive()` can return `2*PI` and `Angle::signed()` can return `-PI`

```rust
use euclid::Angle;

fn main() {
    let pi = std::f64::consts::PI;
    let two_pi = 2.0 * pi;

    let a: Angle<f64> = Angle::radians(-2.220446049250313e-16); // -f64::EPSILON
    println!("a.positive().radians = {:.20}", a.positive().radians);
    println!("2*PI                 = {:.20}", two_pi);
    println!("a.positive() == 2*PI : {}", a.positive().radians == two_pi);

    let b: Angle<f64> = Angle::radians(pi.next_up());
    println!("b.signed().radians   = {:.20}", b.signed().radians);
    println!("-PI                  = {:.20}", -pi);
    println!("b.signed() == -PI    : {}", b.signed().radians == -pi);
}
```

Output:

```
a.positive().radians = 6.28318530717958623200
2*PI                 = 6.28318530717958623200
a.positive() == 2*PI : true
b.signed().radians   = -3.14159265358979311600
-PI                  = -3.14159265358979311600
b.signed() == -PI    : true
```

`positive()` is documented as returning the angle in `[0, 2*PI)` and `signed()` in `(-PI, PI]`. Here `positive()` returns exactly `2*PI` for an input just below zero, and `signed()` returns exactly `-PI` for an input just above `PI` — both outside their documented ranges.

Tested on euclid 0.22.14 (debug and release).

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
