# `BroCatli` reports Success but produces a stream that fails to decompress when an empty catable stream is appended after a particular quality-1 stream

```rust
use brotli::concat::{BroCatli, BroCatliResult};
use brotli::enc::backward_references::BrotliEncoderParams;
use brotli::{BrotliCompress, BrotliDecompress};
use std::io::Cursor;

fn compress_catable(q: i32, inp: &[u8]) -> Vec<u8> {
    let mut params = BrotliEncoderParams::default();
    params.quality = q;
    params.lgwin = 10;
    params.catable = true;
    params.use_dictionary = false;
    params.appendable = false;
    let mut out = Vec::new();
    BrotliCompress(&mut Cursor::new(inp), &mut out, &params).expect("compression failed");
    out
}

fn decompress(compressed: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut out = Vec::new();
    BrotliDecompress(&mut Cursor::new(compressed), &mut out).map(|_| out)
}

fn brocatli_concat(streams: &[Vec<u8>], out_buf_size: usize) -> Vec<u8> {
    let mut cat = BroCatli::new();
    let mut result = Vec::new();
    let mut obuf = vec![0u8; out_buf_size];
    for s in streams.iter() {
        cat.new_brotli_file();
        let mut ioffset = 0usize;
        loop {
            let mut ooffset = 0usize;
            match cat.stream(&s[..], &mut ioffset, &mut obuf[..], &mut ooffset) {
                BroCatliResult::NeedsMoreOutput => {
                    result.extend_from_slice(&obuf[..ooffset]);
                }
                BroCatliResult::NeedsMoreInput => {
                    result.extend_from_slice(&obuf[..ooffset]);
                    break;
                }
                other => panic!("unexpected BroCatli stream result: {:?}", other),
            }
        }
    }
    loop {
        let mut ooffset = 0usize;
        match cat.finish(&mut obuf[..], &mut ooffset) {
            BroCatliResult::NeedsMoreOutput => {
                result.extend_from_slice(&obuf[..ooffset]);
            }
            BroCatliResult::Success => {
                result.extend_from_slice(&obuf[..ooffset]);
                break;
            }
            other => panic!("unexpected BroCatli finish result: {:?}", other),
        }
    }
    result
}

fn main() {
    let mut input = vec![b'0'; 22];
    input[3] = b'1';

    let streams = [compress_catable(1, &input), compress_catable(5, &[])];

    // both inputs are individually valid catable streams:
    assert_eq!(decompress(&streams[0]).unwrap(), input);
    assert_eq!(decompress(&streams[1]).unwrap(), Vec::<u8>::new());

    let concatenated = brocatli_concat(&streams, 4096);
    println!("concatenated bytes: {:?}", concatenated);
    match decompress(&concatenated) {
        Ok(roundtrip) => println!("decompressed ok: {:?}", roundtrip),
        Err(e) => println!("decompress error: {:?}", e),
    }
}
```

Output:

```
concatenated bytes: [131, 0, 128, 48, 48, 152, 0, 0, 0, 10, 38, 6, 14, 224, 250, 125, 73, 137, 13, 224, 2, 28, 94, 33, 128, 128]
decompress error: Custom { kind: UnexpectedEof, error: "Unexpected EOF" }
```

Both input streams decompress correctly on their own (asserted above before the concatenation). `BroCatli::stream`/`finish` accept both and report `NeedsMoreOutput`/`Success` throughout, but the emitted concatenation ends `[0x21, 0x80, 0x80]` where the single valid stream ends `[0xA1, 0x01]`, and the result does not decompress. Using a non-empty second stream, or a first stream at quality 0, 2, or 5 instead of 1, with the same shapes concatenates and decompresses fine.

Tested on `brotli` 8.0.4 (with `brotli_decompressor` 5.0.3, resolved by Cargo).

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
