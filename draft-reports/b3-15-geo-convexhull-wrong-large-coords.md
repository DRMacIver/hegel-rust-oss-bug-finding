# `ConvexHull` returns a non-convex, self-intersecting ring for large-magnitude coordinates

```rust
use geo::{ConvexHull, MultiPoint, Point};

fn main() {
    let points = vec![
        Point::new(0.0, 0.0),
        Point::new(-1.0, 9150170671525436.0),
        Point::new(63.0, 0.0),
        Point::new(0.0, 1.0),
    ];
    let hull = MultiPoint::new(points.clone()).convex_hull();
    let ring: Vec<_> = hull.exterior().coords().collect();
    println!("hull ring: {:?}", ring);

    for w in ring.windows(2) {
        let (a, b) = (w[0], w[1]);
        for p in &points {
            let sign = robust::orient2d(
                robust::Coord { x: a.x, y: a.y },
                robust::Coord { x: b.x, y: b.y },
                robust::Coord { x: p.x(), y: p.y() },
            );
            if sign < 0.0 {
                println!(
                    "input point {:?} is strictly to the RIGHT of hull edge {:?} -> {:?} (orient2d = {})",
                    p, a, b, sign
                );
            }
        }
    }
}
```

This also needs `robust = "1.2.0"` as a dependency. Output:

```
hull ring: [COORD(0.0 0.0), COORD(0.0 1.0), COORD(0.0 0.0), COORD(63.0 0.0), COORD(-1.0 9150170671525436.0), COORD(0.0 0.0)]
input point POINT(63.0 0.0) is strictly to the RIGHT of hull edge COORD(0.0 0.0) -> COORD(0.0 1.0) (orient2d = -63)
input point POINT(-1.0 9150170671525436.0) is strictly to the RIGHT of hull edge COORD(0.0 1.0) -> COORD(0.0 0.0) (orient2d = -1)
```

The returned ring visits `(0,0)` twice and is self-intersecting rather than convex: two of the four input points end up strictly outside edges of the "hull" that is supposed to contain them, which the exact `orient2d` predicate from the `robust` crate (already a dependency of `geo`) confirms. Same result in a release build.

Tested on `geo` 0.33.1.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
