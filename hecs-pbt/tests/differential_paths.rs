//! Differential construction-path testing for hecs (hecs 0.11.0).
//!
//! Technique
//! =========
//! The same *logical* world can be built through several distinct API paths,
//! each exercising different machinery inside hecs:
//!
//!   1. `spawn(EntityBuilder)`          — DynamicBundle via the dynamic builder;
//!   2. `spawn(concrete tuple)`         — the static `Bundle` impls (arity 0–4,
//!      dispatched over all 16 subsets of the component universe);
//!   3. `reserve_entity` + `insert`     — allocation via the atomic reserve path,
//!      materialized by insert's implicit flush;
//!   4. `CommandBuffer::spawn` + `run_on` — deferred spawning through the
//!      buffer's raw component storage;
//!   5. `spawn(())` + `insert_one` each — incremental archetype migration
//!      (empty → ... → final archetype, one component at a time).
//!
//! Because hecs allocates handles deterministically, all five worlds must end
//! up **observationally identical**: same `Entity` handles, same component
//! sets, same values (asserted via the shared fingerprint). The spec itself
//! provides ground truth: every world's fingerprint must equal the fingerprint
//! implied by the drawn specs — so the five paths can't all agree on a wrong
//! answer.
//!
//! After construction, a drawn sequence of mutations is applied identically to
//! all five worlds. Internally the worlds differ (e.g. path 5 created a chain
//! of intermediate archetypes; path 2 created only the final ones), so this
//! phase checks that construction history is *not* observable: every operation
//! must report the same result and leave all worlds fingerprint-equal.
//!
//! The Drop oracle runs throughout: `D` is drop-tracked and the live count
//! must equal what the five fingerprints account for (catches a path that
//! leaks or double-drops components while building).

mod common;

use common::*;
use hecs::{CommandBuffer, Entity, World};
use hegel::generators as gs;

const N_PATHS: usize = 5;

/// Spawn `spec` via the static tuple `Bundle` impls — one concrete tuple type
/// per subset of {A, B, C, D}.
fn spawn_tuple(world: &mut World, s: Bundle4) -> Entity {
    match (s.a, s.b, s.c, s.d) {
        (None, None, false, None) => world.spawn(()),
        (Some(a), None, false, None) => world.spawn((A(a),)),
        (None, Some(b), false, None) => world.spawn((B(b),)),
        (None, None, true, None) => world.spawn((C,)),
        (None, None, false, Some(d)) => world.spawn((D::new(d),)),
        (Some(a), Some(b), false, None) => world.spawn((A(a), B(b))),
        (Some(a), None, true, None) => world.spawn((A(a), C)),
        (Some(a), None, false, Some(d)) => world.spawn((A(a), D::new(d))),
        (None, Some(b), true, None) => world.spawn((B(b), C)),
        (None, Some(b), false, Some(d)) => world.spawn((B(b), D::new(d))),
        (None, None, true, Some(d)) => world.spawn((C, D::new(d))),
        (Some(a), Some(b), true, None) => world.spawn((A(a), B(b), C)),
        (Some(a), Some(b), false, Some(d)) => world.spawn((A(a), B(b), D::new(d))),
        (Some(a), None, true, Some(d)) => world.spawn((A(a), C, D::new(d))),
        (None, Some(b), true, Some(d)) => world.spawn((B(b), C, D::new(d))),
        (Some(a), Some(b), true, Some(d)) => world.spawn((A(a), B(b), C, D::new(d))),
    }
}

/// Spawn `spec` incrementally: empty entity, then one `insert_one` per
/// component, migrating through intermediate archetypes.
fn spawn_incremental(world: &mut World, s: Bundle4) -> Entity {
    let e = world.spawn(());
    if let Some(v) = s.a {
        world.insert_one(e, A(v)).expect("insert_one A on just-spawned entity");
    }
    if let Some(v) = s.b {
        world.insert_one(e, B(v)).expect("insert_one B on just-spawned entity");
    }
    if s.c {
        world.insert_one(e, C).expect("insert_one C on just-spawned entity");
    }
    if let Some(v) = s.d {
        world.insert_one(e, D::new(v)).expect("insert_one D on just-spawned entity");
    }
    e
}

/// One drawn mutation, applied identically to every world; returns ok-ness.
#[derive(Clone, Copy, Debug)]
enum Mut {
    InsertOne(u8, i32),
    RemoveOne(u8),
    InsertBundle(Bundle4),
    RemoveAB,
    Despawn,
    ExchangeAToB(i32),
}

fn draw_mut(tc: &hegel::TestCase) -> Mut {
    match tc.draw(gs::integers::<u8>().min_value(0).max_value(5)) {
        0 => Mut::InsertOne(tc.draw(gs::integers::<u8>().min_value(0).max_value(3)), tc.draw(val())),
        1 => Mut::RemoveOne(tc.draw(gs::integers::<u8>().min_value(0).max_value(3))),
        2 => Mut::InsertBundle(draw_bundle(tc)),
        3 => Mut::RemoveAB,
        4 => Mut::Despawn,
        _ => Mut::ExchangeAToB(tc.draw(val())),
    }
}

fn apply_mut(world: &mut World, e: Entity, op: Mut) -> bool {
    match op {
        Mut::InsertOne(which, v) => match which {
            0 => world.insert_one(e, A(v)).is_ok(),
            1 => world.insert_one(e, B(v)).is_ok(),
            2 => world.insert_one(e, C).is_ok(),
            _ => world.insert_one(e, D::new(v)).is_ok(),
        },
        Mut::RemoveOne(which) => match which {
            0 => world.remove_one::<A>(e).is_ok(),
            1 => world.remove_one::<B>(e).is_ok(),
            2 => world.remove_one::<C>(e).is_ok(),
            _ => world.remove_one::<D>(e).is_ok(),
        },
        Mut::InsertBundle(s) => world.insert(e, make_builder(s).build()).is_ok(),
        Mut::RemoveAB => world.remove::<(A, B)>(e).is_ok(),
        Mut::Despawn => world.despawn(e).is_ok(),
        Mut::ExchangeAToB(v) => world.exchange_one::<A, B>(e, B(v)).is_ok(),
    }
}

fn total_d(worlds: &[World]) -> i64 {
    worlds.iter().map(|w| fingerprint_d_count(&fingerprint(w))).sum()
}

fn drive(tc: &hegel::TestCase, max_entities: u32, max_muts: u32) {
    assert_d_balanced_at_start();

    // A drawn shared history (spawns + despawns) puts every allocator into the
    // same non-trivial state (non-empty freelist, stale generations) before
    // the paths diverge.
    let (mut worlds, mut pool) = build_twins(tc, N_PATHS, max_entities);

    // ---- construction phase: same specs through five different paths ----
    let n = tc.draw(gs::integers::<u32>().min_value(0).max_value(max_entities));
    let specs: Vec<Bundle4> = (0..n).map(|_| draw_bundle(tc)).collect();

    let mut cmd = CommandBuffer::new();
    for &s in &specs {
        cmd.spawn(make_builder(s).build());
    }

    let mut handles: Vec<Vec<Entity>> = Vec::with_capacity(N_PATHS);
    // Path 1: EntityBuilder.
    handles.push(specs.iter().map(|&s| worlds[0].spawn(make_builder(s).build())).collect());
    // Path 2: concrete tuple bundles.
    handles.push(specs.iter().map(|&s| spawn_tuple(&mut worlds[1], s)).collect());
    // Path 3: reserve + insert.
    handles.push(
        specs
            .iter()
            .map(|&s| {
                let e = worlds[2].reserve_entity();
                worlds[2]
                    .insert(e, make_builder(s).build())
                    .expect("insert on freshly reserved entity");
                e
            })
            .collect(),
    );
    // Path 4: CommandBuffer (handles are not surfaced; recovered below).
    cmd.run_on(&mut worlds[3]);
    // Path 5: incremental insert_one chain.
    handles.push(specs.iter().map(|&s| spawn_incremental(&mut worlds[4], s)).collect());

    // All handle-surfacing paths must agree exactly.
    for (i, hs) in handles.iter().enumerate().skip(1) {
        assert_eq!(&handles[0], hs, "construction path {} allocated different handles", i);
    }
    for &e in &handles[0] {
        pool.push(e);
    }

    // Ground truth: the path-1 world's fingerprint must contain exactly the
    // drawn spec for every new entity — the five paths can't all agree on a
    // wrong answer.
    let fp0 = fingerprint(&worlds[0]);
    for (e, s) in handles[0].iter().zip(specs.iter()) {
        assert_eq!(fp0.get(e), Some(s), "path-1 world disagrees with spec for {e:?}");
    }
    for (i, w) in worlds.iter().enumerate().skip(1) {
        assert_eq!(fp0, fingerprint(w), "construction path {i} is observationally different");
        check_archetypes(w, "constructed world");
    }
    assert_eq!(d_live(), total_d(&worlds), "Drop imbalance across construction paths");

    // ---- mutation phase: construction history must not be observable ----
    let muts = tc.draw(gs::integers::<u32>().min_value(0).max_value(max_muts));
    for _ in 0..muts {
        let Some(e) = pick(tc, &pool) else { break };
        let op = draw_mut(tc);
        let mut results = worlds.iter_mut().map(|w| apply_mut(w, e, op));
        let first = results.next().expect("five worlds");
        let rest: Vec<bool> = results.collect();
        for (i, r) in rest.iter().enumerate() {
            assert_eq!(
                first,
                *r,
                "mutation {op:?} on {e:?} gave a different result in path-{} world",
                i + 2
            );
        }
        let fp = fingerprint(&worlds[0]);
        for (i, w) in worlds.iter().enumerate().skip(1) {
            assert_eq!(fp, fingerprint(w), "worlds diverged after {op:?} on {e:?} (path {i})");
        }
    }
    assert_eq!(d_live(), total_d(&worlds), "Drop imbalance after mutation phase");
}

#[cfg(not(miri))]
#[hegel::test(test_cases = 400)]
fn construction_paths_agree(tc: hegel::TestCase) {
    drive(&tc, 5, 10);
}

#[cfg(miri)]
#[hegel::test(test_cases = 8, suppress_health_check = [hegel::HealthCheck::TooSlow])]
fn construction_paths_agree(tc: hegel::TestCase) {
    drive(&tc, 3, 5);
}
