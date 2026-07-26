# `query::distance` between two cuboids is asymmetric and overestimates the true separation

```rust
use parry2d::math::{Pose, Vector};
use parry2d::query;
use parry2d::query::closest_points::ClosestPoints;
use parry2d::shape::Cuboid;

fn main() {
    let c1 = Cuboid::new(Vector::new(1.0, 1.0));
    let c2 = Cuboid::new(Vector::new(1.0, 2.0));
    let p1 = Pose::identity();
    let p2 = Pose::new(Vector::new(-5.573167, 0.0), 0.0);

    let d12 = query::distance(&p1, &c1, &p2, &c2).unwrap();
    let d21 = query::distance(&p2, &c2, &p1, &c1).unwrap();

    println!("distance(c1, c2) = {}", d12);
    println!("distance(c2, c1) = {}", d21);

    if let ClosestPoints::WithinMargin(a, b) =
        query::closest_points(&p1, &c1, &p2, &c2, f32::MAX).unwrap()
    {
        println!("closest_points distance = {}", (a - b).length());
    }
}
```

Output:

```
distance(c1, c2) = 3.710461
distance(c2, c1) = 3.5731668
closest_points distance = 3.5731668
```

`query::distance` gives a different answer depending on argument order, and the `c1, c2` order overestimates the true gap between the two cuboids (5.573167 − 1 − 1 = 3.5731668, which is what `closest_points` and the `c2, c1` order both agree on).

Tested on `parry2d` 0.29.0.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
