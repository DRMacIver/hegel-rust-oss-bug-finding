# Task list marker with `ENABLE_TASKLISTS` can end up inside a heading

```rust
use pulldown_cmark::{html, Options, Parser};

fn main() {
    let input = "- [ ] a\n  - ";
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TASKLISTS);

    for event in Parser::new_ext(input, options) {
        println!("{:?}", event);
    }

    let parser = Parser::new_ext(input, options);
    let mut html_out = String::new();
    html::push_html(&mut html_out, parser);
    println!("---HTML---");
    println!("{}", html_out);
}
```

Output:

```
Start(List(None))
Start(Item)
Start(Heading { level: H2, id: None, classes: [], attrs: [] })
TaskListMarker(false)
Text(Borrowed("a"))
End(Heading(H2))
End(Item)
End(List(false))
---HTML---
<ul>
<li>
<h2><input disabled="" type="checkbox"/>
a</h2>
</li>
</ul>
```

The `TaskListMarker` event, and the resulting `<input type="checkbox">`, land inside the `Heading` (a setext `h2` formed by the `  -` underline), producing `<h2><input .../>a</h2>`. Per GFM a task list marker is only produced when the list item's content is a paragraph; here the content is a heading, so no marker should be emitted.

Tested on pulldown-cmark 0.13.4.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
