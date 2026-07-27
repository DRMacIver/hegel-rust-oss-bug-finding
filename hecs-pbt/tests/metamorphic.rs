//! Metamorphic properties for hecs (hecs 0.11.0) — model-free oracles.
//!
//! Technique
//! =========
//! Instead of mirroring the World in a reference model, each test asserts a
//! *relation between two executions* that must hold by the API's documented
//! semantics:
//!
//!   * `commuting_disjoint_ops`  — ops targeting two different entities
//!     commute: X;Y ≡ Y;X observationally, and each op's own result is
//!     order-independent. (Spawning ops are excluded: allocation order is
//!     observable through handles, so only non-allocating ops commute.)
//!   * `insert_then_remove_adjusted_identity` — `insert_one::<T>` followed by
//!     `remove_one::<T>` yields exactly "the original world with T removed
//!     from that entity" (the plain identity when T was absent), and the
//!     removed value is the inserted one.
//!   * `exchange_roundtrip_adjusted_identity` — `exchange_one::<A,B>` then
//!     `exchange_one::<B,A>` (returning the taken A) restores the world
//!     except that any pre-existing B is gone (overwritten by the exchange).
//!   * `spawn_batch_matches_individual_spawns` — `spawn_batch` of N bundles
//!     is observationally identical to N individual `spawn`s, including the
//!     exact handles, and *partial consumption* of the `SpawnBatchIter` still
//!     spawns everything (its `Drop` drains the iterator).
//!   * `clear_makes_world_fresh` — after `clear()`, a world is observationally
//!     indistinguishable from `World::new()` under any subsequent operation
//!     sequence, including repeating the exact `Entity` values (documented:
//!     "clears metadata so that Entity values will repeat").
//!   * `insert_order_on_same_entity` — inserting two *different* components
//!     on one entity is order-independent, even though the two orders route
//!     through different intermediate archetypes.
//!
//! Since `World` is not `Clone`, "two executions from the same state" uses
//! twin worlds built by replaying one drawn history (hecs allocation is
//! deterministic; the twin builder asserts handle equality loudly).
//!
//! Every test keeps the Drop oracle running: `D` is drop-tracked, and the
//! live count must equal what the fingerprints account for.

mod common;

use common::*;
use hecs::{Entity, World};
use hegel::generators as gs;

// ---- a small vocabulary of non-allocating, single-entity operations ----

#[derive(Clone, Copy, Debug)]
enum TOp {
    InsertOne(u8, i32),
    RemoveOne(u8),
    InsertBundle(Bundle4),
    RemoveAB,
    RemoveCD,
    Despawn,
    TakeDrop,
    MutateA(i32),
    ExchangeAToB(i32),
    ExchangeDToA(i32),
}

fn draw_top(tc: &hegel::TestCase) -> TOp {
    match tc.draw(gs::integers::<u8>().min_value(0).max_value(9)) {
        0 => TOp::InsertOne(tc.draw(gs::integers::<u8>().min_value(0).max_value(3)), tc.draw(val())),
        1 => TOp::RemoveOne(tc.draw(gs::integers::<u8>().min_value(0).max_value(3))),
        2 => TOp::InsertBundle(draw_bundle(tc)),
        3 => TOp::RemoveAB,
        4 => TOp::RemoveCD,
        5 => TOp::Despawn,
        6 => TOp::TakeDrop,
        7 => TOp::MutateA(tc.draw(val())),
        8 => TOp::ExchangeAToB(tc.draw(val())),
        _ => TOp::ExchangeDToA(tc.draw(val())),
    }
}

/// Apply `op` to `e` in `world`; return its observable outcome (ok-ness).
fn apply_top(world: &mut World, e: Entity, op: TOp) -> bool {
    match op {
        TOp::InsertOne(which, v) => match which {
            0 => world.insert_one(e, A(v)).is_ok(),
            1 => world.insert_one(e, B(v)).is_ok(),
            2 => world.insert_one(e, C).is_ok(),
            _ => world.insert_one(e, D::new(v)).is_ok(),
        },
        TOp::RemoveOne(which) => match which {
            0 => world.remove_one::<A>(e).is_ok(),
            1 => world.remove_one::<B>(e).is_ok(),
            2 => world.remove_one::<C>(e).is_ok(),
            _ => world.remove_one::<D>(e).is_ok(),
        },
        TOp::InsertBundle(s) => world.insert(e, make_builder(s).build()).is_ok(),
        TOp::RemoveAB => world.remove::<(A, B)>(e).is_ok(),
        TOp::RemoveCD => world.remove::<(C, D)>(e).is_ok(),
        TOp::Despawn => world.despawn(e).is_ok(),
        TOp::TakeDrop => world.take(e).is_ok(),
        TOp::MutateA(v) => {
            if let Ok(mut a) = world.get::<&mut A>(e) {
                a.0 = v;
                true
            } else {
                false
            }
        }
        TOp::ExchangeAToB(v) => world.exchange_one::<A, B>(e, B(v)).is_ok(),
        TOp::ExchangeDToA(v) => world.exchange_one::<D, A>(e, A(v)).is_ok(),
    }
}

fn total_d(worlds: &[World]) -> i64 {
    worlds.iter().map(|w| fingerprint_d_count(&fingerprint(w))).sum()
}

// ---- relation 1: ops on distinct entities commute ----

fn drive_commuting(tc: &hegel::TestCase, max_entities: u32) {
    assert_d_balanced_at_start();
    let (mut worlds, pool) = build_twins(tc, 2, max_entities);
    if pool.len() < 2 {
        return;
    }
    let e1 = pick(tc, &pool).unwrap();
    let e2 = pick(tc, &pool).unwrap();
    if e1 == e2 {
        return;
    }
    let x = draw_top(tc);
    let y = draw_top(tc);

    let (rx0, ry0, rx1, ry1);
    {
        let (w0, w1) = worlds.split_at_mut(1);
        rx0 = apply_top(&mut w0[0], e1, x);
        ry0 = apply_top(&mut w0[0], e2, y);
        ry1 = apply_top(&mut w1[0], e2, y);
        rx1 = apply_top(&mut w1[0], e1, x);
    }
    assert_eq!(rx0, rx1, "result of {x:?} on {e1:?} depends on order relative to {y:?} on {e2:?}");
    assert_eq!(ry0, ry1, "result of {y:?} on {e2:?} depends on order relative to {x:?} on {e1:?}");
    let fp0 = fingerprint(&worlds[0]);
    let fp1 = fingerprint(&worlds[1]);
    assert_eq!(fp0, fp1, "worlds diverged: [{x:?} on {e1:?}; {y:?} on {e2:?}] vs reverse order");
    assert_eq!(d_live(), total_d(&worlds), "Drop imbalance after commuting ops");
    check_archetypes(&worlds[0], "X;Y world");
    check_archetypes(&worlds[1], "Y;X world");
}

// ---- relation 2: insert_one then remove_one == original minus that component ----

fn drive_insert_remove(tc: &hegel::TestCase, max_entities: u32) {
    assert_d_balanced_at_start();
    let (mut worlds, pool) = build_twins(tc, 1, max_entities);
    let world = &mut worlds[0];
    let Some(e) = pick(tc, &pool) else { return };
    let fp0 = fingerprint(world);
    let live = fp0.contains_key(&e);
    let v = tc.draw(val());
    let which = tc.draw(gs::integers::<u8>().min_value(0).max_value(3));

    let (inserted, removed_matches) = match which {
        0 => {
            let ins = world.insert_one(e, A(v)).is_ok();
            let rem = world.remove_one::<A>(e);
            (ins, rem.map(|got| got == A(v)))
        }
        1 => {
            let ins = world.insert_one(e, B(v)).is_ok();
            let rem = world.remove_one::<B>(e);
            (ins, rem.map(|got| got == B(v)))
        }
        2 => {
            let ins = world.insert_one(e, C).is_ok();
            let rem = world.remove_one::<C>(e);
            (ins, rem.map(|_| true))
        }
        _ => {
            let ins = world.insert_one(e, D::new(v)).is_ok();
            let rem = world.remove_one::<D>(e);
            (ins, rem.map(|got| got.0 == v))
        }
    };

    assert_eq!(inserted, live, "insert_one ok-ness vs liveness for {e:?}");
    match removed_matches {
        Ok(matches) => {
            assert!(live, "remove_one succeeded on dead {e:?}");
            assert!(matches, "remove_one returned a different value than just inserted");
        }
        Err(_) => assert!(!live, "remove_one failed right after successful insert on {e:?}"),
    }

    // Expected world: original, with that component absent on e (plain
    // identity whenever e lacked it before).
    let mut expected = fp0.clone();
    if let Some(obs) = expected.get_mut(&e) {
        match which {
            0 => obs.a = None,
            1 => obs.b = None,
            2 => obs.c = false,
            _ => obs.d = None,
        }
    }
    assert_eq!(fingerprint(world), expected, "insert+remove left unexpected residue");
    assert_eq!(d_live(), total_d(&worlds), "Drop imbalance after insert+remove");
}

// ---- relation 3: exchange A->B then B->A restores the world (minus old B) ----

fn drive_exchange_roundtrip(tc: &hegel::TestCase, max_entities: u32) {
    assert_d_balanced_at_start();
    let (mut worlds, pool) = build_twins(tc, 1, max_entities);
    let world = &mut worlds[0];
    let Some(e) = pick(tc, &pool) else { return };
    let fp0 = fingerprint(world);
    let had_a = fp0.get(&e).and_then(|o| o.a);
    let x = tc.draw(val());

    match world.exchange_one::<A, B>(e, B(x)) {
        Ok(old_a) => {
            let a0 = had_a.expect("exchange_one::<A,B> succeeded without a modelled A");
            assert_eq!(old_a.0, a0, "exchange returned wrong old A for {e:?}");
            let back = world
                .exchange_one::<B, A>(e, A(old_a.0))
                .expect("reverse exchange must succeed: B was just inserted");
            assert_eq!(back.0, x, "reverse exchange returned wrong B for {e:?}");
            // Original world, except any pre-existing B was overwritten by the
            // exchange and then taken by the reverse exchange.
            let mut expected = fp0.clone();
            expected.get_mut(&e).unwrap().b = None;
            assert_eq!(fingerprint(world), expected, "exchange round-trip residue on {e:?}");
        }
        Err(_) => {
            assert!(had_a.is_none(), "exchange_one::<A,B> failed but {e:?} has A");
            assert_eq!(fingerprint(world), fp0, "failed exchange mutated the world");
        }
    }
    assert_eq!(d_live(), total_d(&worlds), "Drop imbalance after exchange round-trip");
}

// ---- relation 4: spawn_batch == individual spawns (incl. partial consumption) ----

fn drive_spawn_batch(tc: &hegel::TestCase, max_entities: u32) {
    assert_d_balanced_at_start();
    let (mut worlds, _pool) = build_twins(tc, 2, max_entities);
    let n = tc.draw(gs::integers::<usize>().min_value(0).max_value(5));
    let specs: Vec<(i32, i32)> = (0..n).map(|_| (tc.draw(val()), tc.draw(val()))).collect();
    // Sometimes stop consuming the SpawnBatchIter early: its Drop impl must
    // finish spawning the remainder.
    let consume = tc.draw(gs::integers::<usize>().min_value(0).max_value(n));

    let batch_handles: Vec<Entity> = {
        let mut iter = worlds[0].spawn_batch(specs.iter().map(|&(a, b)| (A(a), B(b))));
        let mut taken = Vec::new();
        for _ in 0..consume {
            taken.push(iter.next().expect("SpawnBatchIter ended early"));
        }
        taken
        // rest of the batch spawns on drop
    };
    let loop_handles: Vec<Entity> =
        specs.iter().map(|&(a, b)| worlds[1].spawn((A(a), B(b)))).collect();

    assert_eq!(
        batch_handles,
        loop_handles[..consume].to_vec(),
        "spawn_batch handed out different handles than individual spawns"
    );
    assert_eq!(
        fingerprint(&worlds[0]),
        fingerprint(&worlds[1]),
        "spawn_batch world != individual-spawn world (n={n}, consumed={consume})"
    );
    check_archetypes(&worlds[0], "batch world");
}

// ---- relation 5: clear() makes a world behave exactly like World::new() ----

fn drive_clear_fresh(tc: &hegel::TestCase, max_entities: u32, max_steps: u32) {
    assert_d_balanced_at_start();
    let (mut worlds, _pool) = build_twins(tc, 1, max_entities);
    let mut cleared = worlds.pop().unwrap();
    cleared.clear();
    assert_eq!(cleared.len(), 0, "clear() left live entities");
    assert!(fingerprint(&cleared).is_empty(), "clear() left observable entities");
    assert_eq!(d_live(), 0, "clear() failed to drop some D components");

    // From here on, `cleared` must be indistinguishable from a fresh world —
    // including handing out the *same* Entity values (documented: clear()
    // "clears metadata so that Entity values will repeat").
    let mut fresh = World::new();
    let mut pool: Vec<Entity> = Vec::new();
    let steps = tc.draw(gs::integers::<u32>().min_value(0).max_value(max_steps));
    for _ in 0..steps {
        match tc.draw(gs::integers::<u8>().min_value(0).max_value(3)) {
            0 | 1 => {
                let s = draw_bundle(tc);
                let ec = cleared.spawn(make_builder(s).build());
                let ef = fresh.spawn(make_builder(s).build());
                assert_eq!(ec, ef, "cleared world allocated a different handle than a fresh one");
                pool.push(ec);
            }
            2 => {
                if let Some(e) = pick(tc, &pool) {
                    assert_eq!(
                        cleared.despawn(e).is_ok(),
                        fresh.despawn(e).is_ok(),
                        "despawn ok-ness diverged for {e:?}"
                    );
                }
            }
            _ => {
                if let Some(e) = pick(tc, &pool) {
                    let v = tc.draw(val());
                    assert_eq!(
                        cleared.insert_one(e, A(v)).is_ok(),
                        fresh.insert_one(e, A(v)).is_ok(),
                        "insert_one ok-ness diverged for {e:?}"
                    );
                }
            }
        }
        assert_eq!(
            fingerprint(&cleared),
            fingerprint(&fresh),
            "cleared world diverged from fresh world"
        );
    }
    check_archetypes(&cleared, "cleared");
    check_archetypes(&fresh, "fresh");
}

// ---- relation 6: inserting two different components commutes on one entity ----

fn drive_insert_order(tc: &hegel::TestCase, max_entities: u32) {
    assert_d_balanced_at_start();
    let (mut worlds, pool) = build_twins(tc, 2, max_entities);
    let Some(e) = pick(tc, &pool) else { return };
    let c1 = tc.draw(gs::integers::<u8>().min_value(0).max_value(3));
    let c2 = tc.draw(gs::integers::<u8>().min_value(0).max_value(3));
    if c1 == c2 {
        return;
    }
    let (v1, v2) = (tc.draw(val()), tc.draw(val()));

    let ins = |w: &mut World, which: u8, v: i32| -> bool {
        match which {
            0 => w.insert_one(e, A(v)).is_ok(),
            1 => w.insert_one(e, B(v)).is_ok(),
            2 => w.insert_one(e, C).is_ok(),
            _ => w.insert_one(e, D::new(v)).is_ok(),
        }
    };
    let r10 = ins(&mut worlds[0], c1, v1);
    let r20 = ins(&mut worlds[0], c2, v2);
    let r21 = ins(&mut worlds[1], c2, v2);
    let r11 = ins(&mut worlds[1], c1, v1);
    assert_eq!(r10, r11, "insert of component {c1} order-dependent on {e:?}");
    assert_eq!(r20, r21, "insert of component {c2} order-dependent on {e:?}");
    assert_eq!(
        fingerprint(&worlds[0]),
        fingerprint(&worlds[1]),
        "insert order of components {c1},{c2} observable on {e:?}"
    );
    assert_eq!(d_live(), total_d(&worlds), "Drop imbalance after ordered inserts");
}

// ---- entry points ----

#[cfg(not(miri))]
mod normal {
    use super::*;

    #[hegel::test(test_cases = 500)]
    fn commuting_disjoint_ops(tc: hegel::TestCase) {
        drive_commuting(&tc, 6);
    }

    #[hegel::test(test_cases = 500)]
    fn insert_then_remove_adjusted_identity(tc: hegel::TestCase) {
        drive_insert_remove(&tc, 6);
    }

    #[hegel::test(test_cases = 500)]
    fn exchange_roundtrip_adjusted_identity(tc: hegel::TestCase) {
        drive_exchange_roundtrip(&tc, 6);
    }

    #[hegel::test(test_cases = 500)]
    fn spawn_batch_matches_individual_spawns(tc: hegel::TestCase) {
        drive_spawn_batch(&tc, 4);
    }

    #[hegel::test(test_cases = 300)]
    fn clear_makes_world_fresh(tc: hegel::TestCase) {
        drive_clear_fresh(&tc, 6, 12);
    }

    #[hegel::test(test_cases = 500)]
    fn insert_order_on_same_entity(tc: hegel::TestCase) {
        drive_insert_order(&tc, 6);
    }
}

#[cfg(miri)]
mod miri {
    use super::*;

    #[hegel::test(test_cases = 8, suppress_health_check = [hegel::HealthCheck::TooSlow])]
    fn commuting_disjoint_ops(tc: hegel::TestCase) {
        drive_commuting(&tc, 4);
    }

    #[hegel::test(test_cases = 8, suppress_health_check = [hegel::HealthCheck::TooSlow])]
    fn spawn_batch_matches_individual_spawns(tc: hegel::TestCase) {
        drive_spawn_batch(&tc, 3);
    }

    #[hegel::test(test_cases = 8, suppress_health_check = [hegel::HealthCheck::TooSlow])]
    fn clear_makes_world_fresh(tc: hegel::TestCase) {
        drive_clear_fresh(&tc, 4, 6);
    }
}
