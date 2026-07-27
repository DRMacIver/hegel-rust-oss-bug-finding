//! Query-surface properties for hecs 0.11.0 — every public way to *read* the
//! world must agree with the observational fingerprint (ground truth taken
//! once via plain per-entity reads).
//!
//! Coverage-guided: targets the query.rs machinery the rest of the suite
//! never touched — `Satisfies<Q>`, `&mut` fetches through the dynamically
//! checked `query()` path, `Or` accessors (`split`/`left`/`right`/`as_mut`/
//! `cloned`), `QueryBorrow`/`QueryMut` combinators (`with`/`without`/`view`/
//! `into_iter_batched`), iterator `len`/`size_hint`, `ViewIter` (`iter_mut` /
//! `IntoIterator for &mut View/PreparedView/ViewBorrow`), `ViewBorrow`
//! random-access (`get_mut`/`get_disjoint_mut`/`get_unchecked`), the full
//! `PreparedView` surface, and `PreparedQuery::default`.
//!
//! Oracle: each read shape is an independent derived view of the same state;
//! all must project the fingerprint exactly (right entity set, right values,
//! no duplicates).

mod common;

use common::*;
use hecs::{Entity, Or, PreparedQuery, Satisfies};
use hegel::generators as gs;
use std::collections::BTreeMap;

/// The A-values the fingerprint predicts, keyed by entity.
fn want_a(fp: &Fingerprint) -> BTreeMap<Entity, i32> {
    fp.iter().filter_map(|(&e, o)| o.a.map(|v| (e, v))).collect()
}

/// Collect an iterator of (Entity, &A)-shaped items into a map, asserting no duplicates.
fn collect_a(it: impl Iterator<Item = (Entity, i32)>, label: &str) -> BTreeMap<Entity, i32> {
    let mut got = BTreeMap::new();
    for (e, v) in it {
        assert!(got.insert(e, v).is_none(), "{label}: duplicate {e:?}");
    }
    got
}

fn drive(tc: &hegel::TestCase, max_entities: u32) {
    assert_d_balanced_at_start();
    let (mut worlds, pool) = build_twins(tc, 1, max_entities);
    let world = &mut worlds[0];
    let mut fp = fingerprint(world);

    // ---- Satisfies<Q>: yields a bool for EVERY entity, even componentless ----
    {
        let mut got: BTreeMap<Entity, (bool, bool)> = BTreeMap::new();
        for (e, sa, sab) in world
            .query::<(Entity, Satisfies<&A>, Satisfies<(&A, &B)>)>()
            .iter()
        {
            assert!(got.insert(e, (sa, sab)).is_none(), "Satisfies dup {e:?}");
        }
        let want: BTreeMap<Entity, (bool, bool)> = fp
            .iter()
            .map(|(&e, o)| (e, (o.a.is_some(), o.a.is_some() && o.b.is_some())))
            .collect();
        assert_eq!(got, want, "query::<Satisfies> disagrees with fingerprint");
    }

    // ---- &mut fetch through the dynamically-checked query() path, plus
    //      iterator len / size_hint ----
    let v = tc.draw(val());
    {
        let mut q = world.query::<(Entity, &mut A)>();
        let mut it = q.iter();
        let n_a = want_a(&fp).len();
        assert_eq!(it.len(), n_a, "QueryIter::len");
        assert_eq!(it.size_hint(), (n_a, Some(n_a)), "QueryIter::size_hint");
        for (e, a) in &mut it {
            assert_eq!(Some(a.0), fp[&e].a, "query::<&mut A> initial value {e:?}");
            a.0 = v; // write through the unique fetch
        }
    }
    for o in fp.values_mut() {
        if o.a.is_some() {
            o.a = Some(v);
        }
    }
    assert_eq!(fingerprint(world), fp, "writes through query::<&mut A> not observed");

    // ---- Or accessors: split/left/right/as_mut/cloned agree with the variant ----
    for (e, or) in world.query::<(Entity, Or<&A, &B>)>().iter() {
        let o = fp[&e];
        let (l, r) = or.split();
        assert_eq!(l.map(|a| a.0), o.a, "Or::split left {e:?}");
        assert_eq!(r.map(|b| b.0), o.b, "Or::split right {e:?}");
        assert_eq!(or.left().map(|a| a.0), o.a, "Or::left {e:?}");
        assert_eq!(or.right().map(|b| b.0), o.b, "Or::right {e:?}");
        let mut owned: Or<A, B> = or.cloned();
        let (lm, rm) = owned.as_mut().split();
        assert_eq!(lm.map(|a| a.0), o.a, "Or::as_mut left {e:?}");
        assert_eq!(rm.map(|b| b.0), o.b, "Or::as_mut right {e:?}");
    }

    // ---- QueryBorrow combinators: with / without / view ----
    {
        let got = collect_a(
            world
                .query::<(Entity, &A)>()
                .with::<&B>()
                .iter()
                .map(|(e, a)| (e, a.0)),
            "QueryBorrow::with",
        );
        let want: BTreeMap<Entity, i32> = fp
            .iter()
            .filter_map(|(&e, o)| o.b.and(o.a).map(|av| (e, av)))
            .collect();
        assert_eq!(got, want, "QueryBorrow::with::<&B>");
    }
    {
        let got = collect_a(
            world
                .query::<(Entity, &A)>()
                .without::<&B>()
                .iter()
                .map(|(e, a)| (e, a.0)),
            "QueryBorrow::without",
        );
        let want: BTreeMap<Entity, i32> = fp
            .iter()
            .filter_map(|(&e, o)| match (o.a, o.b) {
                (Some(av), None) => Some((e, av)),
                _ => None,
            })
            .collect();
        assert_eq!(got, want, "QueryBorrow::without::<&B>");
    }
    {
        let mut q = world.query::<&A>();
        let view = q.view();
        for (&e, o) in &fp {
            assert_eq!(view.get(e).map(|a| a.0), o.a, "QueryBorrow::view get {e:?}");
        }
    }

    // ---- QueryMut combinators: with / without / view / into_iter_batched ----
    {
        let got = collect_a(
            world
                .query_mut::<(Entity, &A)>()
                .with::<&B>()
                .into_iter()
                .map(|(e, a)| (e, a.0)),
            "QueryMut::with",
        );
        let want: BTreeMap<Entity, i32> = fp
            .iter()
            .filter_map(|(&e, o)| o.b.and(o.a).map(|av| (e, av)))
            .collect();
        assert_eq!(got, want, "QueryMut::with::<&B>");
    }
    {
        let got = collect_a(
            world
                .query_mut::<(Entity, &A)>()
                .without::<&B>()
                .into_iter()
                .map(|(e, a)| (e, a.0)),
            "QueryMut::without",
        );
        let want: BTreeMap<Entity, i32> = fp
            .iter()
            .filter_map(|(&e, o)| match (o.a, o.b) {
                (Some(av), None) => Some((e, av)),
                _ => None,
            })
            .collect();
        assert_eq!(got, want, "QueryMut::without::<&B>");
    }
    {
        let mut qm = world.query_mut::<&A>();
        let view = qm.view();
        for (&e, o) in &fp {
            assert_eq!(view.get(e).map(|a| a.0), o.a, "QueryMut::view get {e:?}");
        }
    }
    {
        let batch_size = tc.draw(gs::integers::<u32>().min_value(1).max_value(4));
        let mut got = BTreeMap::new();
        for batch in world
            .query_mut::<(Entity, &A)>()
            .into_iter_batched(batch_size)
        {
            for (e, a) in batch {
                assert!(got.insert(e, a.0).is_none(), "into_iter_batched dup {e:?}");
            }
        }
        assert_eq!(got, want_a(&fp), "QueryMut::into_iter_batched");
    }

    // ---- View iteration (ViewIter via iter_mut and IntoIterator) ----
    {
        let mut view = world.view_mut::<(Entity, &mut A)>();
        let got = collect_a((&mut view).into_iter().map(|(e, a)| (e, a.0)), "View iter");
        assert_eq!(got, want_a(&fp), "&mut View IntoIterator");
        let got2 = collect_a(view.iter_mut().map(|(e, a)| (e, a.0)), "View iter_mut");
        assert_eq!(got2, want_a(&fp), "View::iter_mut");
    }

    // ---- ViewBorrow random access: get_mut / get_disjoint_mut / get_unchecked ----
    {
        let mut vb = world.view::<(Entity, &A)>();
        let got = collect_a((&mut vb).into_iter().map(|(e, a)| (e, a.0)), "ViewBorrow iter");
        assert_eq!(got, want_a(&fp), "&mut ViewBorrow IntoIterator");
        for (&e, o) in &fp {
            assert_eq!(
                vb.get_mut(e).map(|(ge, a)| (ge, a.0)),
                o.a.map(|av| (e, av)),
                "ViewBorrow::get_mut {e:?}"
            );
            // SAFETY: Q = (Entity, &A) yields only shared references and no
            // unique borrow of A is alive here.
            let unchecked = unsafe { vb.get_unchecked(e) };
            assert_eq!(
                unchecked.map(|(ge, a)| (ge, a.0)),
                o.a.map(|av| (e, av)),
                "ViewBorrow::get_unchecked {e:?}"
            );
        }
        if pool.len() >= 2 {
            let e1 = pick(tc, &pool).unwrap();
            let e2 = pick(tc, &pool).unwrap();
            if e1 != e2 {
                let [r1, r2] = vb.get_disjoint_mut([e1, e2]);
                assert_eq!(
                    r1.map(|(_, a)| a.0),
                    fp.get(&e1).and_then(|o| o.a),
                    "ViewBorrow::get_disjoint_mut e1 {e1:?}"
                );
                assert_eq!(
                    r2.map(|(_, a)| a.0),
                    fp.get(&e2).and_then(|o| o.a),
                    "ViewBorrow::get_disjoint_mut e2 {e2:?}"
                );
            }
        }
    }

    // ---- PreparedView full surface (+ PreparedQuery::default) ----
    {
        let mut pq = PreparedQuery::<(Entity, &A)>::default();
        let mut pv = pq.view_mut(world);
        let got = collect_a((&mut pv).into_iter().map(|(e, a)| (e, a.0)), "PreparedView iter");
        assert_eq!(got, want_a(&fp), "&mut PreparedView IntoIterator");
        let got2 = collect_a(pv.iter_mut().map(|(e, a)| (e, a.0)), "PreparedView iter_mut");
        assert_eq!(got2, want_a(&fp), "PreparedView::iter_mut");
        for (&e, o) in &fp {
            assert_eq!(
                pv.get_mut(e).map(|(ge, a)| (ge, a.0)),
                o.a.map(|av| (e, av)),
                "PreparedView::get_mut {e:?}"
            );
            // SAFETY: Q yields only shared references; no unique borrow alive.
            let unchecked = unsafe { pv.get_unchecked(e) };
            assert_eq!(
                unchecked.map(|(ge, a)| (ge, a.0)),
                o.a.map(|av| (e, av)),
                "PreparedView::get_unchecked {e:?}"
            );
        }
        if pool.len() >= 2 {
            let e1 = pick(tc, &pool).unwrap();
            let e2 = pick(tc, &pool).unwrap();
            if e1 != e2 {
                let [r1, r2] = pv.get_disjoint_mut([e1, e2]);
                assert_eq!(
                    r1.map(|(_, a)| a.0),
                    fp.get(&e1).and_then(|o| o.a),
                    "PreparedView::get_disjoint_mut e1 {e1:?}"
                );
                assert_eq!(
                    r2.map(|(_, a)| a.0),
                    fp.get(&e2).and_then(|o| o.a),
                    "PreparedView::get_disjoint_mut e2 {e2:?}"
                );
            }
        }
    }

    // Reads must not have changed anything.
    assert_eq!(fingerprint(world), fp, "a read-only shape mutated the world");
    check_archetypes(world, "query-shapes world");
}

#[cfg(not(miri))]
#[hegel::test(test_cases = 400)]
fn query_shapes_agree_with_fingerprint(tc: hegel::TestCase) {
    drive(&tc, 8);
}

#[cfg(miri)]
#[hegel::test(test_cases = 8, suppress_health_check = [hegel::HealthCheck::TooSlow])]
fn query_shapes_agree_with_fingerprint(tc: hegel::TestCase) {
    drive(&tc, 4);
}
