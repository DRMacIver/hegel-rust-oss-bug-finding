//! Property tests for the builder/bundle/ref API surface of hecs 0.11.0 —
//! coverage-guided: these target the regions the rest of the suite provably
//! never reached (bundle.rs `*_satisfies_query` + tuple `put_with_clone`,
//! EntityBuilder(Clone) `get_mut`/`clear`/`add_bundle`/`Clone`/`From`,
//! `Ref`/`RefMut` `map`/`clone`/formatting, and whole-column `Archetype::get`).
//!
//! The oracles stay property-shaped:
//!   * builder introspection and bundle query-satisfaction must agree with the
//!     drawn spec (ground truth is the spec, as in the differential suite);
//!   * `add_bundle` of a tuple ≡ individual `add`s (differential, observed
//!     through spawned-entity fingerprints);
//!   * a cloned builder ≡ its original; a rebuilt (`From<BuiltEntityClone>`)
//!     builder retains exactly the original components;
//!   * whole-column archetype access must present exactly the same values as
//!     per-entity access (fingerprint), and writes through a unique column
//!     must be observable per entity;
//!   * the Drop oracle runs throughout: builders and built-but-unspawned
//!     bundles must drop their components exactly once.

mod common;

use common::*;
use hecs::{
    bundle_satisfies_query, dynamic_bundle_satisfies_query, DynamicBundle, EntityBuilderClone,
    MissingComponent, Ref, RefMut, World,
};
use std::fmt;

impl fmt::Display for A {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "A={}", self.0)
    }
}

// ---- bundle query-satisfaction agrees with the drawn spec ----

fn drive_bundle_satisfaction(tc: &hegel::TestCase) {
    assert_d_balanced_at_start();
    let s = draw_bundle(tc);
    {
        let mut builder = make_builder(s);
        let built = builder.build();
        // BuiltEntity's DynamicBundle::has
        assert_eq!(built.has::<A>(), s.a.is_some(), "built.has::<A>");
        assert_eq!(built.has::<B>(), s.b.is_some(), "built.has::<B>");
        assert_eq!(built.has::<C>(), s.c, "built.has::<C>");
        assert_eq!(built.has::<D>(), s.d.is_some(), "built.has::<D>");
        // dynamic_bundle_satisfies_query: presence-queries mirror the spec
        assert_eq!(dynamic_bundle_satisfies_query::<_, &A>(&built), s.a.is_some());
        assert_eq!(dynamic_bundle_satisfies_query::<_, &B>(&built), s.b.is_some());
        assert_eq!(dynamic_bundle_satisfies_query::<_, &C>(&built), s.c);
        assert_eq!(dynamic_bundle_satisfies_query::<_, &D>(&built), s.d.is_some());
        assert_eq!(
            dynamic_bundle_satisfies_query::<_, (&A, &B)>(&built),
            s.a.is_some() && s.b.is_some(),
            "compound query satisfaction"
        );
        // built but never spawned: dropping builder+built must drop the
        // components exactly once (checked below via the D counter)
    }
    assert_eq!(d_live(), 0, "unspawned BuiltEntity leaked or double-dropped components");

    // Static-bundle satisfaction over fixed shapes.
    assert!(bundle_satisfies_query::<(A, B), &A>());
    assert!(bundle_satisfies_query::<(A, B), (&A, &B)>());
    assert!(!bundle_satisfies_query::<(A,), &B>());
    assert!(!bundle_satisfies_query::<(), &A>());
    // Tuple DynamicBundle::has
    let tup = (A(1), B(2));
    assert!(tup.has::<A>() && tup.has::<B>() && !tup.has::<C>());

    // MissingComponent's Display names the missing type.
    let msg = format!("{}", MissingComponent::new::<A>());
    assert!(msg.contains("missing") && msg.contains("A"), "MissingComponent display: {msg}");
}

// ---- EntityBuilderClone algebra ----

/// Add the {A,B,C} subset of `s` to a clonable builder one component at a time.
fn add_individually(b: &mut EntityBuilderClone, s: Bundle4) {
    if let Some(v) = s.a {
        b.add(A(v));
    }
    if let Some(v) = s.b {
        b.add(B(v));
    }
    if s.c {
        b.add(C);
    }
}

/// Add the {A,B,C} subset of `s` via `add_bundle` of one concrete tuple
/// (exercising the tuple `DynamicBundleClone::put_with_clone` impls).
fn add_as_tuple(b: &mut EntityBuilderClone, s: Bundle4) {
    match (s.a, s.b, s.c) {
        (None, None, false) => {
            b.add_bundle(());
        }
        (Some(a), None, false) => {
            b.add_bundle((A(a),));
        }
        (None, Some(v), false) => {
            b.add_bundle((B(v),));
        }
        (None, None, true) => {
            b.add_bundle((C,));
        }
        (Some(a), Some(v), false) => {
            b.add_bundle((A(a), B(v)));
        }
        (Some(a), None, true) => {
            b.add_bundle((A(a), C));
        }
        (None, Some(v), true) => {
            b.add_bundle((B(v), C));
        }
        (Some(a), Some(v), true) => {
            b.add_bundle((A(a), B(v), C));
        }
    }
}

fn obs_of(world: &World, e: hecs::Entity) -> Obs {
    fingerprint(world)
        .get(&e)
        .copied()
        .unwrap_or_else(|| panic!("{e:?} missing from world"))
}

fn drive_clone_builder(tc: &hegel::TestCase) {
    assert_d_balanced_at_start();
    let s = Bundle4 { d: None, ..draw_bundle(tc) }; // D is not Clone
    let expected = Obs { d: None, ..s };
    let n = s.a.is_some() as usize + s.b.is_some() as usize + s.c as usize;
    let mut world = World::new();

    // Introspection agrees with the spec; get_mut can rewrite a component.
    let mut b1 = EntityBuilderClone::new();
    add_individually(&mut b1, s);
    assert_eq!(b1.get::<&A>().map(|r| r.0), s.a, "clone-builder get::<&A>");
    assert_eq!(b1.component_types().count(), n, "clone-builder component_types");
    let v2 = tc.draw(val());
    let mut expected1 = expected;
    if let Some(a) = b1.get_mut::<&mut A>() {
        a.0 = v2;
        expected1.a = Some(v2);
    }

    // Clone before building: original and clone must spawn identical entities.
    // hecs 0.11.0 BUG (unfiled; see
    // draft-reports/hecs-entitybuilderclone-clone-zero-alloc.md and NOTES.md):
    // cloning a builder whose storage layout is zero-sized (empty or ZST-only)
    // calls alloc() with a zero-size layout — UB, caught by Miri. Until that
    // is fixed upstream, the clone-equivalence property can only be exercised
    // when a sized component is present; the UB repro lives in the draft
    // report, not in this suite.
    let has_sized_component = expected1.a.is_some() || expected1.b.is_some();
    let b2 = has_sized_component.then(|| b1.clone());
    let e1 = world.spawn(&b1.build());
    assert_eq!(obs_of(&world, e1), expected1, "spawn of mutated clone-builder");
    if let Some(b2) = b2 {
        let e2 = world.spawn(&b2.build());
        assert_eq!(obs_of(&world, e2), expected1, "clone of builder spawned differently");
    }

    // add_bundle(tuple) ≡ individual adds.
    let mut b3 = EntityBuilderClone::new();
    add_as_tuple(&mut b3, s);
    let e3 = world.spawn(&b3.build());
    assert_eq!(obs_of(&world, e3), expected, "add_bundle(tuple) != individual adds");

    // A BuiltEntityClone can itself be add_bundle'd into another builder.
    // (Re-importing one via `From<BuiltEntityClone> for EntityBuilderClone` and
    // then using get/get_mut/add is BROKEN in hecs 0.11.0 — see the
    // `from_built_entity_clone_stale_indices_observation` test below.)
    let mut b4 = EntityBuilderClone::new();
    add_individually(&mut b4, s);
    let built4 = b4.build();
    let mut b5 = EntityBuilderClone::new();
    b5.add_bundle(&built4);
    let e5 = world.spawn(&b5.build());
    assert_eq!(obs_of(&world, e5), expected, "add_bundle(&built) lost components");

    // clear() empties the builder.
    let mut b7 = EntityBuilderClone::new();
    add_individually(&mut b7, s);
    b7.clear();
    assert_eq!(b7.component_types().count(), 0, "clear() left component types");
    let e7 = world.spawn(&b7.build());
    assert_eq!(
        obs_of(&world, e7),
        Obs { a: None, b: None, c: false, d: None },
        "cleared builder spawned components"
    );
}

/// OBSERVATION of an UNFILED hecs 0.11.0 bug (found by this suite; see
/// `draft-reports/hecs-builtentityclone-stale-indices.md` and NOTES.md).
///
/// `EntityBuilderClone::build()` sorts the internal `info` vec by descending
/// alignment without rebuilding `indices` (the TypeId -> info-index map).
/// Spawning the `BuiltEntityClone` is fine (it iterates `info`), but after the
/// documented round-trip `From<BuiltEntityClone> for EntityBuilderClone`,
/// `get`/`get_mut`/`add` resolve through the stale `indices` and hit the wrong
/// slot. With Small(align 1) added before Big(align 8) the sort is guaranteed
/// to permute, so the wrong-slot read is deterministic: `get::<&Small>()`
/// returns the first byte of Big. (`add` on such a builder is worse — an
/// out-of-bounds read, confirmed by Miri — and is therefore not exercised
/// here.)
///
/// This test asserts the CURRENT (buggy) behaviour so the divergence stays
/// visible without breaking the suite. When upstream fixes it, this assert
/// fails: flip it to `Some(7)` and retire the observation comments.
#[test]
fn from_built_entity_clone_stale_indices_observation() {
    #[derive(Clone)]
    struct Small(u8);
    #[derive(Clone)]
    struct Big(u64);

    let mut b = EntityBuilderClone::new();
    b.add(Small(7));
    b.add(Big(0x4242_4242_4242_4242));
    let built = b.build();

    // The built bundle itself spawns correctly (iterates `info` directly).
    let mut world = World::new();
    let e = world.spawn(&built);
    assert_eq!(world.get::<&Small>(e).unwrap().0, 7, "spawned Small");
    assert_eq!(world.get::<&Big>(e).unwrap().0, 0x4242_4242_4242_4242, "spawned Big");

    let rebuilt: EntityBuilderClone = built.into();
    let got = rebuilt.get::<&Small>().map(|s| s.0);
    assert_eq!(
        got,
        Some(0x42),
        "hecs 0.11.0 returns Big's first byte through the stale index; if this \
         is now Some(7) the upstream bug was fixed — update this observation"
    );
}

// ---- plain EntityBuilder: get_mut, clear, and unspawned-drop balance ----

fn drive_plain_builder(tc: &hegel::TestCase) {
    assert_d_balanced_at_start();
    let s = draw_bundle(tc);
    let mut world = World::new();

    let mut b = make_builder(s);
    let v2 = tc.draw(val());
    let mut expected = s;
    if let Some(a) = b.get_mut::<&mut A>() {
        a.0 = v2;
        expected.a = Some(v2);
    }
    if let Some(d) = b.get_mut::<&mut D>() {
        d.0 = v2;
        expected.d = Some(v2);
    }
    let e = world.spawn(b.build());
    assert_eq!(obs_of(&world, e), expected, "builder get_mut edits not observed");

    // clear() drops the contents exactly once, without spawning.
    let mut b2 = make_builder(s);
    b2.clear();
    assert_eq!(b2.component_types().count(), 0, "clear() left component types");
    assert!(!b2.has::<A>() && !b2.has::<D>(), "clear() left components");
    drop(b2);
    let world_d = fingerprint_d_count(&fingerprint(&world));
    assert_eq!(d_live(), world_d, "EntityBuilder::clear leaked or double-dropped D");
}

// ---- Ref / RefMut wrappers ----

fn drive_ref_wrappers(tc: &hegel::TestCase) {
    assert_d_balanced_at_start();
    let v = tc.draw(val());
    let mut world = World::new();
    let e = world.spawn((A(v), D::new(v)));

    {
        let r: Ref<'_, A> = world.get::<&A>(e).expect("A just spawned");
        assert_eq!(format!("{r:?}"), format!("{:?}", A(v)), "Ref Debug");
        assert_eq!(format!("{r}"), format!("A={v}"), "Ref Display");
        let r2 = r.clone();
        assert_eq!(r2.0, v, "cloned Ref value");
        let inner: Ref<'_, i32> = Ref::map(r, |a| &a.0);
        assert_eq!(*inner, v, "Ref::map projection");
        assert_eq!(r2.0, v, "original clone unaffected by map");
    }
    {
        let m: RefMut<'_, A> = world.get::<&mut A>(e).expect("A just spawned");
        assert_eq!(format!("{m:?}"), format!("{:?}", A(v)), "RefMut Debug");
        assert_eq!(format!("{m}"), format!("A={v}"), "RefMut Display");
        let mut inner: RefMut<'_, i32> = RefMut::map(m, |a| &mut a.0);
        *inner = v + 1;
    }
    assert_eq!(world.get::<&A>(e).unwrap().0, v + 1, "write through RefMut::map lost");
    world.despawn(e).unwrap();
    assert_eq!(d_live(), 0, "D drop imbalance in ref-wrapper test");
}

// ---- whole-column archetype access agrees with per-entity access ----

fn drive_archetype_columns(tc: &hegel::TestCase, max_entities: u32) {
    assert_d_balanced_at_start();
    let (mut worlds, _pool) = build_twins(tc, 1, max_entities);
    let world = &mut worlds[0];
    let fp = fingerprint(world);

    // Shared columns: concatenating (ids ⋈ column) over all archetypes must
    // reproduce exactly the A-values the fingerprint sees, keyed by id.
    let mut got: std::collections::BTreeMap<u32, i32> = std::collections::BTreeMap::new();
    for arch in world.archetypes() {
        if let Some(col) = arch.get::<&A>() {
            assert_eq!(col.len(), arch.len() as usize, "column length != archetype length");
            let col2 = col.clone(); // second shared borrow of the same column
            assert_eq!(format!("{col:?}"), format!("{:?}", &col2[..]), "column Debug");
            for (&id, a) in arch.ids().iter().zip(col.iter()) {
                assert!(got.insert(id, a.0).is_none(), "entity id {id} in two A-columns");
            }
        }
    }
    let want: std::collections::BTreeMap<u32, i32> =
        fp.iter().filter_map(|(e, o)| o.a.map(|v| (e.id(), v))).collect();
    assert_eq!(got, want, "archetype A-columns disagree with per-entity reads");

    // Unique columns: a write through ArchetypeColumnMut must be observable
    // through ordinary reads afterwards.
    let v = tc.draw(val());
    for arch in world.archetypes() {
        if let Some(mut col) = arch.get::<&mut B>() {
            for b in col.iter_mut() {
                b.0 = v;
            }
            assert_eq!(format!("{col:?}"), format!("{:?}", &col[..]), "mut column Debug");
        }
    }
    for (e, o) in fingerprint(world) {
        assert_eq!(
            o.b,
            fp[&e].b.map(|_| v),
            "column-mut sweep of B not observed per entity ({e:?})"
        );
        // untouched components unchanged
        assert_eq!(o.a, fp[&e].a, "column-mut sweep of B disturbed A on {e:?}");
        assert_eq!(o.d, fp[&e].d, "column-mut sweep of B disturbed D on {e:?}");
    }
}

// ---- entry points ----

#[cfg(not(miri))]
mod normal {
    use super::*;

    #[hegel::test(test_cases = 300)]
    fn bundle_satisfaction_matches_spec(tc: hegel::TestCase) {
        drive_bundle_satisfaction(&tc);
    }

    #[hegel::test(test_cases = 300)]
    fn clone_builder_algebra(tc: hegel::TestCase) {
        drive_clone_builder(&tc);
    }

    #[hegel::test(test_cases = 300)]
    fn plain_builder_get_mut_and_clear(tc: hegel::TestCase) {
        drive_plain_builder(&tc);
    }

    #[hegel::test(test_cases = 300)]
    fn ref_wrappers_project_and_format(tc: hegel::TestCase) {
        drive_ref_wrappers(&tc);
    }

    #[hegel::test(test_cases = 300)]
    fn archetype_columns_agree_with_entity_reads(tc: hegel::TestCase) {
        drive_archetype_columns(&tc, 8);
    }
}

// All of these move component data through raw pointers (Common::clone,
// put_with_clone, get_base, builder storage) — worth a UB pass for each.
#[cfg(miri)]
mod miri {
    use super::*;

    #[hegel::test(test_cases = 8, suppress_health_check = [hegel::HealthCheck::TooSlow])]
    fn bundle_satisfaction_matches_spec(tc: hegel::TestCase) {
        drive_bundle_satisfaction(&tc);
    }

    #[hegel::test(test_cases = 8, suppress_health_check = [hegel::HealthCheck::TooSlow])]
    fn clone_builder_algebra(tc: hegel::TestCase) {
        drive_clone_builder(&tc);
    }

    #[hegel::test(test_cases = 8, suppress_health_check = [hegel::HealthCheck::TooSlow])]
    fn plain_builder_get_mut_and_clear(tc: hegel::TestCase) {
        drive_plain_builder(&tc);
    }

    #[hegel::test(test_cases = 8, suppress_health_check = [hegel::HealthCheck::TooSlow])]
    fn ref_wrappers_project_and_format(tc: hegel::TestCase) {
        drive_ref_wrappers(&tc);
    }

    #[hegel::test(test_cases = 8, suppress_health_check = [hegel::HealthCheck::TooSlow])]
    fn archetype_columns_agree_with_entity_reads(tc: hegel::TestCase) {
        drive_archetype_columns(&tc, 4);
    }
}
