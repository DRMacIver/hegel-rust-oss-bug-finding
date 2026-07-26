# Materializing then parsing a conflict can drop a trailing `\r` byte from its content

```rust
use bstr::BString;
use jj_lib::conflict_labels::ConflictLabels;
use jj_lib::conflicts::materialize_merge_result_to_bytes;
use jj_lib::conflicts::parse_conflict;
use jj_lib::conflicts::ConflictMarkerStyle;
use jj_lib::conflicts::ConflictMaterializeOptions;
use jj_lib::files::FileMergeHunkLevel;
use jj_lib::merge::Merge;
use jj_lib::merge::SameChange;
use jj_lib::tree_merge::MergeOptions;

fn main() {
    let merge = Merge::from_vec(vec![
        BString::from("left\r"),
        BString::from("base"),
        BString::from("right"),
    ]);

    let options = ConflictMaterializeOptions {
        marker_style: ConflictMarkerStyle::Git,
        marker_len: None,
        merge: MergeOptions {
            hunk_level: FileMergeHunkLevel::Line,
            same_change: SameChange::Keep,
        },
    };

    let materialized =
        materialize_merge_result_to_bytes(&merge, &ConflictLabels::unlabeled(), &options);

    let parsed = parse_conflict(&materialized, 2, 7).expect("should parse as a conflict");
    let side0 = parsed[0].iter().next().unwrap();
    println!("original left side:      {:?}", BString::from("left\r"));
    println!("round-tripped left side: {side0:?}");
}
```

```
original left side:      "left\r"
round-tripped left side: "left"
```

The merge's left side is the three bytes `left\r`, with no `\n` anywhere in any side. After `materialize_merge_result_to_bytes` writes it out and `parse_conflict` reads it back, the trailing `\r` is gone: `"left\r"` round-trips to `"left"`.

Tested on jj-lib 0.43.0.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.

*This report was drafted by an AI agent (Claude Code) and reviewed by @DRMacIver before filing.*
