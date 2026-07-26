//! Property-based-testing testbed for hecs — stateful model-based testing.
//!
//! The World's observable contents are modelled by `HashMap<Entity, M>` over a fixed
//! component universe {A, B, C(ZST), D(Drop-tracked)}. We draw a sequence of operations,
//! apply each to both the real World and the model, and after every step assert:
//!   * full bidirectional entity/component equivalence + `len`,
//!   * per-op result (ok/err) agrees with the model,
//!   * a Drop leak/double-drop oracle: live `D` instances == modelled `D` count,
//!   * archetype structural invariants (partition of entities; Σ len == world.len),
//!   * query-correctness for several query shapes.

#[cfg(test)]
mod model {
    use hecs::{Entity, EntityBuilder, With, Without, World};
    use hegel::generators as gs;
    use std::cell::Cell;
    use std::collections::{HashMap, HashSet};

    // ---- fixed component universe ----
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct A(i32);
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct B(i32);
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct C; // zero-sized marker

    // Drop-tracked component: NOT Copy; bumps a per-thread live-count on new/drop.
    // Correct hecs behaviour keeps `D_LIVE == (# modelled entities with a D)` at all times;
    // a missed drop (leak) or an extra drop (double-free) on archetype migration/despawn shows up.
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

    // ---- reference model of one entity's components ----
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    struct M {
        a: Option<i32>,
        b: Option<i32>,
        c: bool,
        d: Option<i32>,
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

    fn check_query_a(world: &World, model: &HashMap<Entity, M>) {
        let mut got: HashMap<Entity, i32> = HashMap::new();
        for (e, a) in world.query::<(Entity, &A)>().iter() {
            assert!(got.insert(e, a.0).is_none(), "query::<&A> yielded {:?} twice", e);
        }
        let want: HashMap<Entity, i32> =
            model.iter().filter_map(|(&e, m)| m.a.map(|v| (e, v))).collect();
        assert_eq!(got, want, "query::<&A> set/values");
    }

    fn check_query_ab(world: &World, model: &HashMap<Entity, M>) {
        let mut got: HashMap<Entity, (i32, i32)> = HashMap::new();
        for (e, a, b) in world.query::<(Entity, &A, &B)>().iter() {
            assert!(got.insert(e, (a.0, b.0)).is_none(), "query::<(&A,&B)> dup {:?}", e);
        }
        let want: HashMap<Entity, (i32, i32)> = model
            .iter()
            .filter_map(|(&e, m)| match (m.a, m.b) {
                (Some(a), Some(b)) => Some((e, (a, b))),
                _ => None,
            })
            .collect();
        assert_eq!(got, want, "query::<(&A,&B)> set/values");
    }

    fn check_query_with_without(world: &World, model: &HashMap<Entity, M>) {
        // With<&A, &B>: A-values of entities that also have B
        let mut with: HashMap<Entity, i32> = HashMap::new();
        for (e, a) in world.query::<With<(Entity, &A), &B>>().iter() {
            with.insert(e, a.0);
        }
        let with_want: HashMap<Entity, i32> = model
            .iter()
            .filter_map(|(&e, m)| match (m.a, m.b) {
                (Some(a), Some(_)) => Some((e, a)),
                _ => None,
            })
            .collect();
        assert_eq!(with, with_want, "query::<With<&A,&B>>");

        // Without<&A, &B>: A-values of entities that do NOT have B
        let mut without: HashMap<Entity, i32> = HashMap::new();
        for (e, a) in world.query::<Without<(Entity, &A), &B>>().iter() {
            without.insert(e, a.0);
        }
        let without_want: HashMap<Entity, i32> = model
            .iter()
            .filter_map(|(&e, m)| match (m.a, m.b) {
                (Some(a), None) => Some((e, a)),
                _ => None,
            })
            .collect();
        assert_eq!(without, without_want, "query::<Without<&A,&B>>");
    }

    /// Full oracle: World and model describe the same entities/components, drops balance,
    /// archetypes partition the entities, and queries return exactly the right sets.
    /// Materialize reserved entities into empty real entities (mirrors hecs's implicit flush).
    fn flush_model(model: &mut HashMap<Entity, M>, reserved: &mut Vec<Entity>) {
        for e in reserved.drain(..) {
            model.insert(e, M::default());
        }
    }

    fn check(world: &World, model: &HashMap<Entity, M>, reserved: &[Entity]) {
        assert_eq!(world.len() as usize, model.len(), "len mismatch");

        // model -> world (presence + exact component values)
        for (&e, m) in model {
            assert!(world.contains(e), "world missing modelled entity {:?}", e);
            assert_eq!(world.get::<&A>(e).ok().map(|r| r.0), m.a, "A for {:?}", e);
            assert_eq!(world.get::<&B>(e).ok().map(|r| r.0), m.b, "B for {:?}", e);
            assert_eq!(world.get::<&C>(e).is_ok(), m.c, "C for {:?}", e);
            assert_eq!(world.get::<&D>(e).ok().map(|r| r.0), m.d, "D for {:?}", e);
        }
        // world -> model (nothing extra)
        for eref in world.iter() {
            let e = eref.entity();
            assert!(model.contains_key(&e), "world has un-modelled entity {:?}", e);
        }

        // Drop leak/double-drop oracle
        let model_d = model.values().filter(|m| m.d.is_some()).count() as i64;
        assert_eq!(d_live(), model_d, "live-D count != modelled D count (leak/double-drop)");

        // archetype structural invariants: entities partition across archetypes exactly once
        let mut arch_total = 0u32;
        let mut ids: HashSet<u32> = HashSet::new();
        for arch in world.archetypes() {
            arch_total += arch.len();
            for &id in arch.ids() {
                assert!(ids.insert(id), "entity id {} appears in >1 archetype", id);
            }
        }
        assert_eq!(arch_total, world.len(), "Σ archetype.len() != world.len()");
        assert_eq!(ids.len() as u32, world.len(), "archetype id count != world.len()");

        // query correctness
        check_query_a(world, model);
        check_query_ab(world, model);
        check_query_with_without(world, model);

        // reserved-but-unflushed entities: contained, but excluded from len/iter/queries/model
        for &r in reserved {
            assert!(world.contains(r), "reserved handle {:?} should be contained", r);
            assert!(!model.contains_key(&r), "reserved {:?} leaked into model", r);
        }
    }

    /// Core harness: apply `max_steps` drawn operations to both World and model, checking after each.
    fn drive(tc: &hegel::TestCase, max_steps: u32) {
        assert_eq!(d_live(), 0, "D leaked across test cases (before start)");
        let mut world = World::new();
        let mut model: HashMap<Entity, M> = HashMap::new();
        let mut known: Vec<Entity> = Vec::new(); // all spawned handles; kept post-despawn for stale-handle coverage
        let mut reserved: Vec<Entity> = Vec::new(); // reserve_entity handles not yet flushed

        let steps = tc.draw(gs::integers::<u32>().min_value(0).max_value(max_steps));
        for _ in 0..steps {
            match tc.draw(gs::integers::<u8>().min_value(0).max_value(10)) {
                // reserve an entity id concurrently (not visible until flush)
                8 => {
                    let e = world.reserve_entity();
                    reserved.push(e);
                    known.push(e);
                }
                // explicit flush: reserved -> empty real entities
                9 => {
                    world.flush();
                    flush_model(&mut model, &mut reserved);
                }
                // take: despawn via TakenEntity (distinct unsafe removal path); drops its components
                10 => {
                    flush_model(&mut model, &mut reserved);
                    if let Some(e) = pick(tc, &known) {
                        let ok = world.take(e).is_ok();
                        let existed = model.remove(&e).is_some();
                        assert_eq!(ok, existed, "take ok-ness disagrees for {:?}", e);
                    }
                }
                // spawn an arbitrary subset of {A, B, C, D}
                0 | 1 => {
                    flush_model(&mut model, &mut reserved);
                    let a = tc.draw(gs::optional(val()));
                    let b = tc.draw(gs::optional(val()));
                    let c = tc.draw(gs::booleans());
                    let d = tc.draw(gs::optional(val()));
                    let mut builder = EntityBuilder::new();
                    if let Some(v) = a {
                        builder.add(A(v));
                    }
                    if let Some(v) = b {
                        builder.add(B(v));
                    }
                    if c {
                        builder.add(C);
                    }
                    if let Some(v) = d {
                        builder.add(D::new(v));
                    }
                    let e = world.spawn(builder.build());
                    model.insert(e, M { a, b, c, d });
                    known.push(e);
                }
                // despawn a (possibly stale) handle
                2 => {
                    flush_model(&mut model, &mut reserved);
                    if let Some(e) = pick(tc, &known) {
                        let ok = world.despawn(e).is_ok();
                        let existed = model.remove(&e).is_some();
                        assert_eq!(ok, existed, "despawn ok-ness disagrees for {:?}", e);
                    }
                }
                // insert one component
                3 | 4 => {
                    flush_model(&mut model, &mut reserved);
                    if let Some(e) = pick(tc, &known) {
                        let live = model.contains_key(&e);
                        match tc.draw(gs::integers::<u8>().min_value(0).max_value(3)) {
                            0 => {
                                let v = tc.draw(val());
                                let ok = world.insert_one(e, A(v)).is_ok();
                                assert_eq!(ok, live, "insert A ok-ness for {:?}", e);
                                if let Some(m) = model.get_mut(&e) {
                                    m.a = Some(v);
                                }
                            }
                            1 => {
                                let v = tc.draw(val());
                                let ok = world.insert_one(e, B(v)).is_ok();
                                assert_eq!(ok, live, "insert B ok-ness for {:?}", e);
                                if let Some(m) = model.get_mut(&e) {
                                    m.b = Some(v);
                                }
                            }
                            2 => {
                                let ok = world.insert_one(e, C).is_ok();
                                assert_eq!(ok, live, "insert C ok-ness for {:?}", e);
                                if let Some(m) = model.get_mut(&e) {
                                    m.c = true;
                                }
                            }
                            _ => {
                                let v = tc.draw(val());
                                // D::new increments; on a dead target insert_one drops it again (net 0)
                                let ok = world.insert_one(e, D::new(v)).is_ok();
                                assert_eq!(ok, live, "insert D ok-ness for {:?}", e);
                                if let Some(m) = model.get_mut(&e) {
                                    m.d = Some(v);
                                }
                            }
                        }
                    }
                }
                // remove one component
                _ => {
                    flush_model(&mut model, &mut reserved);
                    if let Some(e) = pick(tc, &known) {
                        match tc.draw(gs::integers::<u8>().min_value(0).max_value(3)) {
                            0 => {
                                let removed = world.remove_one::<A>(e).is_ok();
                                let had = model.get(&e).map(|m| m.a.is_some()).unwrap_or(false);
                                assert_eq!(removed, had, "remove A for {:?}", e);
                                if let Some(m) = model.get_mut(&e) {
                                    m.a = None;
                                }
                            }
                            1 => {
                                let removed = world.remove_one::<B>(e).is_ok();
                                let had = model.get(&e).map(|m| m.b.is_some()).unwrap_or(false);
                                assert_eq!(removed, had, "remove B for {:?}", e);
                                if let Some(m) = model.get_mut(&e) {
                                    m.b = None;
                                }
                            }
                            2 => {
                                let removed = world.remove_one::<C>(e).is_ok();
                                let had = model.get(&e).map(|m| m.c).unwrap_or(false);
                                assert_eq!(removed, had, "remove C for {:?}", e);
                                if let Some(m) = model.get_mut(&e) {
                                    m.c = false;
                                }
                            }
                            _ => {
                                // the returned D drops here -> decrement, matching model d = None
                                let removed = world.remove_one::<D>(e).is_ok();
                                let had = model.get(&e).map(|m| m.d.is_some()).unwrap_or(false);
                                assert_eq!(removed, had, "remove D for {:?}", e);
                                if let Some(m) = model.get_mut(&e) {
                                    m.d = None;
                                }
                            }
                        }
                    }
                }
            }
            check(&world, &model, &reserved);
        }
    }

    #[cfg(not(miri))]
    #[hegel::test(test_cases = 1000)]
    fn world_matches_model(tc: hegel::TestCase) {
        drive(&tc, 300);
    }

    #[cfg(miri)]
    #[hegel::test(test_cases = 12)]
    fn world_matches_model(tc: hegel::TestCase) {
        drive(&tc, 25);
    }
}
