# `Line`, `QuadBez`, and `CubicBez` `eval` can return NaN at t=0/t=1 instead of the curve's own endpoint

```rust
use kurbo::{CubicBez, Line, ParamCurve, Point, QuadBez};

fn main() {
    let l = Line::new(Point::new(1e308, 0.0), Point::new(-1e308, 0.0));
    println!("Line eval(0) = {:?}", l.eval(0.0));
    println!("Line eval(1) = {:?}", l.eval(1.0));

    let big = 1e308;
    let q = QuadBez::new(
        Point::new(0.0, 0.0),
        Point::new(big, big),
        Point::new(0.0, 0.0),
    );
    println!("QuadBez eval(0) = {:?}", q.eval(0.0));
    println!("QuadBez eval(1) = {:?}", q.eval(1.0));

    let c = CubicBez::new(
        Point::new(0.0, 0.0),
        Point::new(big, big),
        Point::new(0.0, 0.0),
        Point::new(0.0, 0.0),
    );
    println!("CubicBez eval(0) = {:?}", c.eval(0.0));
    println!("CubicBez eval(1) = {:?}", c.eval(1.0));
}
```

Output:

```
Line eval(0) = (NaN, 0.0)
Line eval(1) = (-inf, 0.0)
QuadBez eval(0) = (NaN, NaN)
QuadBez eval(1) = (0.0, 0.0)
CubicBez eval(0) = (NaN, NaN)
CubicBez eval(1) = (0.0, 0.0)
```

All the control points passed in above are finite `Point` values, and `eval(0.0)` / `eval(1.0)` should just return the curve's own start / end point, but several of them come back as `(NaN, NaN)` or with an infinite coordinate instead of the finite point that was actually passed in.

Tested on kurbo 0.13.1.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
