//! Property tests for hecs column-batch spawning:
//! `ColumnBatchType` / `ColumnBatchBuilder` / `ColumnBatch`,
//! `World::spawn_column_batch` and `World::spawn_column_batch_at`.
//!
//! Oracle design:
//!   * `spawn_column_batch` is compared against a second world populated by spawning the
//!     same rows one-by-one with plain `World::spawn` — the multiset of observable
//!     component tuples must be identical, and lengths must agree.
//!   * Batch-spawned handles must be valid, distinct, `contains`-true, and carry exactly
//!     the pushed column values, in push order (hecs guarantees and upstream-tests this
//!     ordering).
//!   * Archetype structural invariant: live entities partition across archetypes
//!     (each id in exactly one archetype; Σ archetype.len() == world.len()).
//!   * Drop oracle: `D` is a non-Copy component whose constructor/Drop bump a
//!     thread-local live counter. At every checkpoint the counter must equal the number
//!     of live `D` components; after dropping the worlds it must return to zero. This
//!     catches leaks and double-drops in the unsafe bulk-copy machinery.
//!   * `spawn_column_batch_at` is exercised against three kinds of target handle
//!     (live entity being replaced, reserved-then-flushed empty entity, despawned
//!     handle), asserting the batch rows land on exactly those handles, replaced
//!     components (including a component type *not* in the batch) are gone and dropped,
//!     and bystander entities are untouched.
//!
//! Known upstream bug (found while writing this file, deliberately NOT asserted here
//! because it fails against hecs 0.11.0; reported separately): components written into a
//! `ColumnBatchBuilder` are leaked (their destructors never run) whenever the builder is
//! dropped without a successful `build()` — both when the builder is simply dropped and
//! when `build()` returns `Err(BatchIncomplete)`. `ColumnBatchBuilder::drop` iterates
//! with a byte stride over a `*mut u8` and calls `drop_in_place::<u8>` (a no-op), and
//! `build()` moves the archetype out (len still 0) before the completeness check, so the
//! `Err` path never drops written components at all. The tests below therefore only
//! assert the *documented* contract (`Err(BatchIncomplete)`) on those paths, using
//! Copy-only components so the drop-oracle stays sound elsewhere.

use std::cell::Cell;
use std::collections::HashSet;

use hecs::{ColumnBatch, ColumnBatchType, Entity, World};
use hegel::generators as gs;

// ---- fixed component universe ----

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct A(i32);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct B(i32);
/// A component type that is never part of a batch: used both to check
/// `ColumnBatchBuilder::writer` for an absent type, and as a pre-existing component on
/// entities replaced by `spawn_column_batch_at` (it must not survive the replacement).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct E(i32);

// Drop-tracked component: NOT Copy; bumps a per-thread live-count in new/drop.
thread_local! { static D_LIVE: Cell<i64> = const { Cell::new(0) }; }
#[derive(Debug)]
struct D(i32);
impl D {
    fn new(v: i32) -> D {
        D_LIVE.with(|c| c.set(c.get() + 1));
        D(v)
    }
}
impl Drop for D {
    fn drop(&mut self) {
        D_LIVE.with(|c| c.set(c.get() - 1));
    }
}
fn d_live() -> i64 {
    D_LIVE.with(|c| c.get())
}

fn val() -> impl gs::Generator<i32> {
    gs::integers::<i32>().min_value(-3).max_value(3)
}

/// One entity's worth of batch data: payloads for its A, B and D components.
type Row = (i32, i32, i32);

fn draw_rows(tc: &hegel::TestCase, max: usize) -> Vec<Row> {
    let n = tc.draw(gs::integers::<usize>().min_value(0).max_value(max));
    (0..n)
        .map(|_| (tc.draw(val()), tc.draw(val()), tc.draw(val())))
        .collect()
}

/// Build a complete `ColumnBatch` of {A, B, D} from `rows`, exercising the whole builder
/// surface along the way: per-column writers, `fill()`, rejection of pushes past
/// capacity, and `writer` returning `None` for a type not in the batch.
fn build_batch(rows: &[Row]) -> ColumnBatch {
    let mut ty = ColumnBatchType::new();
    ty.add::<A>();
    ty.add::<B>();
    ty.add::<D>();
    let builder = ty.into_batch(rows.len() as u32);

    assert!(
        builder.writer::<E>().is_none(),
        "writer::<E>() must be None: E is not in the ColumnBatchType"
    );

    {
        let mut wa = builder.writer::<A>().expect("A writer");
        for &(a, _, _) in rows {
            wa.push(A(a)).expect("push A within capacity");
        }
        assert_eq!(wa.fill(), rows.len() as u32, "A column fill count");
        assert_eq!(
            wa.push(A(i32::MAX)),
            Err(A(i32::MAX)),
            "push past capacity must return the value back"
        );
    }
    {
        let mut wb = builder.writer::<B>().expect("B writer");
        for &(_, b, _) in rows {
            wb.push(B(b)).expect("push B within capacity");
        }
        assert_eq!(wb.fill(), rows.len() as u32, "B column fill count");
    }
    {
        let mut wd = builder.writer::<D>().expect("D writer");
        for &(_, _, d) in rows {
            wd.push(D::new(d)).expect("push D within capacity");
        }
        assert_eq!(wd.fill(), rows.len() as u32, "D column fill count");
    }

    builder.build().expect("fully-filled batch must build")
}

// ---- oracles ----

/// Normalized, order-independent observable form of a world over {A, B, D}.
fn observe(world: &World) -> Vec<(Option<i32>, Option<i32>, Option<i32>)> {
    let mut v: Vec<_> = world
        .iter()
        .map(|e| {
            (
                e.get::<&A>().map(|r| r.0),
                e.get::<&B>().map(|r| r.0),
                e.get::<&D>().map(|r| r.0),
            )
        })
        .collect();
    v.sort();
    v
}

/// Number of live `D` components in a world.
fn d_count(world: &World) -> i64 {
    world.query::<&D>().iter().count() as i64
}

/// Archetype structural invariant: live entities partition across archetypes.
fn check_partition(world: &World) {
    let mut arch_total = 0u32;
    let mut ids: HashSet<u32> = HashSet::new();
    for arch in world.archetypes() {
        arch_total += arch.len();
        for &id in arch.ids() {
            assert!(ids.insert(id), "entity id {} appears in more than one archetype", id);
        }
    }
    assert_eq!(arch_total, world.len(), "sum of archetype lens != world.len()");
    assert_eq!(ids.len() as u32, world.len(), "archetype id count != world.len()");
}

// ---- property 1: spawn_column_batch vs one-by-one spawn ----

fn drive_spawn_column_batch(tc: &hegel::TestCase) {
    assert_eq!(d_live(), 0, "D counter dirty at case start");
    {
        let mut world_batch = World::new();
        let mut world_direct = World::new();

        // Seed both worlds identically with entities of a *different* archetype, so the
        // batch lands in a world that already has unrelated archetypes.
        let n_seed_ab = tc.draw(gs::integers::<u32>().min_value(0).max_value(6));
        for _ in 0..n_seed_ab {
            let a = tc.draw(val());
            let b = tc.draw(val());
            world_batch.spawn((A(a), B(b)));
            world_direct.spawn((A(a), B(b)));
        }
        // ... and optionally with entities of the batch's *own* archetype, so
        // `insert_batch` sometimes merges into a pre-existing archetype.
        let n_seed_abd = tc.draw(gs::integers::<u32>().min_value(0).max_value(3));
        for _ in 0..n_seed_abd {
            let a = tc.draw(val());
            let b = tc.draw(val());
            let d = tc.draw(val());
            world_batch.spawn((A(a), B(b), D::new(d)));
            world_direct.spawn((A(a), B(b), D::new(d)));
        }

        // One or two successive batches; the second one always hits the
        // duplicate-archetype (merge into existing storage) path.
        let n_batches = tc.draw(gs::integers::<u8>().min_value(1).max_value(2));
        for _ in 0..n_batches {
            let rows = draw_rows(tc, 32);

            let batch = build_batch(&rows);
            let len_before = world_batch.len();
            let mut seen: HashSet<Entity> =
                world_batch.iter().map(|e| e.entity()).collect();

            let iter = world_batch.spawn_column_batch(batch);
            assert_eq!(iter.len(), rows.len(), "SpawnColumnBatchIter::len()");
            let ents: Vec<Entity> = iter.collect();
            assert_eq!(ents.len(), rows.len(), "number of spawned entities");
            assert_eq!(
                world_batch.len(),
                len_before + rows.len() as u32,
                "world.len() after spawn_column_batch"
            );

            // Every batch-spawned handle: distinct (also from pre-existing entities),
            // contained, and carrying exactly the pushed values, in push order.
            for (i, (&e, row)) in ents.iter().zip(&rows).enumerate() {
                assert!(seen.insert(e), "handle {:?} (row {}) is not distinct", e, i);
                assert!(world_batch.contains(e), "spawned handle {:?} not contained", e);
                assert_eq!(world_batch.get::<&A>(e).unwrap().0, row.0, "A of row {}", i);
                assert_eq!(world_batch.get::<&B>(e).unwrap().0, row.1, "B of row {}", i);
                assert_eq!(world_batch.get::<&D>(e).unwrap().0, row.2, "D of row {}", i);
            }

            // Oracle world: same rows spawned individually.
            for &(a, b, d) in &rows {
                world_direct.spawn((A(a), B(b), D::new(d)));
            }

            assert_eq!(world_batch.len(), world_direct.len(), "world lens diverged");
            assert_eq!(
                observe(&world_batch),
                observe(&world_direct),
                "multiset of component tuples diverged between batch and direct spawn"
            );
            check_partition(&world_batch);
            check_partition(&world_direct);
            assert_eq!(
                d_live(),
                d_count(&world_batch) + d_count(&world_direct),
                "live D count != D components in the two worlds (leak/double-drop)"
            );
        }

        drop(world_batch);
        assert_eq!(
            d_live(),
            d_count(&world_direct),
            "dropping the batch world did not drop exactly its D components"
        );
        drop(world_direct);
    }
    assert_eq!(d_live(), 0, "D leaked or double-dropped by end of case");
}

#[hegel::test(test_cases = 500)]
fn spawn_column_batch_matches_direct_spawn(tc: hegel::TestCase) {
    drive_spawn_column_batch(&tc);
}

// ---- property 2: spawn_column_batch_at ----

#[derive(Clone, Copy, PartialEq, Eq)]
enum TargetKind {
    /// Live entity with components (incl. an old D and an E) that must be replaced.
    Replaced,
    /// reserve_entity() handle, flushed to an empty entity before the call.
    Reserved,
    /// Spawned then despawned; the batch resurrects the exact handle.
    Despawned,
}

fn drive_spawn_column_batch_at(tc: &hegel::TestCase) {
    assert_eq!(d_live(), 0, "D counter dirty at case start");
    {
        let mut world = World::new();

        // Bystanders that must come through the batch completely untouched.
        let n_by = tc.draw(gs::integers::<usize>().min_value(0).max_value(5));
        let mut bystanders: Vec<(Entity, i32, i32)> = Vec::new();
        for _ in 0..n_by {
            let a = tc.draw(val());
            let v = tc.draw(val());
            let e = world.spawn((A(a), E(v)));
            bystanders.push((e, a, v));
        }

        // Draw target kinds, then materialize them in an order that guarantees all
        // target ids are distinct (all spawns strictly before any reserve, all
        // despawns after both, and nothing else allocates ids afterwards — so no id
        // ever gets recycled into a second handle in `handles`).
        let n = tc.draw(gs::integers::<usize>().min_value(0).max_value(16));
        let kinds: Vec<TargetKind> = (0..n)
            .map(|_| match tc.draw(gs::integers::<u8>().min_value(0).max_value(2)) {
                0 => TargetKind::Replaced,
                1 => TargetKind::Reserved,
                _ => TargetKind::Despawned,
            })
            .collect();

        let mut handles: Vec<Entity> = Vec::with_capacity(n);
        for &kind in &kinds {
            match kind {
                TargetKind::Replaced => {
                    let a = tc.draw(val());
                    let v = tc.draw(val());
                    let d = tc.draw(val());
                    handles.push(world.spawn((A(a), E(v), D::new(d))));
                }
                TargetKind::Despawned => {
                    let b = tc.draw(val());
                    handles.push(world.spawn((B(b),)));
                }
                TargetKind::Reserved => handles.push(Entity::DANGLING), // placeholder
            }
        }
        for (h, &kind) in handles.iter_mut().zip(&kinds) {
            if kind == TargetKind::Reserved {
                *h = world.reserve_entity();
            }
        }
        world.flush();
        for (&h, &kind) in handles.iter().zip(&kinds) {
            if kind == TargetKind::Despawned {
                world.despawn(h).expect("despawn of live target");
            }
        }

        let live_targets_before = kinds
            .iter()
            .filter(|&&k| k != TargetKind::Despawned)
            .count();
        assert_eq!(world.len() as usize, n_by + live_targets_before);
        let d_before_batch = d_live();

        let rows = draw_rows(tc, 16);
        // spawn_column_batch_at requires handles.len() == batch len; use exactly n rows.
        let rows: Vec<Row> = {
            let mut r = rows;
            r.truncate(n);
            while r.len() < n {
                r.push((tc.draw(val()), tc.draw(val()), tc.draw(val())));
            }
            r
        };
        let batch = build_batch(&rows);
        assert_eq!(d_live(), d_before_batch + n as i64, "batch holds one D per row");

        world.spawn_column_batch_at(&handles, batch);

        // Every target handle now exists and carries exactly its row, and nothing else:
        // in particular the E and D components of replaced entities must be gone.
        assert_eq!(world.len() as usize, n_by + n, "world.len() after spawn_column_batch_at");
        for (i, (&e, row)) in handles.iter().zip(&rows).enumerate() {
            assert!(world.contains(e), "target handle {:?} (row {}) not contained", e, i);
            assert_eq!(world.get::<&A>(e).unwrap().0, row.0, "A of target row {}", i);
            assert_eq!(world.get::<&B>(e).unwrap().0, row.1, "B of target row {}", i);
            assert_eq!(world.get::<&D>(e).unwrap().0, row.2, "D of target row {}", i);
            assert!(
                world.get::<&E>(e).is_err(),
                "component E from the replaced entity survived on {:?} (row {})",
                e,
                i
            );
        }
        // Bystanders untouched.
        for &(e, a, v) in &bystanders {
            assert!(world.contains(e), "bystander {:?} vanished", e);
            assert_eq!(world.get::<&A>(e).unwrap().0, a, "bystander A changed");
            assert_eq!(world.get::<&E>(e).unwrap().0, v, "bystander E changed");
            assert!(world.get::<&D>(e).is_err(), "bystander {:?} gained a D", e);
        }
        check_partition(&world);
        // Old Ds of replaced entities must have been dropped exactly once; the batch's
        // Ds are now the only live ones.
        assert_eq!(d_count(&world), n as i64, "one D per target after the batch");
        assert_eq!(
            d_live(),
            d_count(&world),
            "live D count != D components in world (replaced components leaked or double-dropped)"
        );

        drop(world);
    }
    assert_eq!(d_live(), 0, "D leaked or double-dropped by end of case");
}

#[hegel::test(test_cases = 500)]
fn spawn_column_batch_at_places_rows_on_handles(tc: hegel::TestCase) {
    drive_spawn_column_batch_at(&tc);
}

// ---- deterministic edge cases ----

#[test]
fn empty_batch_spawns_nothing() {
    assert_eq!(d_live(), 0);
    {
        let mut world = World::new();
        let ents: Vec<Entity> = world.spawn_column_batch(build_batch(&[])).collect();
        assert!(ents.is_empty());
        assert_eq!(world.len(), 0);
        check_partition(&world);

        // ... and into a non-empty world, which must be left unchanged.
        let e = world.spawn((A(1), B(2), D::new(3)));
        let ents: Vec<Entity> = world.spawn_column_batch(build_batch(&[])).collect();
        assert!(ents.is_empty());
        assert_eq!(world.len(), 1);
        assert!(world.contains(e));
        assert_eq!(world.get::<&A>(e).unwrap().0, 1);
        check_partition(&world);

        // Zero-length spawn_column_batch_at with zero handles is a no-op too.
        world.spawn_column_batch_at(&[], build_batch(&[]));
        assert_eq!(world.len(), 1);
        drop(world);
    }
    assert_eq!(d_live(), 0);
}

#[test]
fn single_row_batch() {
    assert_eq!(d_live(), 0);
    {
        let mut world = World::new();
        let ents: Vec<Entity> = world
            .spawn_column_batch(build_batch(&[(7, -7, 42)]))
            .collect();
        assert_eq!(ents.len(), 1);
        assert_eq!(world.len(), 1);
        assert_eq!(world.get::<&A>(ents[0]).unwrap().0, 7);
        assert_eq!(world.get::<&B>(ents[0]).unwrap().0, -7);
        assert_eq!(world.get::<&D>(ents[0]).unwrap().0, 42);
        check_partition(&world);
        assert_eq!(d_live(), 1);
        drop(world);
    }
    assert_eq!(d_live(), 0);
}

#[test]
fn large_batch_10k() {
    assert_eq!(d_live(), 0);
    {
        const N: i32 = 10_000;
        let rows: Vec<Row> = (0..N).map(|i| (i, 2 * i, 3 * i)).collect();
        let mut world = World::new();
        let ents: Vec<Entity> = world.spawn_column_batch(build_batch(&rows)).collect();
        assert_eq!(ents.len(), N as usize);
        assert_eq!(world.len(), N as u32);
        for (i, &e) in ents.iter().enumerate() {
            let i = i as i32;
            assert!(world.contains(e));
            assert_eq!(world.get::<&A>(e).unwrap().0, i);
            assert_eq!(world.get::<&B>(e).unwrap().0, 2 * i);
            assert_eq!(world.get::<&D>(e).unwrap().0, 3 * i);
        }
        let distinct: HashSet<Entity> = ents.iter().copied().collect();
        assert_eq!(distinct.len(), N as usize, "all 10k handles distinct");
        check_partition(&world);
        assert_eq!(d_live(), N as i64);
        drop(world);
    }
    assert_eq!(d_live(), 0, "large batch leaked or double-dropped D components");
}

/// A builder whose columns are not all filled to `size` must refuse to build.
/// (Documented contract: `build()` returns `Err(BatchIncomplete)`.)
///
/// Deliberately uses only Copy components: hecs 0.11.0 leaks the already-written
/// component values on this path (see file-level comment), so a Drop-tracked component
/// here would poison the thread-local counter for unrelated tests. The leak itself is
/// reported upstream separately rather than asserted around.
#[test]
fn incomplete_batch_does_not_build() {
    let mut ty = ColumnBatchType::new();
    ty.add::<A>();
    ty.add::<B>();
    let builder = ty.into_batch(3);
    {
        let mut wa = builder.writer::<A>().unwrap();
        wa.push(A(1)).unwrap();
        wa.push(A(2)).unwrap();
        wa.push(A(3)).unwrap();
        let mut wb = builder.writer::<B>().unwrap();
        wb.push(B(1)).unwrap();
        wb.push(B(2)).unwrap(); // one short
    }
    assert!(
        builder.build().is_err(),
        "build() must fail while a column is underfilled"
    );

    // A column that was never written at all must also prevent building.
    let mut ty = ColumnBatchType::new();
    ty.add::<A>();
    ty.add::<B>();
    let builder = ty.into_batch(1);
    {
        let mut wa = builder.writer::<A>().unwrap();
        wa.push(A(1)).unwrap();
    }
    assert!(builder.build().is_err(), "untouched column must prevent building");
}

#[test]
#[should_panic(expected = "must match number of entities")]
fn mismatched_handle_count_panics() {
    let mut world = World::new();
    let e = world.spawn((A(0),));
    // 2-row batch, 1 handle: documented to panic.
    let mut ty = ColumnBatchType::new();
    ty.add::<A>();
    ty.add::<B>();
    let builder = ty.into_batch(2);
    {
        let mut wa = builder.writer::<A>().unwrap();
        wa.push(A(1)).unwrap();
        wa.push(A(2)).unwrap();
        let mut wb = builder.writer::<B>().unwrap();
        wb.push(B(1)).unwrap();
        wb.push(B(2)).unwrap();
    }
    let batch = builder.build().unwrap();
    world.spawn_column_batch_at(&[e], batch);
}
