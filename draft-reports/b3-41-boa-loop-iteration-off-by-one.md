# `for` loops run one more iteration than `while` loops under the same loop iteration limit

```rust
use boa_engine::{Context, JsString, Source};

fn run_and_read_count(src: &str) -> (String, String) {
    let mut context = Context::default();
    context.runtime_limits_mut().set_loop_iteration_limit(10);

    let eval_result = context.eval(Source::from_bytes(src));
    let error = match eval_result {
        Ok(_) => "(no error)".to_string(),
        Err(e) => e.to_string(),
    };

    let count = context
        .global_object()
        .get(JsString::from("count"), &mut context)
        .expect("get count");

    (error, count.display().to_string())
}

fn main() {
    let for_src = r#"
        var count = 0;
        for (let i = 0; i < 1000; ++i) { count++; }
    "#;
    let (err, count) = run_and_read_count(for_src);
    println!("for loop:   count = {count}, error = {err}");

    let while_src = r#"
        var count = 0;
        while (true) { count++; }
    "#;
    let (err, count) = run_and_read_count(while_src);
    println!("while loop: count = {count}, error = {err}");
}
```

```
for loop:   count = 12, error = RuntimeLimit: Maximum loop iteration limit 10 exceeded
    at <main> (unknown at :?:?)
while loop: count = 11, error = RuntimeLimit: Maximum loop iteration limit 10 exceeded
    at <main> (unknown at :?:?)
```

With `runtime_limits_mut().set_loop_iteration_limit(10)` set on the same `Context`, a `for` loop's body runs 12 times before the limit error is raised, while a `while` loop's body runs only 11 times, even though both are given the identical limit of 10. A limit of 10 should stop the body at 10 iterations, and the two loop kinds should agree.

Tested on boa_engine 0.21.1.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
