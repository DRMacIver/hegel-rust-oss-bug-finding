//! Serialize ERROR and EDGE path tests for `hecs::serialize::{row, column}`.
//!
//! Complements `tests/serialize_roundtrip.rs` (which covers the arbitrary-world happy
//! path) by closing coverage of:
//!
//! * edge round-trips: empty world, componentless entities, all-components entities,
//!   many entities across several archetypes (both formats);
//! * malformed input: every strict prefix of a valid byte stream must `Err` (both
//!   formats), bounded random corruption must never panic, and hand-crafted malformed
//!   streams must hit the specific error paths (zero entity bits, unknown component id,
//!   duplicate column id -> "extra component", truncated entity list);
//! * `serialize_satisfying` (both formats): exactly the `&A`-satisfying entities
//!   round-trip, with all of their components;
//! * a Drop-tracked leak hunt on the truncated-input deserialize paths.
//!
//! FINDINGS (documented here, NOT asserted as correct):
//!
//! 1. LEAK (corroborates the known `ColumnBatchBuilder` drop bug, see
//!    `../draft-reports/hecs-columnbatchbuilder-leak.md` and `tests/column_batch.rs`):
//!    when column-format `deserialize` fails partway through component data (e.g.
//!    truncated input), every component value already deserialized into the internal
//!    `ColumnBatchBuilder` is LEAKED -- its destructor never runs.
//!    `ColumnBatchBuilder::drop` steps a `*mut u8` by byte index and calls
//!    `drop_in_place::<u8>` (a no-op). Minimal trigger: serialize a world with ONE
//!    entity carrying one Drop component in column format, truncate the final byte,
//!    deserialize -> Err, and the one deserialized component is leaked (live-counter
//!    stays +1). `column_truncated_input_errors_and_leaks_observation` below measures
//!    the leak but only asserts the documented contract (`Err`, no double-drop).
//!    The row format is clean: `EntityBuilder` drops buffered components correctly
//!    (asserted in `row_truncated_input_errors_and_does_not_leak`).
//!
//! 2. PANIC on malformed input: a column stream declaring `entity_count == u32::MAX`
//!    makes `ColumnBatchType::into_batch` panic (`assert!(size < u32::MAX)` in
//!    src/batch.rs) instead of `deserialize` returning `Err`.
//!    `column_huge_entity_count_must_not_produce_a_world` asserts only the documented
//!    contract (malformed input never yields `Ok(World)`) and tolerates Err-or-panic
//!    without endorsing the panic.
//!
//! 4. PANIC / UB on malformed input with DUPLICATE entity ids (column format): a column
//!    archetype whose entity-id list repeats an id makes `World::spawn_column_batch_at`
//!    call `Entities::alloc_at` twice for that id; the second call takes the "id already
//!    in use" branch and returns the entity's EMPTY location (`{archetype: 0, index:
//!    u32::MAX}`), so hecs then calls `Archetype::remove(u32::MAX, true)` on archetype 0.
//!    In debug this panics (`self.len - 1` underflow, archetype.rs:321); in release the
//!    subtraction wraps and `remove` does an out-of-bounds `drop_in_place`/copy at index
//!    u32::MAX -- undefined behaviour. Deterministic trigger:
//!    `column_duplicate_entity_ids_must_not_corrupt` (two entities sharing bits `1<<32`,
//!    zero components). This is why `column_corrupted_input_does_not_panic` wraps
//!    `deserialize` in `catch_unwind`: bit-flips in the entity-id region routinely
//!    produce colliding ids and hit this path. Both tests assert only the documented
//!    contract (malformed input must never yield an incorrect `Ok(World)`); the panic is
//!    tolerated, never asserted correct.
//!
//! 5. UNBOUNDED ALLOCATION on malformed input (analysis, deliberately not exercised at
//!    scale): both deserializers trust attacker-controlled integers before validation.
//!    Row: `spawn_at` grows the entity metadata table to the deserialized entity id
//!    (`Entities::alloc_at` does `pending.extend(0..id)` + `meta.resize(id+1, ..)`,
//!    i.e. ~20 bytes of *touched* memory per id), so a single corrupted high byte of
//!    an entity id can commit tens of GB and invite the OOM killer. Column:
//!    `entity_count` drives `Vec::reserve` and `Archetype::reserve` before any data is
//!    validated. This is why the random-corruption properties below flip only the low
//!    two bits of drawn bytes: with the small worlds used here that provably bounds
//!    any count/id the parser can see (<= a few tens of millions), keeping the tests
//!    safe to run while still exercising framing shifts, invalid enum discriminants,
//!    invalid entity bits, and value corruption.

use std::any::TypeId;
use std::cell::Cell;
use std::collections::HashMap;
use std::panic::{catch_unwind, AssertUnwindSafe};

use bincode::Options;
use hecs::serialize::{column, row};
use hecs::{
    Archetype, ColumnBatchBuilder, ColumnBatchType, Entity, EntityBuilder, EntityRef, Query, World,
};
use hegel::generators as gs;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

// ---- fixed component universe (same as serialize_roundtrip.rs) ----

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct A(i32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct B(i32);
/// Zero-sized marker component.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct C;

/// Stable identifiers for the components we serialize. Discriminants (bincode varint
/// u32) are A=0, B=1, C=2; anything >= 3 is an unknown-variant error on deserialize.
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
        assert!(
            prev.is_none(),
            "world yielded a duplicate entity handle {:?}",
            e.entity()
        );
    }
    out
}

/// Assert two worlds are observably equivalent, including entity-id/generation preservation.
fn assert_equivalent(original: &World, restored: &World, format: &str) {
    let before = observe(original);
    let after = observe(restored);

    let mut ids_before: Vec<Entity> = before.keys().copied().collect();
    let mut ids_after: Vec<Entity> = after.keys().copied().collect();
    ids_before.sort();
    ids_after.sort();
    assert_eq!(
        ids_before, ids_after,
        "{format} round-trip did not preserve entity ids/generations"
    );
    assert_eq!(
        before, after,
        "{format} round-trip changed observable component data"
    );
}

// ---- row format context (as in serialize_roundtrip.rs) ----

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

// ---- column format contexts (as in serialize_roundtrip.rs) ----

struct ColumnSerContext;

impl column::SerializeContext for ColumnSerContext {
    fn component_count(&self, archetype: &Archetype) -> usize {
        archetype
            .component_types()
            .filter(|&t| t == TypeId::of::<A>() || t == TypeId::of::<B>() || t == TypeId::of::<C>())
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

// ---- byte-level helpers ----

fn row_to_bytes(world: &World) -> Vec<u8> {
    let mut buf = Vec::new();
    let mut ser = bincode::Serializer::new(&mut buf, bincode::options());
    row::serialize(world, &mut RowContext, &mut ser).expect("row serialize");
    buf
}

fn row_from_bytes(bytes: &[u8]) -> Result<World, bincode::Error> {
    let mut de = bincode::Deserializer::with_reader(bytes, bincode::options());
    row::deserialize(&mut RowContext, &mut de)
}

fn row_satisfying_to_bytes<Q: Query>(world: &World) -> Vec<u8> {
    let mut buf = Vec::new();
    let mut ser = bincode::Serializer::new(&mut buf, bincode::options());
    row::serialize_satisfying::<Q, _, _>(world, &mut RowContext, &mut ser)
        .expect("row serialize_satisfying");
    buf
}

fn column_to_bytes(world: &World) -> Vec<u8> {
    let mut buf = Vec::new();
    let mut ser = bincode::Serializer::new(&mut buf, bincode::options());
    column::serialize(world, &mut ColumnSerContext, &mut ser).expect("column serialize");
    buf
}

fn column_from_bytes(bytes: &[u8]) -> Result<World, bincode::Error> {
    let mut de = bincode::Deserializer::with_reader(bytes, bincode::options());
    column::deserialize(&mut ColumnDeContext::default(), &mut de)
}

fn column_satisfying_to_bytes<Q: Query>(world: &World) -> Vec<u8> {
    let mut buf = Vec::new();
    let mut ser = bincode::Serializer::new(&mut buf, bincode::options());
    column::serialize_satisfying::<Q, _, _>(world, &mut ColumnSerContext, &mut ser)
        .expect("column serialize_satisfying");
    buf
}

/// bincode-encode a primitive with the same options used everywhere else, for
/// hand-crafting malformed streams.
fn enc<T: Serialize>(value: &T) -> Vec<u8> {
    bincode::options().serialize(value).expect("encode")
}

// ---- generators ----

fn val() -> impl gs::Generator<i32> {
    gs::integers::<i32>().min_value(-3).max_value(3)
}

/// Arbitrary world over {A, B, C}: spawns with arbitrary component subsets interleaved
/// with despawns, so ids recycle and generations advance (as in serialize_roundtrip.rs).
/// `max_steps` bounds world size (corruption tests use small worlds to bound the
/// allocations reachable through corrupted ids/counts; see file comment, finding 5).
fn arbitrary_world(tc: &hegel::TestCase, max_steps: u32) -> World {
    let mut world = World::new();
    let mut live: Vec<Entity> = Vec::new();

    let steps = tc.draw(gs::integers::<u32>().min_value(0).max_value(max_steps));
    for _ in 0..steps {
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

/// A world with entities spread over every subset-of-{A,B,C} archetype (up to 8
/// archetypes, up to 40 entities).
fn multi_archetype_world(tc: &hegel::TestCase) -> World {
    let mut world = World::new();
    for subset in 0u8..8 {
        let n = tc.draw(gs::integers::<u32>().min_value(0).max_value(5));
        for _ in 0..n {
            let mut builder = EntityBuilder::new();
            if subset & 1 != 0 {
                builder.add(A(tc.draw(val())));
            }
            if subset & 2 != 0 {
                builder.add(B(tc.draw(val())));
            }
            if subset & 4 != 0 {
                builder.add(C);
            }
            world.spawn(builder.build());
        }
    }
    world
}

fn assert_roundtrips_both_formats(world: &World) {
    let row_restored = row_from_bytes(&row_to_bytes(world)).expect("row deserialize");
    assert_equivalent(world, &row_restored, "row");
    let col_restored = column_from_bytes(&column_to_bytes(world)).expect("column deserialize");
    assert_equivalent(world, &col_restored, "column");
}

// =====================================================================
// (a) EDGE ROUND-TRIPS
// =====================================================================

#[test]
fn empty_world_roundtrips_both_formats() {
    let world = World::new();
    let restored_row = row_from_bytes(&row_to_bytes(&world)).expect("row deserialize of empty");
    assert_eq!(restored_row.len(), 0, "row round-trip of empty world not empty");
    assert_equivalent(&world, &restored_row, "row");

    let restored_col =
        column_from_bytes(&column_to_bytes(&world)).expect("column deserialize of empty");
    assert_eq!(restored_col.len(), 0, "column round-trip of empty world not empty");
    assert_equivalent(&world, &restored_col, "column");
}

#[test]
fn componentless_entities_roundtrip_both_formats() {
    // Entities with NO components live in the unit archetype; the column format must
    // still serialize them (entity ids only, zero component columns).
    let mut world = World::new();
    for _ in 0..5 {
        world.spawn(());
    }
    // Advance a generation so handle preservation is non-trivial.
    let doomed = world.spawn(());
    world.despawn(doomed).expect("despawn");
    world.spawn(());

    assert_roundtrips_both_formats(&world);
}

#[test]
fn every_entity_all_components_roundtrips_both_formats() {
    let mut world = World::new();
    let mut spawned = Vec::new();
    for i in 0..6 {
        spawned.push(world.spawn((A(i), B(-i), C)));
    }
    // Recycle two ids so generations > 1 appear.
    world.despawn(spawned[1]).expect("despawn");
    world.despawn(spawned[4]).expect("despawn");
    world.spawn((A(100), B(-100), C));
    world.spawn((A(101), B(-101), C));

    assert_roundtrips_both_formats(&world);
}

#[hegel::test(test_cases = 300)]
fn multi_archetype_worlds_roundtrip_both_formats(tc: hegel::TestCase) {
    let world = multi_archetype_world(&tc);
    assert_roundtrips_both_formats(&world);
}

// =====================================================================
// (b) MALFORMED / TRUNCATED INPUT
// =====================================================================

// Every strict prefix of a valid stream must fail: bincode parsing is deterministic,
// so a prefix parse follows the full parse byte-for-byte until it runs out of input.

#[hegel::test(test_cases = 300)]
fn row_truncated_input_always_errors(tc: hegel::TestCase) {
    let world = arbitrary_world(&tc, 40);
    let bytes = row_to_bytes(&world);
    for cut in 0..bytes.len() {
        let result = row_from_bytes(&bytes[..cut]);
        assert!(
            result.is_err(),
            "row deserialize of strict prefix {}/{} unexpectedly succeeded",
            cut,
            bytes.len()
        );
    }
    // The untruncated stream must still parse, and to the same world.
    let restored = row_from_bytes(&bytes).expect("row deserialize of full stream");
    assert_equivalent(&world, &restored, "row");
}

#[hegel::test(test_cases = 300)]
fn column_truncated_input_always_errors(tc: hegel::TestCase) {
    let world = arbitrary_world(&tc, 40);
    let bytes = column_to_bytes(&world);
    for cut in 0..bytes.len() {
        let result = column_from_bytes(&bytes[..cut]);
        assert!(
            result.is_err(),
            "column deserialize of strict prefix {}/{} unexpectedly succeeded",
            cut,
            bytes.len()
        );
    }
    let restored = column_from_bytes(&bytes).expect("column deserialize of full stream");
    assert_equivalent(&world, &restored, "column");
}

// Random corruption: the IDEAL contract is "deserialize returns Err or a usable Ok,
// never a panic". `Ok` is legitimate (flipping bits of a component value byte yields a
// different valid stream). hecs 0.11.0 VIOLATES the no-panic half for the column format
// (finding 4: corrupted entity ids can collide and panic/UB in `spawn_column_batch_at`),
// so we run `deserialize` inside `catch_unwind` and assert only the documented contract:
// any `Ok` world must be self-consistent (observable without duplicate handles); a panic
// is recorded but tolerated, never asserted correct. The row format is expected to never
// panic here, so its wrapper additionally asserts `!panicked`.
//
// Corruption is a low-2-bit XOR at up to 3 drawn positions. This still reaches invalid
// enum discriminants (0..=2 ^ 3 -> unknown variant), invalid entity bits (generation 0),
// colliding entity ids, varint-marker mutations (0xFD entity markers become 0xFC/0xFE),
// framing shifts, and value corruption, while provably bounding every count/id the parser
// can derive from a small world's stream -- full byte overwrites can commit tens of GB
// via `spawn_at`/`reserve` on this crate's trusted-integer paths (file comment, finding 5).

fn corrupt(tc: &hegel::TestCase, bytes: &mut [u8]) {
    let flips = tc.draw(gs::integers::<u32>().min_value(1).max_value(3));
    for _ in 0..flips {
        let pos = tc.draw(gs::integers::<usize>().min_value(0).max_value(bytes.len() - 1));
        let mask = tc.draw(gs::integers::<u8>().min_value(1).max_value(3));
        bytes[pos] ^= mask;
    }
}

/// Deserialize under `catch_unwind`; assert any `Ok` world is self-consistent. Returns
/// `true` if `deserialize` panicked (tolerated for column, see finding 4).
fn deserialize_corrupted(bytes: &[u8], column_format: bool) -> bool {
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        if column_format {
            column_from_bytes(bytes)
        } else {
            row_from_bytes(bytes)
        }
    }));
    match outcome {
        Ok(Ok(restored)) => {
            // Corruption may still be a valid stream; the world must at least be usable
            // and free of duplicate handles (observe asserts uniqueness).
            let _ = observe(&restored);
            false
        }
        Ok(Err(_)) => false,
        Err(_) => true,
    }
}

#[hegel::test(test_cases = 400)]
fn row_corrupted_input_does_not_panic(tc: hegel::TestCase) {
    let world = arbitrary_world(&tc, 10);
    let mut bytes = row_to_bytes(&world);
    if bytes.is_empty() {
        return;
    }
    corrupt(&tc, &mut bytes);
    let panicked = deserialize_corrupted(&bytes, false);
    assert!(!panicked, "row deserialize panicked on corrupted input");
}

#[hegel::test(test_cases = 400)]
fn column_corrupted_input_stays_memory_safe(tc: hegel::TestCase) {
    // Documented contract only: never a wrong/duplicated Ok world. A panic is the known
    // finding-4 behaviour (a debug-mode guard over release-mode UB) and is tolerated here
    // rather than asserted correct.
    let world = arbitrary_world(&tc, 10);
    let mut bytes = column_to_bytes(&world);
    if bytes.is_empty() {
        return;
    }
    corrupt(&tc, &mut bytes);
    let _panicked = deserialize_corrupted(&bytes, true);
}

// Hand-crafted malformed streams hitting specific validation paths.

#[test]
fn row_rejects_zero_entity_bits() {
    // Entity bits 0 has generation 0: Entity::from_bits returns None, so the Entity
    // Deserialize impl must reject it. Stream: map len 1, key bits 0, empty component map.
    let mut bytes = Vec::new();
    bytes.extend(enc(&1u64)); // world map length
    bytes.extend(enc(&0u64)); // entity bits: invalid (generation 0)
    bytes.extend(enc(&0u64)); // component map length
    let result = row_from_bytes(&bytes);
    assert!(
        result.is_err(),
        "row deserialize accepted entity bits 0 (invalid generation)"
    );
}

#[test]
fn row_rejects_unknown_component_id() {
    // A component key with variant index 99 (ComponentId has variants 0..=2).
    let mut bytes = Vec::new();
    bytes.extend(enc(&1u64)); // world map length
    bytes.extend(enc(&(1u64 << 32))); // entity bits: generation 1, id 0 (valid)
    bytes.extend(enc(&1u64)); // component map length
    bytes.extend(enc(&99u32)); // unknown ComponentId variant
    bytes.extend(enc(&5i32)); // would-be component payload
    let result = row_from_bytes(&bytes);
    assert!(result.is_err(), "row deserialize accepted an unknown component id");
}

#[test]
fn column_rejects_duplicate_component_id() {
    // Archetype declaring the SAME component id twice: ColumnBatchType dedups the
    // type, so the second column's first value fails BatchWriter::push and must
    // surface as the "extra component" error, not a panic.
    let mut bytes = Vec::new();
    bytes.extend(enc(&1u64)); // one archetype
    bytes.extend(enc(&1u32)); // entity_count = 1
    bytes.extend(enc(&2u32)); // component_count = 2
    bytes.extend(enc(&0u32)); // ComponentId::A
    bytes.extend(enc(&0u32)); // ComponentId::A again
    bytes.extend(enc(&(1u64 << 32))); // entity bits: generation 1, id 0
    bytes.extend(enc(&5i32)); // column 1: one A value
    bytes.extend(enc(&6i32)); // column 2: one A value (no space left)
    let result = column_from_bytes(&bytes);
    assert!(
        result.is_err(),
        "column deserialize accepted a duplicate component column"
    );
}

#[test]
fn column_rejects_truncated_entity_list() {
    // Declared entity_count = 2 but only one entity id present before EOF.
    let mut bytes = Vec::new();
    bytes.extend(enc(&1u64)); // one archetype
    bytes.extend(enc(&2u32)); // entity_count = 2
    bytes.extend(enc(&0u32)); // component_count = 0
    bytes.extend(enc(&(1u64 << 32))); // only ONE entity id
    let result = column_from_bytes(&bytes);
    assert!(
        result.is_err(),
        "column deserialize accepted an entity list shorter than entity_count"
    );
}

#[test]
fn column_huge_entity_count_must_not_produce_a_world() {
    // FINDING 2 (see file comment): entity_count == u32::MAX trips
    // `assert!(size < u32::MAX)` in ColumnBatchType::into_batch (src/batch.rs), so
    // hecs 0.11.0 PANICS on this malformed input instead of returning Err. The
    // documented contract we assert is only that malformed input never produces an
    // Ok(World); the catch_unwind tolerates the panic without asserting it correct.
    // (Note: before failing, hecs also `Vec::reserve`s entity_count * 8 bytes -- ~34GB
    // virtual -- from the untrusted count; harmless under memory overcommit, fatal
    // without it. Finding 5.)
    let mut bytes = Vec::new();
    bytes.extend(enc(&1u64)); // one archetype
    bytes.extend(enc(&u32::MAX)); // entity_count = u32::MAX
    bytes.extend(enc(&0u32)); // component_count = 0
    let outcome = catch_unwind(AssertUnwindSafe(|| column_from_bytes(&bytes)));
    match outcome {
        Ok(Ok(world)) => panic!(
            "column deserialize returned Ok (a {}-entity world) for entity_count == u32::MAX",
            world.len()
        ),
        Ok(Err(_)) => {} // the contractual outcome
        Err(_) => {
            // Known bug: panic instead of Err. Documented above; not asserted as correct.
        }
    }
}

#[test]
fn column_duplicate_entity_ids_must_not_corrupt() {
    // FINDING 4 (see file comment): a column archetype whose entity-id list repeats an
    // id makes `spawn_column_batch_at` call `alloc_at` twice for that id; the second
    // call returns the entity's EMPTY location {archetype: 0, index: u32::MAX}, so hecs
    // calls `Archetype::remove(u32::MAX, true)` on archetype 0. Debug: `self.len - 1`
    // underflow panic (archetype.rs:321). Release: wraps -> OOB drop/copy at u32::MAX ->
    // UB. Minimal trigger: two componentless entities both with bits `1<<32`.
    // Documented contract: this malformed input must never yield an Ok(World). The panic
    // is the observed (buggy) behaviour; we tolerate it via catch_unwind, not assert it.
    let bits = 1u64 << 32; // generation 1, id 0
    let mut bytes = Vec::new();
    bytes.extend(enc(&1u64)); // one archetype
    bytes.extend(enc(&2u32)); // entity_count = 2
    bytes.extend(enc(&0u32)); // component_count = 0
    // no component ids
    bytes.extend(enc(&bits)); // entity 0
    bytes.extend(enc(&bits)); // entity 1 -- DUPLICATE id
    let outcome = catch_unwind(AssertUnwindSafe(|| column_from_bytes(&bytes)));
    match outcome {
        Ok(Ok(world)) => panic!(
            "column deserialize returned Ok ({} entities) for duplicate entity ids",
            world.len()
        ),
        Ok(Err(_)) => {} // the contractual (non-buggy) outcome
        Err(_) => {
            // Known bug: panic (debug) / UB (release) instead of Err. Not asserted correct.
        }
    }
}

// =====================================================================
// (c) LEAK HUNT on truncated deserialize paths (Drop-tracked component)
// =====================================================================

thread_local! {
    static TRACKED_LIVE: Cell<i64> = const { Cell::new(0) };
}

fn tracked_live() -> i64 {
    TRACKED_LIVE.with(|c| c.get())
}

/// Serde-serializable AND Drop-tracked component. EVERY construction goes through
/// `new()` (including deserialization: the manual Deserialize impl below builds the
/// value via `new()`), so `TRACKED_LIVE` counts exactly the live instances. No Copy,
/// no Clone.
#[derive(Debug)]
struct Tracked(i32);

impl Tracked {
    fn new(v: i32) -> Self {
        TRACKED_LIVE.with(|c| c.set(c.get() + 1));
        Tracked(v)
    }
}

impl Drop for Tracked {
    fn drop(&mut self) {
        TRACKED_LIVE.with(|c| c.set(c.get() - 1));
    }
}

impl Serialize for Tracked {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Tracked {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        i32::deserialize(deserializer).map(Tracked::new)
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
enum TrackedId {
    Tracked,
    B,
}

struct TrackedRowContext;

impl row::SerializeContext for TrackedRowContext {
    fn serialize_entity<S>(&mut self, entity: EntityRef<'_>, mut map: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::SerializeMap,
    {
        row::try_serialize::<Tracked, _, _>(&entity, &TrackedId::Tracked, &mut map)?;
        row::try_serialize::<B, _, _>(&entity, &TrackedId::B, &mut map)?;
        map.end()
    }

    fn component_count(&self, entity: EntityRef<'_>) -> Option<usize> {
        Some(entity.len())
    }
}

impl row::DeserializeContext for TrackedRowContext {
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
                TrackedId::Tracked => {
                    entity.add::<Tracked>(map.next_value()?);
                }
                TrackedId::B => {
                    entity.add::<B>(map.next_value()?);
                }
            }
        }
        Ok(())
    }
}

struct TrackedColumnSerContext;

impl column::SerializeContext for TrackedColumnSerContext {
    fn component_count(&self, archetype: &Archetype) -> usize {
        archetype
            .component_types()
            .filter(|&t| t == TypeId::of::<Tracked>() || t == TypeId::of::<B>())
            .count()
    }

    fn serialize_component_ids<S: serde::ser::SerializeTuple>(
        &mut self,
        archetype: &Archetype,
        mut out: S,
    ) -> Result<S::Ok, S::Error> {
        // Tracked first, so truncation inside a later column finds fully-parsed
        // Tracked values already sitting in the ColumnBatchBuilder.
        column::try_serialize_id::<Tracked, _, _>(archetype, &TrackedId::Tracked, &mut out)?;
        column::try_serialize_id::<B, _, _>(archetype, &TrackedId::B, &mut out)?;
        out.end()
    }

    fn serialize_components<S: serde::ser::SerializeTuple>(
        &mut self,
        archetype: &Archetype,
        mut out: S,
    ) -> Result<S::Ok, S::Error> {
        column::try_serialize::<Tracked, _>(archetype, &mut out)?;
        column::try_serialize::<B, _>(archetype, &mut out)?;
        out.end()
    }
}

#[derive(Default)]
struct TrackedColumnDeContext {
    components: Vec<TrackedId>,
}

impl column::DeserializeContext for TrackedColumnDeContext {
    fn deserialize_component_ids<'de, D>(&mut self, mut seq: D) -> Result<ColumnBatchType, D::Error>
    where
        D: serde::de::SeqAccess<'de>,
    {
        self.components.clear();
        let mut batch = ColumnBatchType::new();
        while let Some(id) = seq.next_element()? {
            match id {
                TrackedId::Tracked => {
                    batch.add::<Tracked>();
                }
                TrackedId::B => {
                    batch.add::<B>();
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
                TrackedId::Tracked => {
                    column::deserialize_column::<Tracked, _>(entity_count, &mut seq, batch)?;
                }
                TrackedId::B => {
                    column::deserialize_column::<B, _>(entity_count, &mut seq, batch)?;
                }
            }
        }
        Ok(())
    }
}

fn tracked_row_to_bytes(world: &World) -> Vec<u8> {
    let mut buf = Vec::new();
    let mut ser = bincode::Serializer::new(&mut buf, bincode::options());
    row::serialize(world, &mut TrackedRowContext, &mut ser).expect("tracked row serialize");
    buf
}

fn tracked_row_from_bytes(bytes: &[u8]) -> Result<World, bincode::Error> {
    let mut de = bincode::Deserializer::with_reader(bytes, bincode::options());
    row::deserialize(&mut TrackedRowContext, &mut de)
}

fn tracked_column_to_bytes(world: &World) -> Vec<u8> {
    let mut buf = Vec::new();
    let mut ser = bincode::Serializer::new(&mut buf, bincode::options());
    column::serialize(world, &mut TrackedColumnSerContext, &mut ser)
        .expect("tracked column serialize");
    buf
}

fn tracked_column_from_bytes(bytes: &[u8]) -> Result<World, bincode::Error> {
    let mut de = bincode::Deserializer::with_reader(bytes, bincode::options());
    column::deserialize(&mut TrackedColumnDeContext::default(), &mut de)
}

/// FINDING 1 (see file comment). Truncated column input makes `deserialize` return
/// `Err` -- asserted -- but every `Tracked` value already deserialized into the
/// internal `ColumnBatchBuilder` is leaked when the builder is dropped on the error
/// path. Observed with this exact test (hecs 0.11.0):
///   * minimal trigger: 1 entity with `(Tracked, B)` (Tracked serialized first), final
///     byte truncated -> the Tracked column parses fully, the B value hits EOF, and the
///     1 parsed Tracked is leaked (+1). (A `Tracked`-only single entity does NOT leak on
///     final-byte truncation: the sole value fails to parse before it is ever
///     constructed -- the leak needs a fully-parsed column followed by a later failure.)
///   * 4 entities with (Tracked, B), all strict prefixes -> up to 4 leaks per attempt
///     (cuts inside/after the B column leak all 4 parsed Tracked values); this test
///     observed 22 leaked Tracked total across 7 leaking prefixes.
/// Per the known-bug protocol we assert only the documented contract here: Err on
/// truncation, and never a NEGATIVE delta (a double-drop would be a new, worse bug).
/// The leak totals are printed (visible with --nocapture) rather than asserted.
#[test]
fn column_truncated_input_errors_and_leaks_observation() {
    // Minimal LEAKING trigger: one entity carrying (Tracked, B); Tracked is serialized
    // before B, so truncating the trailing B byte leaves one fully-parsed Tracked in the
    // ColumnBatchBuilder when it is dropped on the error path.
    let minimal_leak;
    {
        let mut world = World::new();
        world.spawn((Tracked::new(7), B(1)));
        let bytes = tracked_column_to_bytes(&world);
        let before = tracked_live();
        let result = tracked_column_from_bytes(&bytes[..bytes.len() - 1]);
        assert!(result.is_err(), "column deserialize of truncated stream succeeded");
        minimal_leak = tracked_live() - before;
        assert!(
            minimal_leak >= 0,
            "double drop: live Tracked count fell by {} after a failed column deserialize",
            -minimal_leak
        );
    }

    // Broader sweep: every strict prefix of a 4-entity (Tracked, B) archetype.
    let mut total_leaked = 0;
    let mut max_leaked = 0;
    let mut leaking_prefixes = 0;
    {
        let mut world = World::new();
        for i in 0..4 {
            world.spawn((Tracked::new(i), B(-i)));
        }
        let bytes = tracked_column_to_bytes(&world);
        for cut in 0..bytes.len() {
            let before = tracked_live();
            let result = tracked_column_from_bytes(&bytes[..cut]);
            assert!(
                result.is_err(),
                "column deserialize of strict prefix {}/{} succeeded",
                cut,
                bytes.len()
            );
            let delta = tracked_live() - before;
            assert!(
                delta >= 0,
                "double drop at prefix {}: live Tracked count fell by {}",
                cut,
                -delta
            );
            total_leaked += delta;
            max_leaked = max_leaked.max(delta);
            if delta > 0 {
                leaking_prefixes += 1;
            }
        }

        // The happy path must NOT leak: full parse constructs 4, dropping the restored
        // world destroys exactly those 4.
        let before_full = tracked_live();
        let restored = tracked_column_from_bytes(&bytes).expect("full column parse");
        assert_eq!(
            tracked_live() - before_full,
            4,
            "full column parse should construct exactly the 4 Tracked components"
        );
        drop(restored);
        assert_eq!(
            tracked_live(),
            before_full,
            "dropping the restored world leaked or double-dropped Tracked components"
        );
    }

    eprintln!(
        "column truncation leak observation: minimal trigger leaked {minimal_leak}, \
         sweep leaked {total_leaked} total across {leaking_prefixes} leaking prefixes \
         (max {max_leaked} per attempt)"
    );
}

/// Row-format counterpart: `EntityBuilder` (unlike `ColumnBatchBuilder`) drops its
/// buffered components correctly, so the row error path must not leak. This is a hard
/// assertion -- if it ever fails, that is a NEW bug.
#[test]
fn row_truncated_input_errors_and_does_not_leak() {
    let mut world = World::new();
    for i in 0..4 {
        world.spawn((Tracked::new(i), B(-i)));
    }
    let bytes = tracked_row_to_bytes(&world);
    let baseline = tracked_live();
    for cut in 0..bytes.len() {
        let result = tracked_row_from_bytes(&bytes[..cut]);
        assert!(
            result.is_err(),
            "row deserialize of strict prefix {}/{} succeeded",
            cut,
            bytes.len()
        );
        assert_eq!(
            tracked_live(),
            baseline,
            "row deserialize of strict prefix {} leaked or double-dropped Tracked components",
            cut
        );
    }

    // Happy path: full parse constructs 4, dropping the restored world releases them.
    let restored = tracked_row_from_bytes(&bytes).expect("full row parse");
    assert_eq!(tracked_live(), baseline + 4);
    drop(restored);
    assert_eq!(tracked_live(), baseline);
    drop(world);
    assert_eq!(tracked_live(), baseline - 4, "original world did not release its Tracked components");
}

// =====================================================================
// (d) serialize_satisfying
// =====================================================================

fn expected_satisfying_a(world: &World) -> HashMap<Entity, Obs> {
    observe(world)
        .into_iter()
        .filter(|(_, obs)| obs.0.is_some())
        .collect()
}

#[hegel::test(test_cases = 300)]
fn row_serialize_satisfying_keeps_exactly_matching_entities(tc: hegel::TestCase) {
    let world = arbitrary_world(&tc, 40);
    let bytes = row_satisfying_to_bytes::<&A>(&world);
    let restored = row_from_bytes(&bytes).expect("row deserialize of satisfying subset");
    assert_eq!(
        expected_satisfying_a(&world),
        observe(&restored),
        "row serialize_satisfying::<&A> did not round-trip exactly the A-bearing entities"
    );
}

#[hegel::test(test_cases = 300)]
fn column_serialize_satisfying_keeps_exactly_matching_entities(tc: hegel::TestCase) {
    let world = arbitrary_world(&tc, 40);
    let bytes = column_satisfying_to_bytes::<&A>(&world);
    let restored = column_from_bytes(&bytes).expect("column deserialize of satisfying subset");
    assert_eq!(
        expected_satisfying_a(&world),
        observe(&restored),
        "column serialize_satisfying::<&A> did not round-trip exactly the A-bearing entities"
    );
}
