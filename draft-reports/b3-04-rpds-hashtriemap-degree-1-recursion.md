# `HashTrieMap::new_with_degree(1)` causes a stack overflow on the second insert

```rust
use rpds::HashTrieMap;

fn main() {
    let mut m: HashTrieMap<i32, i32> = HashTrieMap::new_with_degree(1);
    println!("created with degree 1");
    m = m.insert(1, 1);
    println!("first insert ok");
    m = m.insert(2, 2);
    println!("second insert ok: {:?}", m.get(&2));
}
```

```
created with degree 1
first insert ok

thread 'main' has overflowed its stack
fatal runtime error: stack overflow, aborting
```

`new_with_degree(1)` is accepted without error (1 is a power of two, which is what the constructor checks), and the first insert succeeds; the second insert overflows the stack and aborts the process. `new_with_degree` should reject a degree of 1 at construction rather than accept it and then abort on the second insert.

Tested on rpds 1.2.1.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
