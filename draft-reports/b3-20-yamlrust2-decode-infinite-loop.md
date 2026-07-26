# `YamlDecoder::decode` hangs forever on some short byte inputs

```rust
use yaml_rust2::yaml::YamlDecoder;

fn main() {
    let input: &[u8] = &[0xFF, 0xFE, 0x34, 0x12, 0x34, 0x12, 0x34, 0x12];
    println!("starting decode...");
    let docs = YamlDecoder::read(input).decode();
    println!("decode returned: {:?}", docs.is_ok());
}
```

Output:

```
$ timeout 5 cargo run
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.00s
     Running `target/debug/repro`
starting decode...
```

The process is killed by the timeout; `decode()` never returns and the "decode returned" line is never printed. The same happens without a timeout — it just hangs indefinitely, consuming a CPU core.

Tested on yaml-rust2 0.11.0.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
