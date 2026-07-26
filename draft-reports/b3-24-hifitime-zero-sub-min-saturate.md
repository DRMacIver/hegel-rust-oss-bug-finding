# `Duration::ZERO - Duration::MIN` and `-Duration::MIN` disagree

```rust
use hifitime::Duration;

fn main() {
    println!("Duration::ZERO - Duration::MIN = {:?}", Duration::ZERO - Duration::MIN);
    println!("-Duration::MIN               = {:?}", -Duration::MIN);
    println!("Duration::MIN                = {:?}", Duration::MIN);
    println!("Duration::MAX                = {:?}", Duration::MAX);
}
```

```
Duration::ZERO - Duration::MIN = Duration { centuries: -32768, nanoseconds: 0 }
-Duration::MIN               = Duration { centuries: 32767, nanoseconds: 3155760000000000000 }
Duration::MIN                = Duration { centuries: -32768, nanoseconds: 0 }
Duration::MAX                = Duration { centuries: 32767, nanoseconds: 3155760000000000000 }
```

`Duration::ZERO - Duration::MIN` is mathematically the same operation as `-Duration::MIN`, and unary negation correctly saturates it to `Duration::MAX`. But going through `Sub` instead returns `Duration::MIN` unchanged, as if the subtraction had no effect at all.

Tested on hifitime 4.3.0.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
