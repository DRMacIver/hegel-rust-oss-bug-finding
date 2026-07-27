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
    use hecs::{
        Entity, EntityBuilder, EntityBuilderClone, Or, PreparedQuery, QueryOneError, With, Without,
        World,
    };
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

    /// `query::<Or<&A,&B>>`: exactly the entities with A or B, each variant reflecting which it has.
    fn check_query_or(world: &World, model: &HashMap<Entity, M>) {
        let mut got: HashMap<Entity, (Option<i32>, Option<i32>)> = HashMap::new();
        for (e, ab) in world.query::<(Entity, Or<&A, &B>)>().iter() {
            let pair = match ab {
                Or::Left(a) => (Some(a.0), None),
                Or::Right(b) => (None, Some(b.0)),
                Or::Both(a, b) => (Some(a.0), Some(b.0)),
            };
            assert!(got.insert(e, pair).is_none(), "query::<Or<&A,&B>> dup {:?}", e);
        }
        let want: HashMap<Entity, (Option<i32>, Option<i32>)> = model
            .iter()
            .filter(|(_, m)| m.a.is_some() || m.b.is_some())
            .map(|(&e, m)| (e, (m.a, m.b)))
            .collect();
        assert_eq!(got, want, "query::<Or<&A,&B>> set/values");
    }

    /// `query::<(&A, Option<&B>)>`: exactly the entities with A, each carrying its optional B.
    fn check_query_option(world: &World, model: &HashMap<Entity, M>) {
        let mut got: HashMap<Entity, Option<i32>> = HashMap::new();
        for (e, _a, ob) in world.query::<(Entity, &A, Option<&B>)>().iter() {
            assert!(got.insert(e, ob.map(|b| b.0)).is_none(), "Option query dup {:?}", e);
        }
        let want: HashMap<Entity, Option<i32>> =
            model.iter().filter_map(|(&e, m)| m.a.map(|_| (e, m.b))).collect();
        assert_eq!(got, want, "query::<(&A, Option<&B>)> set/values");
    }

    /// `query_one` (shared borrow) agrees with the model for every live entity, incl. Unsatisfied.
    fn check_query_one(world: &World, model: &HashMap<Entity, M>) {
        for (&e, m) in model {
            match world.query_one::<&A>(e).get() {
                Ok(a) => assert_eq!(Some(a.0), m.a, "query_one &A value {:?}", e),
                Err(QueryOneError::Unsatisfied) => {
                    assert!(m.a.is_none(), "query_one &A Unsatisfied but modelled present {:?}", e)
                }
                Err(QueryOneError::NoSuchEntity) => panic!("query_one &A NoSuchEntity for live {:?}", e),
            }
            match world.query_one::<(&A, &B)>(e).get() {
                Ok((a, b)) => {
                    assert_eq!(Some(a.0), m.a, "query_one (A,B).A {:?}", e);
                    assert_eq!(Some(b.0), m.b, "query_one (A,B).B {:?}", e);
                }
                Err(QueryOneError::Unsatisfied) => assert!(
                    m.a.is_none() || m.b.is_none(),
                    "query_one (A,B) Unsatisfied but both modelled {:?}",
                    e
                ),
                Err(QueryOneError::NoSuchEntity) => panic!("query_one (A,B) NoSuchEntity for live {:?}", e),
            }
            // QueryOne::with / without combinators: A-value filtered by B-presence
            match world.query_one::<&A>(e).with::<&B>().get() {
                Ok(a) => {
                    assert_eq!(Some(a.0), m.a, "query_one::<&A>.with::<&B> value {:?}", e);
                    assert!(m.b.is_some(), "with::<&B> Ok but no B modelled {:?}", e);
                }
                Err(QueryOneError::Unsatisfied) => assert!(
                    m.a.is_none() || m.b.is_none(),
                    "with::<&B> Unsatisfied but A&B modelled {:?}",
                    e
                ),
                Err(QueryOneError::NoSuchEntity) => panic!("with NoSuchEntity for live {:?}", e),
            }
            match world.query_one::<&A>(e).without::<&B>().get() {
                Ok(a) => {
                    assert_eq!(Some(a.0), m.a, "query_one::<&A>.without::<&B> value {:?}", e);
                    assert!(m.b.is_none(), "without::<&B> Ok but B modelled {:?}", e);
                }
                Err(QueryOneError::Unsatisfied) => assert!(
                    m.a.is_none() || m.b.is_some(),
                    "without::<&B> Unsatisfied but A-only modelled {:?}",
                    e
                ),
                Err(QueryOneError::NoSuchEntity) => panic!("without NoSuchEntity for live {:?}", e),
            }
        }
    }

    /// `query_one_mut` (unique borrow — the fast path) agrees with the model for every live entity.
    fn check_query_one_mut(world: &mut World, model: &HashMap<Entity, M>) {
        for (&e, m) in model {
            match world.query_one_mut::<&A>(e) {
                Ok(a) => assert_eq!(Some(a.0), m.a, "query_one_mut &A value {:?}", e),
                Err(QueryOneError::Unsatisfied) => {
                    assert!(m.a.is_none(), "query_one_mut &A Unsatisfied but modelled present {:?}", e)
                }
                Err(QueryOneError::NoSuchEntity) => panic!("query_one_mut &A NoSuchEntity for live {:?}", e),
            }
        }
    }

    /// `satisfies` agrees with the model for single and compound queries.
    fn check_satisfies(world: &World, model: &HashMap<Entity, M>) {
        for (&e, m) in model {
            assert_eq!(world.satisfies::<&A>(e), m.a.is_some(), "satisfies &A {:?}", e);
            assert_eq!(world.satisfies::<&B>(e), m.b.is_some(), "satisfies &B {:?}", e);
            assert_eq!(world.satisfies::<&C>(e), m.c, "satisfies &C {:?}", e);
            assert_eq!(world.satisfies::<&D>(e), m.d.is_some(), "satisfies &D {:?}", e);
            assert_eq!(
                world.satisfies::<(&A, &B)>(e),
                m.a.is_some() && m.b.is_some(),
                "satisfies (A,B) {:?}",
                e
            );
        }
    }

    /// The `EntityRef` view of each live entity matches the model (has/get/len/component_types/query).
    fn check_entity_ref(world: &World, model: &HashMap<Entity, M>) {
        for (&e, m) in model {
            let er = world.entity(e).expect("live modelled entity must have an EntityRef");
            assert_eq!(er.entity(), e, "EntityRef.entity");
            assert_eq!(er.has::<A>(), m.a.is_some(), "EntityRef.has::<A> {:?}", e);
            assert_eq!(er.has::<B>(), m.b.is_some(), "EntityRef.has::<B> {:?}", e);
            assert_eq!(er.has::<C>(), m.c, "EntityRef.has::<C> {:?}", e);
            assert_eq!(er.has::<D>(), m.d.is_some(), "EntityRef.has::<D> {:?}", e);
            assert_eq!(er.get::<&A>().map(|r| r.0), m.a, "EntityRef.get::<&A> {:?}", e);
            assert_eq!(er.get::<&B>().map(|r| r.0), m.b, "EntityRef.get::<&B> {:?}", e);
            assert_eq!(er.get::<&D>().map(|r| r.0), m.d, "EntityRef.get::<&D> {:?}", e);
            assert_eq!(er.satisfies::<&A>(), m.a.is_some(), "EntityRef.satisfies::<&A> {:?}", e);
            let expect_len = m.a.is_some() as usize
                + m.b.is_some() as usize
                + m.c as usize
                + m.d.is_some() as usize;
            assert_eq!(er.len(), expect_len, "EntityRef.len {:?}", e);
            assert_eq!(er.is_empty(), expect_len == 0, "EntityRef.is_empty {:?}", e);
            assert_eq!(
                er.component_types().count(),
                expect_len,
                "EntityRef.component_types count {:?}",
                e
            );
            match er.query::<&A>().get() {
                Ok(a) => assert_eq!(Some(a.0), m.a, "EntityRef.query::<&A> {:?}", e),
                Err(_) => assert!(m.a.is_none(), "EntityRef.query::<&A> err but modelled {:?}", e),
            }
        }
    }

    /// Random-access `View` (via `world.view`): `get`/`contains` agree with the model per entity.
    fn check_view(world: &World, model: &HashMap<Entity, M>) {
        let view = world.view::<&A>();
        for (&e, m) in model {
            assert_eq!(view.get(e).map(|a| a.0), m.a, "view.get::<&A> {:?}", e);
            assert_eq!(view.contains(e), m.a.is_some(), "view.contains::<&A> {:?}", e);
        }
        // an entity with no A must be absent from the view even though it exists
        for (&e, m) in model {
            if m.a.is_none() {
                assert!(view.get(e).is_none(), "view.get returned Some for A-less {:?}", e);
            }
        }
    }

    /// Batched iteration must visit exactly the same set/values as flat iteration.
    fn check_iter_batched(world: &World, model: &HashMap<Entity, M>) {
        let mut got: HashMap<Entity, i32> = HashMap::new();
        let mut q = world.query::<(Entity, &A)>();
        for batch in q.iter_batched(2) {
            for (e, a) in batch {
                assert!(got.insert(e, a.0).is_none(), "iter_batched dup {:?}", e);
            }
        }
        let want: HashMap<Entity, i32> =
            model.iter().filter_map(|(&e, m)| m.a.map(|v| (e, v))).collect();
        assert_eq!(got, want, "iter_batched::<&A> set/values");
    }

    /// A `PreparedQuery` (archetype-cached) must return exactly the same set/values as a fresh query,
    /// both by iteration and by `PreparedQueryBorrow::view` random access.
    fn check_prepared(world: &World, model: &HashMap<Entity, M>) {
        let mut pq = PreparedQuery::<(Entity, &A)>::new();
        let mut got: HashMap<Entity, i32> = HashMap::new();
        {
            let mut borrow = pq.query(world);
            for (e, a) in borrow.iter() {
                assert!(got.insert(e, a.0).is_none(), "prepared dup {:?}", e);
            }
        }
        let want: HashMap<Entity, i32> =
            model.iter().filter_map(|(&e, m)| m.a.map(|v| (e, v))).collect();
        assert_eq!(got, want, "PreparedQuery::<&A> set/values");

        // PreparedQueryBorrow::view — random access by handle over the cached query
        let mut pv = PreparedQuery::<&A>::new();
        let mut borrow = pv.query(world);
        let view = borrow.view();
        for (&e, m) in model {
            assert_eq!(view.get(e).map(|a| a.0), m.a, "prepared borrow view.get {:?}", e);
            assert_eq!(view.contains(e), m.a.is_some(), "prepared borrow view.contains {:?}", e);
        }
    }

    /// `PreparedView` on a uniquely-borrowed world: random access by handle agrees with the model.
    fn check_prepared_view(world: &mut World, model: &HashMap<Entity, M>) {
        let mut pq = PreparedQuery::<&A>::new();
        let view = pq.view_mut(world);
        for (&e, m) in model {
            assert_eq!(view.get(e).map(|a| a.0), m.a, "prepared_view.get {:?}", e);
            assert_eq!(view.contains(e), m.a.is_some(), "prepared_view.contains {:?}", e);
        }
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

        // query correctness (shared-borrow query shapes + per-entity views)
        check_query_a(world, model);
        check_query_ab(world, model);
        check_query_with_without(world, model);
        check_query_or(world, model);
        check_query_option(world, model);
        check_query_one(world, model);
        check_satisfies(world, model);
        check_entity_ref(world, model);
        check_view(world, model);
        check_iter_batched(world, model);
        check_prepared(world, model);

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
            match tc.draw(gs::integers::<u8>().min_value(0).max_value(23)) {
                // reserve entity id(s) concurrently (not visible until flush): one, or a bulk range
                8 => {
                    match tc.draw(gs::integers::<u8>().min_value(0).max_value(1)) {
                        0 => {
                            let e = world.reserve_entity();
                            reserved.push(e);
                            known.push(e);
                        }
                        _ => {
                            let n = tc.draw(gs::integers::<u32>().min_value(0).max_value(4));
                            let ents: Vec<Entity> = world.reserve_entities(n).collect();
                            for e in ents {
                                reserved.push(e);
                                known.push(e);
                            }
                        }
                    }
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
                // insert a multi-component bundle in one call (Bundle path, not insert_one)
                11 => {
                    flush_model(&mut model, &mut reserved);
                    if let Some(e) = pick(tc, &known) {
                        let live = model.contains_key(&e);
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
                        let ok = world.insert(e, builder.build()).is_ok();
                        assert_eq!(ok, live, "insert bundle ok-ness for {:?}", e);
                        if let Some(m) = model.get_mut(&e) {
                            if let Some(v) = a {
                                m.a = Some(v);
                            }
                            if let Some(v) = b {
                                m.b = Some(v);
                            }
                            if c {
                                m.c = true;
                            }
                            if let Some(v) = d {
                                m.d = Some(v);
                            }
                        }
                        // if the target was dead, the built D (if any) is dropped by insert -> net 0
                    }
                }
                // remove a whole bundle: all-or-nothing (Err if any member missing, removes nothing)
                12 => {
                    flush_model(&mut model, &mut reserved);
                    if let Some(e) = pick(tc, &known) {
                        match tc.draw(gs::integers::<u8>().min_value(0).max_value(1)) {
                            0 => {
                                let ok = world.remove::<(A, B)>(e).is_ok();
                                let had = model
                                    .get(&e)
                                    .map(|m| m.a.is_some() && m.b.is_some())
                                    .unwrap_or(false);
                                assert_eq!(ok, had, "remove (A,B) for {:?}", e);
                                if ok {
                                    let m = model.get_mut(&e).unwrap();
                                    m.a = None;
                                    m.b = None;
                                }
                            }
                            _ => {
                                // the removed D (on success) is returned and dropped here
                                let ok = world.remove::<(C, D)>(e).is_ok();
                                let had = model
                                    .get(&e)
                                    .map(|m| m.c && m.d.is_some())
                                    .unwrap_or(false);
                                assert_eq!(ok, had, "remove (C,D) for {:?}", e);
                                if ok {
                                    let m = model.get_mut(&e).unwrap();
                                    m.c = false;
                                    m.d = None;
                                }
                            }
                        }
                    }
                }
                // exchange one component for another (routes via an intermediate archetype)
                13 => {
                    flush_model(&mut model, &mut reserved);
                    if let Some(e) = pick(tc, &known) {
                        let v = tc.draw(val());
                        match tc.draw(gs::integers::<u8>().min_value(0).max_value(1)) {
                            0 => {
                                // remove A, add B: requires A present
                                let ok = world.exchange_one::<A, B>(e, B(v)).is_ok();
                                let had =
                                    model.get(&e).map(|m| m.a.is_some()).unwrap_or(false);
                                assert_eq!(ok, had, "exchange A->B for {:?}", e);
                                if ok {
                                    let m = model.get_mut(&e).unwrap();
                                    m.a = None;
                                    m.b = Some(v);
                                }
                            }
                            _ => {
                                // remove D, add A: requires D present; removed D is returned & dropped
                                let ok = world.exchange_one::<D, A>(e, A(v)).is_ok();
                                let had =
                                    model.get(&e).map(|m| m.d.is_some()).unwrap_or(false);
                                assert_eq!(ok, had, "exchange D->A for {:?}", e);
                                if ok {
                                    let m = model.get_mut(&e).unwrap();
                                    m.d = None;
                                    m.a = Some(v);
                                }
                            }
                        }
                    }
                }
                // mutate a single component in place via get::<&mut _> (a read path: does NOT flush)
                14 => {
                    if let Some(e) = pick(tc, &known) {
                        let v = tc.draw(val());
                        match tc.draw(gs::integers::<u8>().min_value(0).max_value(2)) {
                            0 => {
                                if let Ok(mut a) = world.get::<&mut A>(e) {
                                    a.0 = v;
                                }
                                if let Some(m) = model.get_mut(&e) {
                                    if m.a.is_some() {
                                        m.a = Some(v);
                                    }
                                }
                            }
                            1 => {
                                if let Ok(mut b) = world.get::<&mut B>(e) {
                                    b.0 = v;
                                }
                                if let Some(m) = model.get_mut(&e) {
                                    if m.b.is_some() {
                                        m.b = Some(v);
                                    }
                                }
                            }
                            _ => {
                                // mutate D's payload in place — no construct/drop, so D_LIVE is unchanged
                                if let Ok(mut d) = world.get::<&mut D>(e) {
                                    d.0 = v;
                                }
                                if let Some(m) = model.get_mut(&e) {
                                    if m.d.is_some() {
                                        m.d = Some(v);
                                    }
                                }
                            }
                        }
                    }
                }
                // sweep-mutate every A via query_mut, setting each to a drawn value
                15 => {
                    let v = tc.draw(val());
                    for (_e, a) in world.query_mut::<(Entity, &mut A)>() {
                        a.0 = v;
                    }
                    for m in model.values_mut() {
                        if m.a.is_some() {
                            m.a = Some(v);
                        }
                    }
                }
                // clear the whole world (Entity values then repeat — stale handles may alias)
                16 => {
                    world.clear();
                    model.clear();
                    reserved.clear();
                }
                // spawn a homogeneous batch of (A, B) entities in one call
                17 => {
                    flush_model(&mut model, &mut reserved);
                    let n = tc.draw(gs::integers::<u32>().min_value(0).max_value(5));
                    let v = tc.draw(val());
                    let ents: Vec<Entity> =
                        world.spawn_batch((0..n).map(|_| (A(v), B(v)))).collect();
                    for e in ents {
                        model.insert(
                            e,
                            M {
                                a: Some(v),
                                b: Some(v),
                                c: false,
                                d: None,
                            },
                        );
                        known.push(e);
                    }
                }
                // reserve component capacity (a pure hint: no observable change)
                18 => {
                    flush_model(&mut model, &mut reserved);
                    let n = tc.draw(gs::integers::<u32>().min_value(0).max_value(8));
                    world.reserve::<(A, B)>(n);
                }
                // mutate two DISTINCT entities' A at once via query_disjoint_mut (does NOT flush)
                19 => {
                    if known.len() >= 2 {
                        let i1 =
                            tc.draw(gs::integers::<usize>().min_value(0).max_value(known.len() - 1));
                        let i2 =
                            tc.draw(gs::integers::<usize>().min_value(0).max_value(known.len() - 1));
                        let (e1, e2) = (known[i1], known[i2]);
                        // assert_distinct panics on equal handles, so only proceed when distinct
                        if e1 != e2 {
                            let v1 = tc.draw(val());
                            let v2 = tc.draw(val());
                            let (e1_had, e2_had);
                            {
                                let [r1, r2] = world.query_disjoint_mut::<&mut A, 2>([e1, e2]);
                                e1_had = if let Ok(a) = r1 {
                                    a.0 = v1;
                                    true
                                } else {
                                    false
                                };
                                e2_had = if let Ok(a) = r2 {
                                    a.0 = v2;
                                    true
                                } else {
                                    false
                                };
                            }
                            let m1 = model.get(&e1).map(|m| m.a.is_some()).unwrap_or(false);
                            let m2 = model.get(&e2).map(|m| m.a.is_some()).unwrap_or(false);
                            assert_eq!(e1_had, m1, "query_disjoint_mut A-presence e1 {:?}", e1);
                            assert_eq!(e2_had, m2, "query_disjoint_mut A-presence e2 {:?}", e2);
                            if e1_had {
                                model.get_mut(&e1).unwrap().a = Some(v1);
                            }
                            if e2_had {
                                model.get_mut(&e2).unwrap().a = Some(v2);
                            }
                        }
                    }
                }
                // mutate B via a random-access View (view_mut -> get_mut / get_disjoint_mut; no flush)
                20 => {
                    if let Some(e1) = pick(tc, &known) {
                        match tc.draw(gs::integers::<u8>().min_value(0).max_value(1)) {
                            0 => {
                                // single-entity random access
                                let v = tc.draw(val());
                                let had = {
                                    let mut view = world.view_mut::<&mut B>();
                                    if let Some(b) = view.get_mut(e1) {
                                        b.0 = v;
                                        true
                                    } else {
                                        false
                                    }
                                };
                                let m1 = model.get(&e1).map(|m| m.b.is_some()).unwrap_or(false);
                                assert_eq!(had, m1, "view.get_mut B-presence {:?}", e1);
                                if had {
                                    model.get_mut(&e1).unwrap().b = Some(v);
                                }
                            }
                            _ => {
                                // disjoint two-entity random access
                                let i2 = tc.draw(
                                    gs::integers::<usize>().min_value(0).max_value(known.len() - 1),
                                );
                                let e2 = known[i2];
                                if e1 != e2 {
                                    let v1 = tc.draw(val());
                                    let v2 = tc.draw(val());
                                    let (e1_had, e2_had);
                                    {
                                        let mut view = world.view_mut::<&mut B>();
                                        let [r1, r2] = view.get_disjoint_mut([e1, e2]);
                                        e1_had = if let Some(b) = r1 {
                                            b.0 = v1;
                                            true
                                        } else {
                                            false
                                        };
                                        e2_had = if let Some(b) = r2 {
                                            b.0 = v2;
                                            true
                                        } else {
                                            false
                                        };
                                    }
                                    let m1 =
                                        model.get(&e1).map(|m| m.b.is_some()).unwrap_or(false);
                                    let m2 =
                                        model.get(&e2).map(|m| m.b.is_some()).unwrap_or(false);
                                    assert_eq!(e1_had, m1, "view.get_disjoint_mut B e1 {:?}", e1);
                                    assert_eq!(e2_had, m2, "view.get_disjoint_mut B e2 {:?}", e2);
                                    if e1_had {
                                        model.get_mut(&e1).unwrap().b = Some(v1);
                                    }
                                    if e2_had {
                                        model.get_mut(&e2).unwrap().b = Some(v2);
                                    }
                                }
                            }
                        }
                    }
                }
                // take an entity and MOVE it into a scratch world (exercises TakenEntity::put,
                // the unsafe component-move path — distinct from the drop-on-take path in arm 10)
                21 => {
                    flush_model(&mut model, &mut reserved);
                    if let Some(e) = pick(tc, &known) {
                        let expected = model.get(&e).copied();
                        match world.take(e) {
                            Ok(taken) => {
                                let m = expected.expect("take Ok implies entity was modelled");
                                let mut scratch = World::new();
                                let e2 = scratch.spawn(taken); // moves components via put()
                                assert_eq!(scratch.get::<&A>(e2).ok().map(|r| r.0), m.a, "migrated A");
                                assert_eq!(scratch.get::<&B>(e2).ok().map(|r| r.0), m.b, "migrated B");
                                assert_eq!(scratch.get::<&C>(e2).is_ok(), m.c, "migrated C");
                                assert_eq!(scratch.get::<&D>(e2).ok().map(|r| r.0), m.d, "migrated D");
                                assert_eq!(scratch.len(), 1, "scratch world holds exactly the moved entity");
                                model.remove(&e);
                                // scratch drops here: any moved D drops now, matching model.remove
                            }
                            Err(_) => assert!(expected.is_none(), "take failed but {:?} modelled", e),
                        }
                    }
                }
                // spawn via the clonable builder (EntityBuilderClone); D is not Clone, so {A,B,C} only
                22 => {
                    flush_model(&mut model, &mut reserved);
                    let a = tc.draw(gs::optional(val()));
                    let b = tc.draw(gs::optional(val()));
                    let c = tc.draw(gs::booleans());
                    let mut builder = EntityBuilderClone::new();
                    if let Some(v) = a {
                        builder.add(A(v));
                    }
                    if let Some(v) = b {
                        builder.add(B(v));
                    }
                    if c {
                        builder.add(C);
                    }
                    assert_eq!(builder.has::<A>(), a.is_some(), "clone-builder.has::<A>");
                    let built = builder.build();
                    // build() consumes into a reusable bundle; spawn twice to exercise the clone path
                    let e1 = world.spawn(&built);
                    let e2 = world.spawn(&built);
                    model.insert(e1, M { a, b, c, d: None });
                    model.insert(e2, M { a, b, c, d: None });
                    known.push(e1);
                    known.push(e2);
                }
                // spawn_at a chosen handle: forces that exact id+generation live, overwriting
                // (and dropping) whatever entity currently occupies that id
                23 => {
                    flush_model(&mut model, &mut reserved);
                    if let Some(handle) = pick(tc, &known) {
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
                        world.spawn_at(handle, builder.build());
                        // any live entity sharing this id is destroyed (its D, if any, dropped);
                        // the id now resolves to exactly `handle`
                        let vid = handle.id();
                        model.retain(|k, _| k.id() != vid);
                        model.insert(handle, M { a, b, c, d });
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
                    // EntityBuilder introspection agrees with what we added
                    assert_eq!(builder.has::<A>(), a.is_some(), "builder.has::<A>");
                    assert_eq!(builder.has::<B>(), b.is_some(), "builder.has::<B>");
                    assert_eq!(builder.has::<C>(), c, "builder.has::<C>");
                    assert_eq!(builder.has::<D>(), d.is_some(), "builder.has::<D>");
                    assert_eq!(builder.get::<&A>().map(|r| r.0), a, "builder.get::<&A>");
                    let ntypes = a.is_some() as usize
                        + b.is_some() as usize
                        + c as usize
                        + d.is_some() as usize;
                    assert_eq!(builder.component_types().count(), ntypes, "builder.component_types");
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
            // unique-borrow query paths (query_one_mut, PreparedView) checked separately
            check_query_one_mut(&mut world, &model);
            check_prepared_view(&mut world, &model);
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
