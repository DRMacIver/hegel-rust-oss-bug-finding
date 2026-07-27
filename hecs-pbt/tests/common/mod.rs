//! Shared infrastructure for the model-free oracle test binaries
//! (`metamorphic.rs`, `differential_paths.rs`).
//!
//! The core reusable idea is the **observational fingerprint**: a canonical,
//! plain-data snapshot of everything a caller can observe about a `World`
//! (the exact entity handles and, per entity, the exact component set and
//! values). Two worlds are "observationally equivalent" iff their fingerprints
//! are equal. Metamorphic relations and differential construction paths both
//! reduce to fingerprint comparisons — no reference model needed.
//!
//! Each integration-test binary compiles this module independently and uses a
//! subset of it, so some items are dead code in some binaries.
#![allow(dead_code)]

use hecs::{Entity, EntityBuilder, World};
use hegel::generators as gs;
use std::cell::Cell;
use std::collections::BTreeMap;

// ---- fixed component universe (same as the core harness) ----
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct A(pub i32);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct B(pub i32);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct C; // zero-sized marker

// Drop-tracked component: NOT Copy; bumps a per-thread live-count on new/drop.
thread_local! { static D_LIVE: Cell<i64> = const { Cell::new(0) }; }
#[derive(Debug)]
pub struct D(pub i32);
impl D {
    pub fn new(v: i32) -> D {
        D_LIVE.with(|c| c.set(c.get() + 1));
        D(v)
    }
}
impl Drop for D {
    fn drop(&mut self) {
        D_LIVE.with(|c| c.set(c.get() - 1));
    }
}
pub fn d_live() -> i64 {
    D_LIVE.with(|c| c.get())
}

/// Thread-locals persist across hegel's many test cases: any imbalance at
/// case start is a leak/double-drop that escaped a previous case.
pub fn assert_d_balanced_at_start() {
    assert_eq!(d_live(), 0, "D live-count nonzero at case start (escaped a previous case)");
}

pub fn val() -> impl gs::Generator<i32> {
    gs::integers::<i32>().min_value(-3).max_value(3)
}

/// Draw an index into `pool` (which may include stale/despawned handles), or None if empty.
pub fn pick(tc: &hegel::TestCase, pool: &[Entity]) -> Option<Entity> {
    if pool.is_empty() {
        return None;
    }
    let i = tc.draw(gs::integers::<usize>().min_value(0).max_value(pool.len() - 1));
    Some(pool[i])
}

/// An arbitrary subset of the component universe, as drawn values.
/// `D` instances are only materialized (`D::new`) when a bundle is built, so
/// the Drop oracle sees exactly the instances that entered a world.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bundle4 {
    pub a: Option<i32>,
    pub b: Option<i32>,
    pub c: bool,
    pub d: Option<i32>,
}

pub fn draw_bundle(tc: &hegel::TestCase) -> Bundle4 {
    Bundle4 {
        a: tc.draw(gs::optional(val())),
        b: tc.draw(gs::optional(val())),
        c: tc.draw(gs::booleans()),
        d: tc.draw(gs::optional(val())),
    }
}

pub fn make_builder(s: Bundle4) -> EntityBuilder {
    let mut b = EntityBuilder::new();
    if let Some(v) = s.a {
        b.add(A(v));
    }
    if let Some(v) = s.b {
        b.add(B(v));
    }
    if s.c {
        b.add(C);
    }
    if let Some(v) = s.d {
        b.add(D::new(v));
    }
    b
}

// ---- observational fingerprint ----

/// Everything observable about one entity: exact component set and values.
pub type Obs = Bundle4;

/// Canonical snapshot of a `World`'s observable contents, keyed by exact
/// `Entity` handle (id AND generation). Plain data: compare with `==`, edit
/// to build "expected" values for metamorphic relations with adjustment.
pub type Fingerprint = BTreeMap<Entity, Obs>;

pub fn fingerprint(world: &World) -> Fingerprint {
    let mut fp = Fingerprint::new();
    for eref in world.iter() {
        let obs = Obs {
            a: eref.get::<&A>().map(|r| r.0),
            b: eref.get::<&B>().map(|r| r.0),
            c: eref.get::<&C>().is_some(),
            d: eref.get::<&D>().map(|r| r.0),
        };
        assert!(
            fp.insert(eref.entity(), obs).is_none(),
            "world.iter() yielded {:?} twice",
            eref.entity()
        );
    }
    assert_eq!(fp.len() as u32, world.len(), "iter() count != world.len()");
    fp
}

/// Number of live `D` components a fingerprint accounts for.
pub fn fingerprint_d_count(fp: &Fingerprint) -> i64 {
    fp.values().filter(|o| o.d.is_some()).count() as i64
}

/// Structural invariant independent of any model: the archetypes partition
/// the live entities (each id exactly once, lengths sum to world.len()).
pub fn check_archetypes(world: &World, label: &str) {
    let mut total = 0u32;
    let mut ids = std::collections::HashSet::new();
    for arch in world.archetypes() {
        total += arch.len();
        for &id in arch.ids() {
            assert!(ids.insert(id), "{label}: entity id {id} in >1 archetype");
        }
    }
    assert_eq!(total, world.len(), "{label}: sum of archetype lens != world.len()");
}

// ---- twin-world construction ----
//
// hecs entity allocation is deterministic: two fresh Worlds that undergo the
// same logical sequence of spawns/despawns hand out identical handles (the
// assert below would fail loudly if that ever stopped holding). Twin worlds
// let us compare `f(g(w))` against `g(f(w))` without `World: Clone`.

/// Build `n_worlds` observationally identical worlds by replaying one drawn
/// history (spawns of arbitrary bundles, then a drawn subset of despawns)
/// into each. Returns the worlds plus a target pool that keeps stale handles.
pub fn build_twins(
    tc: &hegel::TestCase,
    n_worlds: usize,
    max_entities: u32,
) -> (Vec<World>, Vec<Entity>) {
    let mut worlds: Vec<World> = (0..n_worlds).map(|_| World::new()).collect();
    let mut pool: Vec<Entity> = Vec::new();

    let n = tc.draw(gs::integers::<u32>().min_value(0).max_value(max_entities));
    for _ in 0..n {
        let s = draw_bundle(tc);
        let mut handles = worlds.iter_mut().map(|w| w.spawn(make_builder(s).build()));
        let first = handles.next().expect("at least one world");
        for h in handles {
            assert_eq!(first, h, "deterministic handle allocation violated in twin setup");
        }
        pool.push(first);
    }
    // Despawn a drawn subset in every world, so pools contain stale handles
    // and freelists match.
    for i in 0..pool.len() {
        if tc.draw(gs::booleans()) {
            let e = pool[i];
            let mut oks = worlds.iter_mut().map(|w| w.despawn(e).is_ok());
            let first = oks.next().expect("at least one world");
            for ok in oks {
                assert_eq!(first, ok, "twin despawn ok-ness diverged for {e:?}");
            }
        }
    }
    let fp0 = fingerprint(&worlds[0]);
    for (i, w) in worlds.iter().enumerate().skip(1) {
        assert_eq!(fp0, fingerprint(w), "twin world {i} diverged after setup");
    }
    (worlds, pool)
}
