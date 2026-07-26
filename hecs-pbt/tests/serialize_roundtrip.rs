//! Serialize -> deserialize round-trip property tests for `hecs`.
//!
//! A hegel-generated arbitrary `World` (built over a small fixed component universe,
//! including a zero-sized marker component) is serialized and then deserialized into a
//! fresh `World` in both supported formats -- `hecs::serialize::row` and
//! `hecs::serialize::column` -- and the round-tripped world is asserted to be
//! observably equivalent to the original.
//!
//! Equivalence invariant: two worlds are equivalent iff they map exactly the same set of
//! `Entity` handles (id *and* generation) to the same observable component tuple. We assert
//! this via a normalized `HashMap<Entity, Obs>` rather than relying on iteration order,
//! which hecs does not guarantee to preserve. We verify (not merely assume) that both
//! formats preserve the full `Entity` handle: both deserializers reinstate entities with
//! `spawn_at`, so the exact id+generation survives the round-trip, and the `HashMap` keys
//! being `Entity` makes that part of the asserted invariant. The sorted key sets are also
//! compared separately so an id/generation regression fails with a distinct message.

use std::any::TypeId;
use std::collections::HashMap;

use hecs::serialize::{column, row};
use hecs::{Archetype, ColumnBatchBuilder, ColumnBatchType, Entity, EntityBuilder, EntityRef, World};
use hegel::generators as gs;
use serde::{Deserialize, Serialize};

// ---- fixed component universe ----
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct A(i32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct B(i32);
/// Zero-sized marker component.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct C;

/// Stable identifiers for the components we serialize.
#[derive(Clone, Copy, Serialize, Deserialize)]
enum ComponentId {
    A,
    B,
    C,
}

/// Normalized observable form of a single entity's components.
type Obs = (Option<i32>, Option<i32>, bool);

/// Collect a world into its normalized, order-independent observable representation.
fn observe(world: &World) -> HashMap<Entity, Obs> {
    let mut out = HashMap::new();
    for e in world.iter() {
        let obs: Obs = (
            e.get::<&A>().map(|r| r.0),
            e.get::<&B>().map(|r| r.0),
            e.has::<C>(),
        );
        let prev = out.insert(e.entity(), obs);
        assert!(prev.is_none(), "world yielded a duplicate entity handle {:?}", e.entity());
    }
    out
}

/// Assert two worlds are observably equivalent, including entity-id/generation preservation.
fn assert_equivalent(original: &World, restored: &World, format: &str) {
    let before = observe(original);
    let after = observe(restored);

    // Entity ids + generations must be preserved exactly (both formats use `spawn_at`).
    let mut ids_before: Vec<Entity> = before.keys().copied().collect();
    let mut ids_after: Vec<Entity> = after.keys().copied().collect();
    ids_before.sort();
    ids_after.sort();
    assert_eq!(
        ids_before, ids_after,
        "{format} round-trip did not preserve entity ids/generations"
    );

    // ... and every entity must carry the same components.
    assert_eq!(
        before, after,
        "{format} round-trip changed observable component data"
    );
}

// ---- row format context ----

struct RowContext;

impl row::SerializeContext for RowContext {
    fn serialize_entity<S>(&mut self, entity: EntityRef<'_>, mut map: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::SerializeMap,
    {
        row::try_serialize::<A, _, _>(&entity, &ComponentId::A, &mut map)?;
        row::try_serialize::<B, _, _>(&entity, &ComponentId::B, &mut map)?;
        row::try_serialize::<C, _, _>(&entity, &ComponentId::C, &mut map)?;
        map.end()
    }

    // Required so length-prefixed formats such as bincode work.
    fn component_count(&self, entity: EntityRef<'_>) -> Option<usize> {
        Some(entity.len())
    }
}

impl row::DeserializeContext for RowContext {
    fn deserialize_entity<'de, M>(
        &mut self,
        mut map: M,
        entity: &mut EntityBuilder,
    ) -> Result<(), M::Error>
    where
        M: serde::de::MapAccess<'de>,
    {
        while let Some(key) = map.next_key()? {
            match key {
                ComponentId::A => {
                    entity.add::<A>(map.next_value()?);
                }
                ComponentId::B => {
                    entity.add::<B>(map.next_value()?);
                }
                ComponentId::C => {
                    entity.add::<C>(map.next_value()?);
                }
            }
        }
        Ok(())
    }
}

fn row_roundtrip(world: &World) -> World {
    let opts = bincode::options();
    let mut buf = Vec::new();
    {
        let mut ser = bincode::Serializer::new(&mut buf, opts);
        row::serialize(world, &mut RowContext, &mut ser).expect("row serialize");
    }
    let mut de = bincode::Deserializer::with_reader(&buf[..], opts);
    row::deserialize(&mut RowContext, &mut de).expect("row deserialize")
}

// ---- column format context ----

struct ColumnSerContext;

impl column::SerializeContext for ColumnSerContext {
    fn component_count(&self, archetype: &Archetype) -> usize {
        archetype
            .component_types()
            .filter(|&t| {
                t == TypeId::of::<A>() || t == TypeId::of::<B>() || t == TypeId::of::<C>()
            })
            .count()
    }

    fn serialize_component_ids<S: serde::ser::SerializeTuple>(
        &mut self,
        archetype: &Archetype,
        mut out: S,
    ) -> Result<S::Ok, S::Error> {
        column::try_serialize_id::<A, _, _>(archetype, &ComponentId::A, &mut out)?;
        column::try_serialize_id::<B, _, _>(archetype, &ComponentId::B, &mut out)?;
        column::try_serialize_id::<C, _, _>(archetype, &ComponentId::C, &mut out)?;
        out.end()
    }

    fn serialize_components<S: serde::ser::SerializeTuple>(
        &mut self,
        archetype: &Archetype,
        mut out: S,
    ) -> Result<S::Ok, S::Error> {
        column::try_serialize::<A, _>(archetype, &mut out)?;
        column::try_serialize::<B, _>(archetype, &mut out)?;
        column::try_serialize::<C, _>(archetype, &mut out)?;
        out.end()
    }
}

#[derive(Default)]
struct ColumnDeContext {
    components: Vec<ComponentId>,
}

impl column::DeserializeContext for ColumnDeContext {
    fn deserialize_component_ids<'de, D>(&mut self, mut seq: D) -> Result<ColumnBatchType, D::Error>
    where
        D: serde::de::SeqAccess<'de>,
    {
        self.components.clear();
        let mut batch = ColumnBatchType::new();
        while let Some(id) = seq.next_element()? {
            match id {
                ComponentId::A => {
                    batch.add::<A>();
                }
                ComponentId::B => {
                    batch.add::<B>();
                }
                ComponentId::C => {
                    batch.add::<C>();
                }
            }
            self.components.push(id);
        }
        Ok(batch)
    }

    fn deserialize_components<'de, D>(
        &mut self,
        entity_count: u32,
        mut seq: D,
        batch: &mut ColumnBatchBuilder,
    ) -> Result<(), D::Error>
    where
        D: serde::de::SeqAccess<'de>,
    {
        for component in &self.components {
            match *component {
                ComponentId::A => {
                    column::deserialize_column::<A, _>(entity_count, &mut seq, batch)?;
                }
                ComponentId::B => {
                    column::deserialize_column::<B, _>(entity_count, &mut seq, batch)?;
                }
                ComponentId::C => {
                    column::deserialize_column::<C, _>(entity_count, &mut seq, batch)?;
                }
            }
        }
        Ok(())
    }
}

fn column_roundtrip(world: &World) -> World {
    let opts = bincode::options();
    let mut buf = Vec::new();
    {
        let mut ser = bincode::Serializer::new(&mut buf, opts);
        column::serialize(world, &mut ColumnSerContext, &mut ser).expect("column serialize");
    }
    let mut de = bincode::Deserializer::with_reader(&buf[..], opts);
    column::deserialize(&mut ColumnDeContext::default(), &mut de).expect("column deserialize")
}

// ---- generator: an arbitrary World over {A, B, C} ----

fn val() -> impl gs::Generator<i32> {
    gs::integers::<i32>().min_value(-3).max_value(3)
}

/// Build an arbitrary `World`: a sequence of spawns (each with an arbitrary component
/// subset) interleaved with despawns, so entity ids get recycled and generations advance --
/// exercising handle/generation preservation across the round-trip.
fn arbitrary_world(tc: &hegel::TestCase) -> World {
    let mut world = World::new();
    let mut live: Vec<Entity> = Vec::new();

    let steps = tc.draw(gs::integers::<u32>().min_value(0).max_value(40));
    for _ in 0..steps {
        // 0..=2 => spawn, 3 => despawn a live entity (if any).
        let op = tc.draw(gs::integers::<u8>().min_value(0).max_value(3));
        if op == 3 && !live.is_empty() {
            let i = tc.draw(gs::integers::<usize>().min_value(0).max_value(live.len() - 1));
            let e = live.swap_remove(i);
            world.despawn(e).expect("despawn of live entity");
        } else {
            let a = tc.draw(gs::optional(val()));
            let b = tc.draw(gs::optional(val()));
            let c = tc.draw(gs::booleans());
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
            let e = world.spawn(builder.build());
            live.push(e);
        }
    }
    world
}

#[hegel::test(test_cases = 200)]
fn row_format_roundtrips(tc: hegel::TestCase) {
    let world = arbitrary_world(&tc);
    let restored = row_roundtrip(&world);
    assert_equivalent(&world, &restored, "row");
}

#[hegel::test(test_cases = 200)]
fn column_format_roundtrips(tc: hegel::TestCase) {
    let world = arbitrary_world(&tc);
    let restored = column_roundtrip(&world);
    assert_equivalent(&world, &restored, "column");
}
