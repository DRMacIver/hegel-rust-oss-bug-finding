//! Property tests for `hecs::CommandBuffer` (hecs 0.11.0).
//!
//! Oracle design
//! =============
//! A `CommandBuffer` records operations (`spawn`, `insert`/`insert_one`,
//! `remove`/`remove_one`, `despawn`) and replays them in recorded order with
//! `run_on(&mut World)`. Its documented contract is that replaying is
//! equivalent to applying the same operations directly and ignoring failures:
//!   * buffered `insert(e, bundle)`  ==  `let _ = world.insert(e, bundle)`
//!     (components quietly dropped if `e` is dead),
//!   * buffered `remove::<T>(e)`     ==  `let _ = world.remove::<T>(e)`,
//!   * buffered `despawn(e)`         ==  `let _ = world.despawn(e)`,
//!   * buffered `spawn(bundle)`      ==  `world.spawn(bundle)`.
//!
//! We keep TWO worlds:
//!   * `direct`:   each drawn operation is applied eagerly via World methods;
//!   * `buffered`: the same operations are recorded into a CommandBuffer and
//!     applied in a single `run_on` at the end of the round.
//!
//! Comparison soundness: hecs entity-id allocation is deterministic — two
//! fresh Worlds that undergo the same logical sequence of spawn / despawn /
//! reserve operations hand out identical `Entity` handles (sequential ids plus
//! a deterministic freelist). The setup phase mirrors identical spawns,
//! despawns and reserves into both worlds and *asserts* the returned handles
//! are equal, so both allocators enter each round in identical states. Every
//! round applies the same operation sequence, in the same order, to both
//! worlds (eagerly vs. via `run_on`), so even `CommandBuffer::spawn` — whose
//! handle is never surfaced to the caller — must allocate the same handle as
//! the direct `world.spawn` at the same sequence position. That makes an
//! EXACT comparison well-defined and strong: identical entity sets, and on
//! every entity identical component sets with identical values. (If handle
//! assignment ever diverged, the comparison would fail loudly — which would
//! itself be a divergence between `run_on` and direct application.)
//!
//! Additional invariants checked every round:
//!   * `run_on` clears the buffer: an immediate second `run_on` is a no-op;
//!     the buffer is reused across rounds (reusability).
//!   * `clear()` discards recorded commands: a subsequent `run_on` is a no-op.
//!   * Dropping a non-empty buffer releases its stored components.
//!   * Drop oracle: `D` is a non-Copy component whose constructor/Drop bump a
//!     thread-local live counter. While operations are buffered we assert
//!     `live == D_in(direct) + D_in(buffered) + D_pending_in_buffer`, and after
//!     the buffer is consumed `live == D_in(direct) + D_in(buffered)`. This
//!     catches leaks / double-drops / premature drops in the buffer's raw
//!     component storage (`add_inner` / `RecordedEntity` / `clear`).
//!   * Archetypes of both worlds partition their live entities exactly once.
//!
//! Error paths are exercised on purpose: the target pool keeps stale
//! (despawned) handles, so buffered inserts/removes/despawns against dead
//! entities are common; setup also reserves entities via `reserve_entity`
//! (the documented companion to `CommandBuffer::insert`).

use hecs::{CommandBuffer, Entity, EntityBuilder, World};
use hegel::generators as gs;
use std::cell::Cell;
use std::collections::HashSet;

// ---- fixed component universe ----
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct A(i32);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct B(i32);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct C; // zero-sized marker

// Drop-tracked component: NOT Copy; bumps a per-thread live-count on new/drop.
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
fn d_count(world: &World) -> i64 {
    world.query::<&D>().iter().count() as i64
}

fn val() -> impl gs::Generator<i32> {
    gs::integers::<i32>().min_value(-3).max_value(3)
}

fn which() -> impl gs::Generator<u8> {
    gs::integers::<u8>().min_value(0).max_value(3)
}

/// Draw an index into `pool` (which may include stale/despawned handles), or None if empty.
fn pick(tc: &hegel::TestCase, pool: &[Entity]) -> Option<Entity> {
    if pool.is_empty() {
        return None;
    }
    let i = tc.draw(gs::integers::<usize>().min_value(0).max_value(pool.len() - 1));
    Some(pool[i])
}

/// An arbitrary subset of the component universe, as drawn values.
/// `D` instances are only materialized (D::new) at record/replay time so the
/// Drop oracle sees exactly the instances that entered the buffer or a world.
#[derive(Clone, Copy, Debug)]
struct Bundle4 {
    a: Option<i32>,
    b: Option<i32>,
    c: bool,
    d: Option<i32>,
}

fn draw_bundle(tc: &hegel::TestCase) -> Bundle4 {
    Bundle4 {
        a: tc.draw(gs::optional(val())),
        b: tc.draw(gs::optional(val())),
        c: tc.draw(gs::booleans()),
        d: tc.draw(gs::optional(val())),
    }
}

fn make_builder(s: Bundle4) -> EntityBuilder {
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

/// One logical operation, applied both via CommandBuffer and directly.
#[derive(Clone, Copy, Debug)]
enum Op {
    Spawn(Bundle4),
    Insert(Entity, Bundle4),
    InsertOne(Entity, u8, i32),
    RemoveAB(Entity),
    RemoveCD(Entity),
    RemoveOne(Entity, u8),
    Despawn(Entity),
}

fn record(cmd: &mut CommandBuffer, op: Op) {
    match op {
        Op::Spawn(s) => cmd.spawn(make_builder(s).build()),
        Op::Insert(e, s) => cmd.insert(e, make_builder(s).build()),
        Op::InsertOne(e, w, v) => match w {
            0 => cmd.insert_one(e, A(v)),
            1 => cmd.insert_one(e, B(v)),
            2 => cmd.insert_one(e, C),
            _ => cmd.insert_one(e, D::new(v)),
        },
        Op::RemoveAB(e) => cmd.remove::<(A, B)>(e),
        Op::RemoveCD(e) => cmd.remove::<(C, D)>(e),
        Op::RemoveOne(e, w) => match w {
            0 => cmd.remove_one::<A>(e),
            1 => cmd.remove_one::<B>(e),
            2 => cmd.remove_one::<C>(e),
            _ => cmd.remove_one::<D>(e),
        },
        Op::Despawn(e) => cmd.despawn(e),
    }
}

/// The direct/eager equivalent of the buffered semantics: apply each op via
/// World methods, ignoring failures exactly as `run_on` does.
fn replay(world: &mut World, ops: &[Op]) {
    for &op in ops {
        match op {
            Op::Spawn(s) => {
                world.spawn(make_builder(s).build());
            }
            Op::Insert(e, s) => {
                let _ = world.insert(e, make_builder(s).build());
            }
            Op::InsertOne(e, w, v) => match w {
                0 => {
                    let _ = world.insert_one(e, A(v));
                }
                1 => {
                    let _ = world.insert_one(e, B(v));
                }
                2 => {
                    let _ = world.insert_one(e, C);
                }
                _ => {
                    let _ = world.insert_one(e, D::new(v));
                }
            },
            Op::RemoveAB(e) => {
                let _ = world.remove::<(A, B)>(e);
            }
            Op::RemoveCD(e) => {
                let _ = world.remove::<(C, D)>(e);
            }
            Op::RemoveOne(e, w) => match w {
                0 => {
                    let _ = world.remove_one::<A>(e);
                }
                1 => {
                    let _ = world.remove_one::<B>(e);
                }
                2 => {
                    let _ = world.remove_one::<C>(e);
                }
                _ => {
                    let _ = world.remove_one::<D>(e);
                }
            },
            Op::Despawn(e) => {
                let _ = world.despawn(e);
            }
        }
    }
}

/// How many `D` instances an op sequence has parked inside the buffer.
fn pending_d(ops: &[Op]) -> i64 {
    ops.iter()
        .map(|op| match op {
            Op::Spawn(s) | Op::Insert(_, s) => s.d.is_some() as i64,
            Op::InsertOne(_, 3, _) => 1,
            _ => 0,
        })
        .sum()
}

fn check_archetypes(world: &World, label: &str) {
    let mut total = 0u32;
    let mut ids: HashSet<u32> = HashSet::new();
    for arch in world.archetypes() {
        total += arch.len();
        for &id in arch.ids() {
            assert!(ids.insert(id), "{label}: entity id {id} in >1 archetype");
        }
    }
    assert_eq!(total, world.len(), "{label}: sum of archetype lens != world.len()");
}

/// Exact equivalence: same entity handles, same components, same values.
fn compare(direct: &World, buffered: &World) {
    assert_eq!(direct.len(), buffered.len(), "world lengths differ");
    for eref in direct.iter() {
        let e = eref.entity();
        assert!(buffered.contains(e), "buffered world missing {e:?}");
        assert_eq!(
            direct.get::<&A>(e).ok().map(|r| r.0),
            buffered.get::<&A>(e).ok().map(|r| r.0),
            "A differs for {e:?}"
        );
        assert_eq!(
            direct.get::<&B>(e).ok().map(|r| r.0),
            buffered.get::<&B>(e).ok().map(|r| r.0),
            "B differs for {e:?}"
        );
        assert_eq!(
            direct.get::<&C>(e).is_ok(),
            buffered.get::<&C>(e).is_ok(),
            "C differs for {e:?}"
        );
        assert_eq!(
            direct.get::<&D>(e).ok().map(|r| r.0),
            buffered.get::<&D>(e).ok().map(|r| r.0),
            "D differs for {e:?}"
        );
    }
    for eref in buffered.iter() {
        let e = eref.entity();
        assert!(direct.contains(e), "direct world missing {e:?}");
    }
    check_archetypes(direct, "direct");
    check_archetypes(buffered, "buffered");
}

fn drive(tc: &hegel::TestCase, max_rounds: u32, max_ops: u32) {
    // Thread-locals persist across hegel's many cases: any imbalance here is a
    // leak/double-drop escaped from a previous case. Assert, then reset.
    assert_eq!(d_live(), 0, "D live-count nonzero at case start (leaked from a previous case)");
    D_LIVE.with(|c| c.set(0));

    let mut direct = World::new();
    let mut buffered = World::new();
    let mut pool: Vec<Entity> = Vec::new(); // targets, stale handles included
    let mut known: HashSet<Entity> = HashSet::new();

    // ---- setup: mirror identical spawns/despawns/reserves into both worlds ----
    let n = tc.draw(gs::integers::<u32>().min_value(0).max_value(6));
    for _ in 0..n {
        let s = draw_bundle(tc);
        let ed = direct.spawn(make_builder(s).build());
        let eb = buffered.spawn(make_builder(s).build());
        assert_eq!(ed, eb, "deterministic handle allocation violated in setup");
        pool.push(ed);
        known.insert(ed);
    }
    // Despawn a drawn subset (both worlds) so the pool contains stale handles
    // and the entity freelists hold matching entries.
    for i in 0..pool.len() {
        if tc.draw(gs::booleans()) {
            let e = pool[i];
            assert_eq!(direct.despawn(e).is_ok(), buffered.despawn(e).is_ok());
        }
    }
    // Sometimes reserve entities up front — the documented companion pattern
    // for CommandBuffer::insert ("spawn entities with a known handle").
    if tc.draw(gs::booleans()) {
        let k = tc.draw(gs::integers::<u32>().min_value(1).max_value(3));
        for _ in 0..k {
            let ed = direct.reserve_entity();
            let eb = buffered.reserve_entity();
            assert_eq!(ed, eb, "reserve_entity determinism violated in setup");
            pool.push(ed);
            known.insert(ed);
        }
    }
    compare(&direct, &buffered);

    // ---- rounds: record a batch, then run_on / clear / drop the buffer ----
    let mut cmd = CommandBuffer::new();
    let rounds = tc.draw(gs::integers::<u32>().min_value(1).max_value(max_rounds));
    for _ in 0..rounds {
        let mut ops: Vec<Op> = Vec::new();
        let nops = tc.draw(gs::integers::<u32>().min_value(0).max_value(max_ops));
        for _ in 0..nops {
            let op = match tc.draw(gs::integers::<u8>().min_value(0).max_value(7)) {
                0 | 1 => Op::Spawn(draw_bundle(tc)),
                2 => match pick(tc, &pool) {
                    Some(e) => Op::Insert(e, draw_bundle(tc)),
                    None => Op::Spawn(draw_bundle(tc)),
                },
                3 => match pick(tc, &pool) {
                    Some(e) => Op::InsertOne(e, tc.draw(which()), tc.draw(val())),
                    None => continue,
                },
                4 => match pick(tc, &pool) {
                    Some(e) => Op::RemoveAB(e),
                    None => continue,
                },
                5 => match pick(tc, &pool) {
                    Some(e) => Op::RemoveCD(e),
                    None => continue,
                },
                6 => match pick(tc, &pool) {
                    Some(e) => Op::RemoveOne(e, tc.draw(which())),
                    None => continue,
                },
                _ => match pick(tc, &pool) {
                    Some(e) => Op::Despawn(e),
                    None => continue,
                },
            };
            record(&mut cmd, op);
            ops.push(op);
        }

        // While buffered: every D recorded must be alive inside the buffer,
        // and nothing may have leaked or been dropped early.
        assert_eq!(
            d_live(),
            d_count(&direct) + d_count(&buffered) + pending_d(&ops),
            "D live-count wrong while ops are buffered (early drop or leak in buffer storage)"
        );
        // Recording must not touch the world.
        compare(&direct, &buffered);

        match tc.draw(gs::integers::<u8>().min_value(0).max_value(3)) {
            // Apply: run_on must equal eager application of the same sequence.
            0 | 1 => {
                cmd.run_on(&mut buffered);
                replay(&mut direct, &ops);
                compare(&direct, &buffered);
                // run_on cleared the buffer, so a second run_on is a no-op
                // and the buffer is reusable.
                cmd.run_on(&mut buffered);
                compare(&direct, &buffered);
            }
            // Discard via clear(): both worlds stay unchanged, and the
            // cleared buffer applies as a no-op.
            2 => {
                cmd.clear();
                compare(&direct, &buffered);
                cmd.run_on(&mut buffered);
                compare(&direct, &buffered);
            }
            // Discard by dropping the (possibly non-empty) buffer: its stored
            // components must be dropped exactly once.
            _ => {
                cmd = CommandBuffer::new();
                compare(&direct, &buffered);
            }
        }

        // Buffer fully consumed/discarded: live Ds are exactly those in worlds.
        assert_eq!(
            d_live(),
            d_count(&direct) + d_count(&buffered),
            "D live-count wrong after buffer consumed (leak or double-drop)"
        );

        // Adopt entities spawned this round (identical handles in both worlds,
        // as verified by compare) as targets for later rounds.
        for eref in direct.iter() {
            let e = eref.entity();
            if known.insert(e) {
                pool.push(e);
            }
        }
    }
}

#[cfg(not(miri))]
#[hegel::test(test_cases = 500)]
fn command_buffer_matches_direct(tc: hegel::TestCase) {
    drive(&tc, 4, 12);
}

#[cfg(miri)]
#[hegel::test(test_cases = 8)]
fn command_buffer_matches_direct(tc: hegel::TestCase) {
    drive(&tc, 2, 6);
}
