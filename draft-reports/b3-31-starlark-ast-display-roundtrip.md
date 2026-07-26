# Displaying a parsed AST and reparsing the result fails for slices, f-strings, and dotted integer literals

```rust
use starlark::syntax::{AstModule, Dialect};

fn roundtrip(src: &str) {
    let dialect = Dialect {
        enable_f_strings: true,
        ..Dialect::Extended
    };
    let ast = AstModule::parse("t.star", src.to_owned(), &dialect).unwrap();
    let printed = format!("{}", ast.statement().node);
    println!("input:   {:?}", src);
    println!("printed: {:?}", printed);
    match AstModule::parse("t2.star", printed.clone(), &dialect) {
        Ok(ast2) => {
            let printed2 = format!("{}", ast2.statement().node);
            if printed2 != printed {
                println!("REPARSED BUT DID NOT MATCH: {:?}", printed2);
            } else {
                println!("reparsed ok, matches");
            }
        }
        Err(e) => println!("REPARSE FAILED: {}", e),
    }
    println!();
}

fn main() {
    roundtrip("x[1:2]\n");
    roundtrip("f\"a{x}b\"\n");
    roundtrip("(1).imag\n");
    roundtrip("1.0\n");
}
```

```
input:   "x[1:2]\n"
printed: "x[]1:2\n"
REPARSE FAILED: error: Parse error: unexpected symbol ']', expected expression
 --> t2.star:1:3
  |
1 | x[]1:2
  |   ^
  |


input:   "f\"a{x}b\"\n"
printed: "a{}b.format(x)\n"
REPARSE FAILED: error: Parse error: unexpected symbol '{', expected new line
 --> t2.star:1:2
  |
1 | a{}b.format(x)
  |  ^
  |


input:   "(1).imag\n"
printed: "1.imag\n"
REPARSE FAILED: error: Parse error: unexpected identifier 'imag', expected new line
 --> t2.star:1:3
  |
1 | 1.imag
  |   ^^^^
  |


input:   "1.0\n"
printed: "1\n"
reparsed ok, matches
```

Parsing `x[1:2]`, `f"a{x}b"`, and `(1).imag`, then printing the parsed AST with `Display` and parsing that output again, fails to parse in all three cases — the printed text is not valid Starlark. Printing `1.0` "succeeds" in the sense that the result reparses, but it reparses as the integer `1` rather than the float `1.0`.

Tested on starlark 0.14.2 (latest release).

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
