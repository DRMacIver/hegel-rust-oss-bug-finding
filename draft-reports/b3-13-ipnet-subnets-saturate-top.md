# `Ipv4Subnets`/`Ipv6Subnets` cover an address past the given `end`

```rust
use ipnet::{Ipv4Subnets, Ipv6Subnets};
use std::net::{Ipv4Addr, Ipv6Addr};

fn main() {
    let start = Ipv4Addr::new(0, 0, 0, 0);
    let end = Ipv4Addr::new(255, 255, 255, 254);
    let subnets: Vec<_> = Ipv4Subnets::new(start, end, 0).collect();
    println!("v4: {:?}", subnets);
    println!("  contains 255.255.255.255: {}", subnets[0].contains(&Ipv4Addr::new(255, 255, 255, 255)));

    let start6 = Ipv6Addr::from(0u128);
    let end6 = Ipv6Addr::from(u128::MAX - 1);
    let subnets6: Vec<_> = Ipv6Subnets::new(start6, end6, 0).collect();
    println!("v6: {:?}", subnets6);
    println!("  contains ffff:...:ffff: {}", subnets6[0].contains(&Ipv6Addr::from(u128::MAX)));
}
```

```
v4: [0.0.0.0/0]
  contains 255.255.255.255: true
v6: [::/0]
  contains ffff:...:ffff: true
```

`Ipv4Subnets::new`/`Ipv6Subnets::new` are documented as generating subnets "between the provided `start` and `end` IP addresses inclusive of `end`". Here `end` is one below the top of the address space (`255.255.255.254` / `ffff:ffff:ffff:ffff:ffff:ffff:ffff:fffe`), but the returned subnet (`0.0.0.0/0` / `::/0`) also contains the address one past `end` (`255.255.255.255` / all-`ffff`), which was never in the requested range. The same range one step away from the top of the address space (e.g. `255.255.255.0` to `255.255.255.254`) produces correctly bounded subnets, so this is specific to ranges that reach up to the second-to-last address.

Tested on ipnet 2.12.0.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
