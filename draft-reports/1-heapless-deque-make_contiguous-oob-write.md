# `make_contiguous` on a full `Deque` can lead to an out-of-bounds write

The following program aborts on a debug build and writes out of bounds on a release build:

```rust
use heapless::Deque;

let mut q: Deque<i32, 4> = Deque::new();
for i in 0..4 { q.push_back(i).unwrap(); }
for i in 4..7 { q.pop_front(); q.push_back(i).unwrap(); }
q.make_contiguous();
q.pop_front();
q.push_back(99).unwrap();
```

On debug it aborts with:

```
thread 'main' panicked at src/deque.rs:672:35:
unsafe precondition(s) violated: slice::get_unchecked_mut requires that the index is within the slice
thread caused non-unwinding panic. aborting.
```

On release there is no error; the final `push_back` writes past the end of the backing array.

The deque is full (4/4) and wrapped when `make_contiguous` is called. After that call, `is_full()` returns `false` even though the deque still holds 4 elements:

```rust
use heapless::Deque;

let mut q: Deque<i32, 4> = Deque::new();
for i in 0..4 { q.push_back(i).unwrap(); }
for i in 4..7 { q.pop_front(); q.push_back(i).unwrap(); }
q.make_contiguous();
q.pop_front();
q.push_front(3).unwrap();
assert_eq!(q.len(), 4);
assert!(q.is_full()); // fails: returns false
```

The `push_back` in the first example goes ahead only because `is_full()` wrongly reports `false`.

Tested on current `main` and on 0.9.3.

BTW, this bug was found using [hegel](https://crates.io/crates/hegeltest). Happy to contribute the tests if you're interested.
