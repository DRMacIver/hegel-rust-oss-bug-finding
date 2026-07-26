# A bracket-quoted identifier containing `]]` round-trips to SQL that fails to reparse

```rust
use sqlparser::dialect::MsSqlDialect;
use sqlparser::parser::Parser;

fn main() {
    let sql = "SELECT [a]]b]";
    let ast = Parser::parse_sql(&MsSqlDialect {}, sql).unwrap();
    let printed = ast[0].to_string();
    println!("input:    {}", sql);
    println!("printed:  {}", printed);
    let reparsed = Parser::parse_sql(&MsSqlDialect {}, &printed);
    println!("reparsed: {:?}", reparsed);
}
```

```
input:    SELECT [a]]b]
printed:  SELECT [a]b]
reparsed: Err(ParserError("Expected: end of statement, found: ] at Line: 1, Column: 12"))
```

`[a]]b]` is a bracket-quoted identifier whose value is `a]b` (the tokenizer folds the doubled `]]` into a literal `]`). Displaying the parsed AST back to SQL writes `[a]b]` instead, and that string does not parse as the same identifier: it parses as `a` followed by leftover tokens `]b]`.

Tested on sqlparser 0.62.0.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
