//! Stateful model-based property test for `hecs::ChangeTracker`.
//!
//! Semantics under test (read from hecs-0.11.0 `src/change_tracker.rs`):
//! `ChangeTracker<T>` stores, per tracked entity, a private `Previous<T>` component in
//! the world holding the value as of the most recent `track` call. `track(&mut world)`
//! returns a `Changes` handle whose three iterators mean, relative to the previous
//! `track` call:
//!   * `added()`   — entities that have `T` but no `Previous<T>` snapshot (gained `T`
//!     since the last poll, including newly spawned entities); yields `(Entity, &T)`.
//!   * `changed()` — entities with both `T` and a snapshot whose current value differs
//!     from the snapshot *by `PartialEq`* (NOT by `&mut` access: writing an equal value
//!     through `&mut` is not a change); yields `(Entity, old, &new)`.
//!   * `removed()` — live entities with a snapshot but no `T` (lost `T` since the last
//!     poll but were NOT despawned; despawned entities are never reported); yields
//!     `(Entity, old)`.
//! Iterators not consumed by the caller are drained when `Changes` drops, so the
//! snapshot state always advances to the current state at the end of each poll,
//! regardless of which iterators were called or how far they were driven.
//!
//! Consequences the model encodes exactly:
//!   * remove `T` then re-insert a *different* value between polls => `changed`, not
//!     removed+added;
//!   * remove then re-insert an *equal* value => nothing reported;
//!   * gain `T` and despawn between polls => nothing reported;
//!   * despawn an entity that had a snapshot => NOT in `removed`.
//!
//! Model: `HashMap<Entity, M>` where `M.cur` is the entity's current `T` payload and
//! `M.prev` is the tracker's snapshot payload; `M.b` mirrors an untracked component used
//! to force archetype migrations (the `Previous<T>` snapshot must survive them). At each
//! poll we compute the expected added/changed/removed sets from (cur, prev), call the
//! three iterators in a drawn order (or partially / not at all, exercising the
//! drain-on-drop paths), assert exact equality including old/new payloads and
//! `ExactSizeIterator::len`, then advance `prev := cur` for every entity.
//!
//! Leak oracle: the tracked component `Tr` is non-`Copy`; its constructor/`Clone` bump a
//! thread-local live-count and `Drop` decrements it. Between polls the only live `Tr`
//! values are the world's `T` components plus the tracker's `Previous<T>` snapshots, so
//! we assert `live == #cur + #prev` after every operation and every poll. This catches
//! leaked or double-dropped snapshots inside the tracker machinery.

use std::cell::Cell;
use std::collections::HashMap;

use hecs::{ChangeTracker, Changes, Entity, EntityBuilder, World};
use hegel::generators as gs;

// ---- tracked component: non-Copy, Drop/Clone-counted ----
thread_local! { static TR_LIVE: Cell<i64> = const { Cell::new(0) }; }

#[derive(Debug)]
struct Tr(i32);
impl Tr {
    fn new(v: i32) -> Tr {
        TR_LIVE.with(|c| c.set(c.get() + 1));
        Tr(v)
    }
}
impl Clone for Tr {
    fn clone(&self) -> Tr {
        Tr::new(self.0)
    }
}
impl PartialEq for Tr {
    fn eq(&self, other: &Tr) -> bool {
        self.0 == other.0
    }
}
impl Drop for Tr {
    fn drop(&mut self) {
        TR_LIVE.with(|c| c.set(c.get() - 1));
    }
}
fn tr_live() -> i64 {
    TR_LIVE.with(|c| c.get())
}

/// Untracked component, used to force archetype migrations under the tracker's feet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct B(i32);

// ---- reference model of one entity ----
#[derive(Clone, Copy, Debug, Default)]
struct M {
    /// Current `Tr` payload in the world (None = entity has no `Tr`).
    cur: Option<i32>,
    /// The tracker's snapshot payload (None = tracker holds no `Previous<Tr>`).
    prev: Option<i32>,
    /// Current `B` payload.
    b: Option<i32>,
}

fn val() -> impl gs::Generator<i32> {
    gs::integers::<i32>().min_value(-3).max_value(3)
}

/// Draw an index into `pool` (which may include stale/despawned handles), or None if empty.
fn pick(tc: &hegel::TestCase, pool: &[Entity]) -> Option<Entity> {
    if pool.is_empty() {
        return None;
    }
    let i = tc.draw(gs::integers::<usize>().min_value(0).max_value(pool.len() - 1));
    Some(pool[i])
}

/// Live `Tr` instances must be exactly: world components (cur) + tracker snapshots (prev).
fn check_live(model: &HashMap<Entity, M>) {
    let expect = model.values().filter(|m| m.cur.is_some()).count()
        + model.values().filter(|m| m.prev.is_some()).count();
    assert_eq!(
        tr_live(),
        expect as i64,
        "live Tr count != #cur + #prev (leak or double-drop in tracker/world)"
    );
}

/// World/model equivalence for the user-visible components, plus the leak oracle.
fn check_state(world: &World, model: &HashMap<Entity, M>) {
    assert_eq!(world.len() as usize, model.len(), "world.len() != model.len()");
    for (&e, m) in model {
        assert!(world.contains(e), "world missing modelled entity {:?}", e);
        assert_eq!(world.get::<&Tr>(e).ok().map(|t| t.0), m.cur, "Tr for {:?}", e);
        assert_eq!(world.get::<&B>(e).ok().map(|b| b.0), m.b, "B for {:?}", e);
    }
    for eref in world.iter() {
        let e = eref.entity();
        assert!(model.contains_key(&e), "world has un-modelled entity {:?}", e);
    }
    check_live(model);
}

/// Expected (added, changed, removed) for the next poll, derived purely from the model.
#[allow(clippy::type_complexity)]
fn expected_sets(
    model: &HashMap<Entity, M>,
) -> (
    HashMap<Entity, i32>,
    HashMap<Entity, (i32, i32)>,
    HashMap<Entity, i32>,
) {
    let mut added = HashMap::new();
    let mut changed = HashMap::new();
    let mut removed = HashMap::new();
    for (&e, m) in model {
        match (m.cur, m.prev) {
            (Some(v), None) => {
                added.insert(e, v);
            }
            (Some(new), Some(old)) if new != old => {
                changed.insert(e, (old, new));
            }
            (None, Some(old)) => {
                removed.insert(e, old);
            }
            _ => {}
        }
    }
    (added, changed, removed)
}

fn check_added(changes: &mut Changes<'_, Tr>, exp: &HashMap<Entity, i32>) {
    let it = changes.added();
    assert_eq!(it.len(), exp.len(), "added().len()");
    let mut got: HashMap<Entity, i32> = HashMap::new();
    for (e, t) in it {
        assert!(got.insert(e, t.0).is_none(), "added() yielded {:?} twice", e);
    }
    assert_eq!(got, *exp, "added() set/values");
}

fn check_changed(changes: &mut Changes<'_, Tr>, exp: &HashMap<Entity, (i32, i32)>) {
    let mut got: HashMap<Entity, (i32, i32)> = HashMap::new();
    for (e, old, new) in changes.changed() {
        assert!(
            got.insert(e, (old.0, new.0)).is_none(),
            "changed() yielded {:?} twice",
            e
        );
    }
    assert_eq!(got, *exp, "changed() set/(old,new) values");
}

fn check_removed(changes: &mut Changes<'_, Tr>, exp: &HashMap<Entity, i32>) {
    let it = changes.removed();
    assert_eq!(it.len(), exp.len(), "removed().len()");
    let mut got: HashMap<Entity, i32> = HashMap::new();
    for (e, old) in it {
        assert!(got.insert(e, old.0).is_none(), "removed() yielded {:?} twice", e);
    }
    assert_eq!(got, *exp, "removed() set/old values");
}

/// Poll the tracker, asserting its reports against the model, then advance the model's
/// snapshot. `mode` 0..=5 call all three iterators in each of the six orders; 6 only
/// partially consumes `added()` (drain-on-drop must still finish it) while fully
/// checking the other two; 7 drops `Changes` without calling anything (pure Drop path —
/// no content assertions possible, but the snapshot must still advance, which the next
/// poll and the live-count oracle verify).
fn poll(
    tracker: &mut ChangeTracker<Tr>,
    world: &mut World,
    model: &mut HashMap<Entity, M>,
    mode: u8,
) {
    let (added_exp, changed_exp, removed_exp) = expected_sets(model);
    {
        let mut changes = tracker.track(world);
        match mode {
            0 => {
                check_added(&mut changes, &added_exp);
                check_changed(&mut changes, &changed_exp);
                check_removed(&mut changes, &removed_exp);
            }
            1 => {
                check_added(&mut changes, &added_exp);
                check_removed(&mut changes, &removed_exp);
                check_changed(&mut changes, &changed_exp);
            }
            2 => {
                check_changed(&mut changes, &changed_exp);
                check_added(&mut changes, &added_exp);
                check_removed(&mut changes, &removed_exp);
            }
            3 => {
                check_changed(&mut changes, &changed_exp);
                check_removed(&mut changes, &removed_exp);
                check_added(&mut changes, &added_exp);
            }
            4 => {
                check_removed(&mut changes, &removed_exp);
                check_added(&mut changes, &added_exp);
                check_changed(&mut changes, &changed_exp);
            }
            5 => {
                check_removed(&mut changes, &removed_exp);
                check_changed(&mut changes, &changed_exp);
                check_added(&mut changes, &added_exp);
            }
            6 => {
                {
                    let mut it = changes.added();
                    assert_eq!(it.len(), added_exp.len(), "added().len() (partial mode)");
                    if let Some((e, t)) = it.next() {
                        assert_eq!(
                            added_exp.get(&e),
                            Some(&t.0),
                            "added() first element (partial mode)"
                        );
                    }
                    // dropped part-way: DrainOnDrop must visit the rest so their
                    // snapshots are still recorded
                }
                check_changed(&mut changes, &changed_exp);
                check_removed(&mut changes, &removed_exp);
            }
            _ => {
                // drop `changes` without touching any iterator
            }
        }
    }
    // Whatever was (or wasn't) consumed, the snapshot is now the current state.
    for m in model.values_mut() {
        m.prev = m.cur;
    }
    check_live(model);
}

/// Core harness: apply drawn operations to World + model, polling the tracker along the
/// way and after the final step.
fn drive(tc: &hegel::TestCase, max_steps: u32) {
    assert_eq!(tr_live(), 0, "Tr live count nonzero at test-case start");
    let mut world = World::new();
    let mut tracker = ChangeTracker::<Tr>::new();
    let mut model: HashMap<Entity, M> = HashMap::new();
    let mut known: Vec<Entity> = Vec::new(); // includes stale/despawned handles

    let steps = tc.draw(gs::integers::<u32>().min_value(0).max_value(max_steps));
    for _ in 0..steps {
        match tc.draw(gs::integers::<u8>().min_value(0).max_value(15)) {
            // spawn with an arbitrary subset of {Tr, B}
            0 | 1 => {
                let trv = tc.draw(gs::optional(val()));
                let bv = tc.draw(gs::optional(val()));
                let mut builder = EntityBuilder::new();
                if let Some(v) = trv {
                    builder.add(Tr::new(v));
                }
                if let Some(v) = bv {
                    builder.add(B(v));
                }
                let e = world.spawn(builder.build());
                model.insert(
                    e,
                    M {
                        cur: trv,
                        prev: None,
                        b: bv,
                    },
                );
                known.push(e);
            }
            // despawn a (possibly stale) handle; drops Tr AND its Previous snapshot,
            // and must NOT show up in removed() at the next poll
            2 => {
                if let Some(e) = pick(tc, &known) {
                    let ok = world.despawn(e).is_ok();
                    let existed = model.remove(&e).is_some();
                    assert_eq!(ok, existed, "despawn ok-ness disagrees for {:?}", e);
                }
            }
            // insert Tr (fresh add, or overwrite of an existing Tr)
            3 | 4 => {
                if let Some(e) = pick(tc, &known) {
                    let v = tc.draw(val());
                    let live = model.contains_key(&e);
                    // on a dead target insert_one drops the value again (net 0)
                    let ok = world.insert_one(e, Tr::new(v)).is_ok();
                    assert_eq!(ok, live, "insert Tr ok-ness for {:?}", e);
                    if let Some(m) = model.get_mut(&e) {
                        m.cur = Some(v);
                    }
                }
            }
            // remove Tr (the returned value drops here); snapshot stays until next poll
            5 => {
                if let Some(e) = pick(tc, &known) {
                    let ok = world.remove_one::<Tr>(e).is_ok();
                    let had = model.get(&e).map(|m| m.cur.is_some()).unwrap_or(false);
                    assert_eq!(ok, had, "remove Tr ok-ness for {:?}", e);
                    if let Some(m) = model.get_mut(&e) {
                        m.cur = None;
                    }
                }
            }
            // mutate Tr in place via &mut (may coincidentally write an equal value,
            // which must NOT count as changed)
            6 | 7 => {
                if let Some(e) = pick(tc, &known) {
                    let v = tc.draw(val());
                    let had = model.get(&e).map(|m| m.cur.is_some()).unwrap_or(false);
                    match world.get::<&mut Tr>(e) {
                        Ok(mut t) => {
                            assert!(had, "&mut Tr succeeded for {:?} but model has none", e);
                            t.0 = v;
                        }
                        Err(_) => assert!(!had, "&mut Tr failed for {:?} but model has one", e),
                    }
                    if had {
                        model.get_mut(&e).unwrap().cur = Some(v);
                    }
                }
            }
            // deliberately write the IDENTICAL value through &mut: &mut access alone is
            // not a change under ChangeTracker's PartialEq semantics
            8 => {
                if let Some(e) = pick(tc, &known) {
                    if let Some(v0) = model.get(&e).and_then(|m| m.cur) {
                        world
                            .get::<&mut Tr>(e)
                            .expect("model says Tr present")
                            .0 = v0;
                        // model unchanged on purpose
                    }
                }
            }
            // insert/remove the untracked B: archetype migration under the tracker;
            // the private Previous<Tr> snapshot must migrate intact
            9 => {
                if let Some(e) = pick(tc, &known) {
                    let v = tc.draw(val());
                    let live = model.contains_key(&e);
                    let ok = world.insert_one(e, B(v)).is_ok();
                    assert_eq!(ok, live, "insert B ok-ness for {:?}", e);
                    if let Some(m) = model.get_mut(&e) {
                        m.b = Some(v);
                    }
                }
            }
            10 => {
                if let Some(e) = pick(tc, &known) {
                    let ok = world.remove_one::<B>(e).is_ok();
                    let had = model.get(&e).map(|m| m.b.is_some()).unwrap_or(false);
                    assert_eq!(ok, had, "remove B ok-ness for {:?}", e);
                    if let Some(m) = model.get_mut(&e) {
                        m.b = None;
                    }
                }
            }
            // clear the whole world: components AND snapshots all drop; tracker
            // effectively resets (everything re-reported as added later)
            11 => {
                world.clear();
                model.clear();
            }
            // spawn a homogeneous batch of (Tr, B)
            12 => {
                let n = tc.draw(gs::integers::<u32>().min_value(0).max_value(4));
                let v = tc.draw(val());
                let ents: Vec<Entity> = world
                    .spawn_batch((0..n).map(|_| (Tr::new(v), B(v))))
                    .collect();
                for e in ents {
                    model.insert(
                        e,
                        M {
                            cur: Some(v),
                            prev: None,
                            b: Some(v),
                        },
                    );
                    known.push(e);
                }
            }
            // poll the tracker (~3/16 of steps), in a drawn consumption mode
            _ => {
                let mode = tc.draw(gs::integers::<u8>().min_value(0).max_value(7));
                poll(&mut tracker, &mut world, &mut model, mode);
            }
        }
        check_state(&world, &model);
    }

    // Always finish with a full-assertion poll so every case checks the tracker at least once.
    let mode = tc.draw(gs::integers::<u8>().min_value(0).max_value(5));
    poll(&mut tracker, &mut world, &mut model, mode);
    check_state(&world, &model);
}

#[cfg(not(miri))]
#[hegel::test(test_cases = 500)]
fn change_tracker_matches_model(tc: hegel::TestCase) {
    drive(&tc, 200);
}

#[cfg(miri)]
#[hegel::test(test_cases = 12)]
fn change_tracker_matches_model(tc: hegel::TestCase) {
    drive(&tc, 25);
}
