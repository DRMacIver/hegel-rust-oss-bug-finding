# `into_positive_degrees` can return values outside its documented `[0, 360)` range

```rust
use palette::RgbHue;

fn main() {
    let h1 = RgbHue::from_degrees(-1e-30_f64);
    println!("{}", h1.into_positive_degrees());

    let h2 = RgbHue::from_degrees(-5e-324_f64);
    println!("{}", h2.into_positive_degrees());
}
```

Output:

```
360
-0.000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005
```

`into_positive_degrees` is documented as returning the hue "in the range `[0, 360)`". The first call returns exactly `360.0`, which is outside that range on the upper end. The second call returns a small negative number, which is outside the range on the lower end (it is not `-0.0`, but a negative subnormal).

Tested on palette 0.7.6, on both debug and release builds.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
