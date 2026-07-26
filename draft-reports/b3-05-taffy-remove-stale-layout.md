# `TaffyTree::remove` leaves the ex-parent's cached layout stale

```rust
use taffy::prelude::*;

fn main() {
    let mut tree: TaffyTree<()> = TaffyTree::new();

    let child = tree
        .new_leaf(Style {
            border: Rect {
                top: LengthPercentage::length(0.0),
                bottom: LengthPercentage::length(1.0),
                left: LengthPercentage::length(0.0),
                right: LengthPercentage::length(0.0),
            },
            ..Default::default()
        })
        .unwrap();

    let root = tree.new_with_children(Style::default(), &[child]).unwrap();

    tree.compute_layout(root, Size::MIN_CONTENT).unwrap();
    println!("height after first layout: {}", tree.layout(root).unwrap().size.height);

    let _ = tree.remove(child);
    println!("dirty(root) after remove: {:?}", tree.dirty(root));

    tree.compute_layout(root, Size::MIN_CONTENT).unwrap();
    println!("height after remove + recompute: {}", tree.layout(root).unwrap().size.height);
}
```

Output:

```
height after first layout: 1
dirty(root) after remove: Ok(false)
height after remove + recompute: 1
```

The root's height comes entirely from its one child's 1px bottom border. After that child is removed with `remove`, `dirty(root)` still reports `false`, and a subsequent `compute_layout` on the root leaves it at the old height of 1 instead of recomputing it to 0 for the now-childless root.

Tested on taffy 0.12.2.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
