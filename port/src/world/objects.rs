//! Track placements, random pickups, six sequence hazards and special surfaces.
//! Native owned object services; independent source derivations: objects.trace.md.
use super::{terrain::Projection, tracks::TRUCKS};
use crate::{
    runtime::RandomState,
    trucks::motion::{Truck, WorkingMotion},
};
use anyhow::{Context, Result, ensure};

mod pickup;
pub use pickup::{PickupAssets, PickupKind, PickupPhase, PickupReward};

pub const DYNAMIC_SLOTS: usize = 6;
pub const PICKUP_CHOICES: usize = 4;
pub const PICKUP_TYPES: usize = 2;
pub const PICKUP_TYPE_CAP: u16 = 4;
pub const PICKUP_AWARD_LIFETIME: u16 = 80;
pub const DYNAMIC_RESPONSE_TICKS: u16 = 44;
pub const PLACEMENT_REACTION: u16 = 55;
pub const COMPOSITE_REACTION: u16 = 47;
pub const COMPOSITE_RECORDS: usize = 4;
pub const HIDDEN_COORDINATES: u16 = 0xf000;
pub const DYNAMIC_IDLE_TILE: u16 = 0x2520;
pub const DYNAMIC_EXHAUSTED_TILE: u16 = 0x256e;
pub const PICKUP_NITRO_SOUND: u16 = 0x0602;
pub const PICKUP_MONEY_SOUND: u16 = 0x0502;
pub const DYNAMIC_CONTACT_SOUND: u16 = 0x0702;
pub const SURFACE_CONTACT_SOUND: u16 = 0x0802;

/// Source world words retain byte heights at the projection boundary only.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Placement {
    pub world: [u16; 2],
    pub height: u16,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectSprite {
    pub coordinates: u16,
    pub tile_attributes: u16,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlacedObject {
    pub placement: Placement,
    pub animation: u16,
    pub sprite: ObjectSprite,
}
/// An early negative X stops writes but does not change the source count.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PlacementList {
    pub declared_count: u16,
    pub prefix: Vec<Placement>,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ObjectContent {
    pub placements: PlacementList,
    pub pickups: [Placement; PICKUP_CHOICES],
    /// Original order-dependent groups are resolved before constructing this
    /// selected content; added tracks supply named groups without a fixed count.
    pub dynamic: [Placement; DYNAMIC_SLOTS],
    pub composites: Option<[Placement; COMPOSITE_RECORDS]>,
    pub track_three_sprites: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectTables {
    pub pickups: PickupAssets,
    pub dynamic_vertical: [u16; 3],
    pub dynamic_y_deltas: [i8; DYNAMIC_RESPONSE_TICKS as usize],
    pub dynamic_tiles: [u16; 35],
    pub placement_tiles: [u16; 56],
    pub composite_tiles: [[u16; 3]; 48],
    pub track_three_coordinates: [[u8; 2]; 3],
}
pub fn load_object_tables(rom: &[u8]) -> Result<ObjectTables> {
    const AWARD_ROM: usize = 0x48db;
    const PRIMARY_TILES_ROM: usize = 0x48f1;
    const EXTRA_TILES_ROM: usize = 0x48df;
    const DYNAMIC_VERTICAL_ROM: usize = 0x4dc0;
    const DYNAMIC_DELTA_ROM: usize = 0x4dc6;
    const DYNAMIC_TILES_ROM: usize = 0x4df2;
    const PLACEMENT_TILES_ROM: usize = 0x5229;
    const COMPOSITE_TILES_ROM: usize = 0x502e;
    const THREE_COORDINATES_ROM: usize = 0x4ef9;
    fn words<const N: usize>(rom: &[u8], at: usize) -> Result<[u16; N]> {
        let bytes = rom
            .get(at..at + N * 2)
            .context("truncated object word table")?;
        Ok(std::array::from_fn(|index| {
            u16::from_le_bytes([bytes[index * 2], bytes[index * 2 + 1]])
        }))
    }
    let deltas = rom
        .get(DYNAMIC_DELTA_ROM..DYNAMIC_DELTA_ROM + DYNAMIC_RESPONSE_TICKS as usize)
        .context("truncated object deltas")?;
    let composite: [u16; 144] = words(rom, COMPOSITE_TILES_ROM)?;
    let coordinates = rom
        .get(THREE_COORDINATES_ROM..THREE_COORDINATES_ROM + 6)
        .context("truncated special sprite coordinates")?;
    Ok(ObjectTables {
        pickups: PickupAssets::decode(
            words(rom, AWARD_ROM)?,
            words(rom, PRIMARY_TILES_ROM)?,
            words(rom, EXTRA_TILES_ROM)?,
        ),
        dynamic_vertical: words(rom, DYNAMIC_VERTICAL_ROM)?,
        dynamic_y_deltas: std::array::from_fn(|index| deltas[index] as i8),
        dynamic_tiles: words(rom, DYNAMIC_TILES_ROM)?,
        placement_tiles: words(rom, PLACEMENT_TILES_ROM)?,
        composite_tiles: std::array::from_fn(|frame| {
            std::array::from_fn(|part| composite[frame * 3 + part])
        }),
        track_three_coordinates: std::array::from_fn(|part| {
            [coordinates[part], coordinates[part + 3]]
        }),
    })
}
pub fn decode_placements(bytes: &[u8]) -> Result<PlacementList> {
    let declared_count = read_word(bytes, 0)?;
    ensure!(
        declared_count != 0 && (declared_count as i16) > 0,
        "invalid object count"
    );
    let mut prefix = Vec::new();
    for index in 0..usize::from(declared_count) {
        let at = 2 + index * RECORD_BYTES;
        if (read_word(bytes, at)? as i16) < 0 {
            break;
        }
        prefix.push(read_placement(bytes, at)?);
    }
    Ok(PlacementList {
        declared_count,
        prefix,
    })
}
pub fn load_original_content(rom: &[u8], track: u8, order_slot: u8) -> Result<ObjectContent> {
    const PLACEMENT_POINTERS: usize = 0x5299;
    const PICKUP_RECORDS: usize = 0x46f3;
    const DYNAMIC_POINTERS: usize = 0x4984;
    const COMPOSITE_POINTER: usize = 0x514e;
    ensure!(
        usize::from(track) < super::tracks::ORIGINAL_TRACK_COUNT
            && usize::from(order_slot) < super::tracks::ORDER_SLOTS,
        "invalid original object selector"
    );
    let pointer = |table, index| -> Result<usize> {
        super::tracks::physical(super::tracks::CpuSource(u32::from(read_word(
            rom,
            table + index * 2,
        )?)))
    };
    let start = pointer(PLACEMENT_POINTERS, usize::from(track))?;
    let placements = decode_placements(rom.get(start..).context("placement pointer beyond ROM")?)?;
    let mut pickups = [Placement::default(); PICKUP_CHOICES];
    for (choice, placement) in pickups.iter_mut().enumerate() {
        *placement = read_placement(
            rom,
            PICKUP_RECORDS + (usize::from(track) * PICKUP_CHOICES + choice) * RECORD_BYTES,
        )?;
    }
    let start = pointer(DYNAMIC_POINTERS, usize::from(order_slot))?;
    let mut dynamic = [Placement::default(); DYNAMIC_SLOTS];
    for (index, placement) in dynamic.iter_mut().rev().enumerate() {
        *placement = read_placement(rom, start + index * RECORD_BYTES)?;
    }
    let composites = if track == 0 {
        let start = pointer(COMPOSITE_POINTER, 0)?;
        let list = decode_placements(rom.get(start..).context("composite pointer beyond ROM")?)?;
        ensure!(
            usize::from(list.declared_count) == COMPOSITE_RECORDS
                && list.prefix.len() == COMPOSITE_RECORDS,
            "invalid original composite records"
        );
        Some(
            list.prefix
                .try_into()
                .map_err(|_| anyhow::anyhow!("invalid composite record count"))?,
        )
    } else {
        None
    };
    Ok(ObjectContent {
        placements,
        pickups,
        dynamic,
        composites,
        track_three_sprites: track == 3,
    })
}
/// Game-owned added/override content must be fully validated before mutation.
pub fn validate_content(content: &ObjectContent) -> Result<()> {
    const SORTABLE_PLACEMENTS: usize = crate::presentation::SORTED_SPRITES - TRUCKS;
    ensure!(
        content.placements.declared_count != 0
            && content.placements.prefix.len() <= usize::from(content.placements.declared_count)
            && usize::from(content.placements.declared_count) <= SORTABLE_PLACEMENTS,
        "invalid placement prefix/count"
    );
    Ok(())
}
pub fn apply_placements(objects: &mut Vec<PlacedObject>, list: &PlacementList) -> Result<()> {
    ensure!(
        list.declared_count != 0 && list.prefix.len() <= usize::from(list.declared_count),
        "invalid placement prefix/count"
    );
    objects.resize(usize::from(list.declared_count), PlacedObject::default());
    for (object, &placement) in objects.iter_mut().zip(&list.prefix) {
        object.placement = placement;
        object.animation = 0;
    }
    Ok(())
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pickup {
    #[serde(rename = "state")]
    pub phase: PickupPhase,
    pub placement: Placement,
    #[serde(with = "crate::snapshot::array")]
    pub counts: [u16; PICKUP_TYPES],
    pub lifetime: u16,
    #[serde(with = "crate::snapshot::array")]
    pub sprites: [ObjectSprite; 2],
}
pub fn pickup_near(pickup: Placement, truck: [u16; 2]) -> bool {
    const X_OFFSET: u16 = 0x180;
    const X_WIDTH: u16 = 0x300;
    const Y_LIFT: u16 = 0xc0;
    const Y_OFFSET: u16 = 0x280;
    const Y_WIDTH: u16 = 0x500;
    pickup.world[0]
        .wrapping_sub(truck[0])
        .wrapping_add(X_OFFSET)
        < X_WIDTH
        && pickup.world[1]
            .wrapping_add(Y_LIFT)
            .wrapping_sub(truck[1])
            .wrapping_add(Y_OFFSET)
            < Y_WIDTH
}
pub fn try_spawn_pickup(
    pickup: &mut Pickup,
    choices: &[Placement; PICKUP_CHOICES],
    gate: u8,
    choice: u8,
    animation: u8,
    projection: Projection,
    tables: &ObjectTables,
) {
    if gate != 0 {
        return;
    }
    pickup.placement = choices[usize::from(choice & 3)];
    let kind = PickupKind::for_frame(animation);
    if pickup.counts[kind.index()] >= PICKUP_TYPE_CAP {
        return;
    }
    pickup.phase = PickupPhase::Available(kind);
    // New spawn state1/2 is within the validated finite tile domain.
    project_pickup(pickup, projection, tables).expect("validated initial pickup state");
}
pub fn project_pickup(
    pickup: &mut Pickup,
    projection: Projection,
    tables: &ObjectTables,
) -> Result<()> {
    const PAIR_CENTER: u16 = 8;
    const PAIR_SEPARATION: u16 = 0x10;
    let art = tables.pickups.art(pickup.phase)?;
    pickup.sprites[0] = ObjectSprite {
        coordinates: screen_coordinates(pickup.placement, projection),
        tile_attributes: art.primary,
    };
    if let Some(extra) = art.extra {
        pickup.sprites[0].coordinates = pickup.sprites[0].coordinates.wrapping_sub(PAIR_CENTER);
        pickup.sprites[1] = ObjectSprite {
            coordinates: pickup.sprites[0].coordinates.wrapping_add(PAIR_SEPARATION),
            tile_attributes: extra,
        };
    }
    Ok(())
}
pub fn collect_pickup(
    pickup: &mut Pickup,
    trucks: &mut [Truck; TRUCKS],
    pending_awards: &mut [u8; TRUCKS],
    animation: u8,
    active_humans: u8,
    projection: Projection,
    tables: &ObjectTables,
) -> Result<Vec<u16>> {
    let PickupPhase::Available(kind) = pickup.phase else {
        anyhow::bail!("invalid collectable pickup");
    };
    let slot = usize::from(animation & 3);
    if !pickup_near(pickup.placement, trucks[slot].world) {
        return Ok(Vec::new());
    }
    let old_count = pickup.counts[kind.index()];
    ensure!(old_count <= PICKUP_TYPE_CAP, "unsupported pickup count");
    if old_count < PICKUP_TYPE_CAP {
        pickup.counts[kind.index()] += 1;
    }
    let collection = tables.pickups.collection(kind, old_count)?;
    pickup.phase = collection.phase;
    let command = match collection.reward {
        PickupReward::Money(units) => {
            pending_awards[slot] = pending_awards[slot].wrapping_add(units);
            PICKUP_MONEY_SOUND
        }
        PickupReward::Nitro(charges) => {
            trucks[slot].nitro.award_pickup(charges);
            PICKUP_NITRO_SOUND
        }
    };
    project_pickup(pickup, projection, tables)?;
    pickup.lifetime = PICKUP_AWARD_LIFETIME;
    tick_pickup_award(pickup, animation);
    Ok(if active_humans != 0 {
        vec![command]
    } else {
        Vec::new()
    })
}
pub fn tick_pickup_award(pickup: &mut Pickup, animation: u8) {
    const Y_STEP: u16 = 0x100;
    if animation & 3 == 0 {
        for sprite in &mut pickup.sprites {
            sprite.coordinates = sprite.coordinates.wrapping_sub(Y_STEP);
        }
    }
    pickup.lifetime = pickup.lifetime.wrapping_sub(1);
    if pickup.lifetime == 0 {
        pickup.phase = PickupPhase::Absent;
        for sprite in &mut pickup.sprites {
            sprite.coordinates = HIDDEN_COORDINATES;
        }
    }
}
// Explicit disjoint owner borrows keep mutation authority visible at this boundary.
#[allow(clippy::too_many_arguments)]
pub fn service_pickup(
    pickup: &mut Pickup,
    choices: &[Placement; PICKUP_CHOICES],
    trucks: &mut [Truck; TRUCKS],
    pending_awards: &mut [u8; TRUCKS],
    animation: u8,
    active_humans: u8,
    random: &mut RandomState,
    projection: Projection,
    tables: &ObjectTables,
) -> Result<Vec<u16>> {
    use crate::runtime::RandomResultWidth;
    match pickup.phase {
        PickupPhase::Absent => {
            let gate = random.next(RandomResultWidth::Byte) as u8;
            if gate == 0 {
                let choice = random.next(RandomResultWidth::Byte) as u8;
                try_spawn_pickup(pickup, choices, gate, choice, animation, projection, tables);
            }
            Ok(Vec::new())
        }
        PickupPhase::Available(_) => collect_pickup(
            pickup,
            trucks,
            pending_awards,
            animation,
            active_humans,
            projection,
            tables,
        ),
        _ => {
            tick_pickup_award(pickup, animation);
            Ok(Vec::new())
        }
    }
}

mod hazard;
use hazard::ResponseTick;
pub use hazard::{DynamicObject, HazardStage, ReactionBudget, ResponseClock};
pub fn initialize_dynamic(
    objects: &mut [DynamicObject; DYNAMIC_SLOTS],
    records: &[Placement; DYNAMIC_SLOTS],
    random: &mut RandomState,
    projection: Projection,
) {
    use crate::runtime::RandomResultWidth;
    const USE_MASK: u16 = 7;
    for slot in (0..DYNAMIC_SLOTS).rev() {
        objects[slot].stage = HazardStage::Inactive;
        if records[slot].world[0] == 0 {
            continue;
        }
        objects[slot].placement = records[slot];
        objects[slot].reactions =
            ReactionBudget::remaining(1 + (random.next(RandomResultWidth::Byte) & USE_MASK));
        objects[slot].animation = 0;
        objects[slot].response = ResponseClock::default();
        objects[slot].stage = HazardStage::Reactive;
        objects[slot].sprite = ObjectSprite {
            coordinates: screen_coordinates(records[slot], projection),
            tile_attributes: DYNAMIC_IDLE_TILE,
        };
    }
}
pub fn dynamic_near(
    object: &DynamicObject,
    truck: [u16; 2],
    tables: &ObjectTables,
) -> Result<bool> {
    const X_OFFSET: u16 = 0x100;
    const WIDTH: u16 = 0x300;
    const Y_LIFT: u16 = 0xc0;
    let index = object
        .stage
        .collision_index()
        .context("invalid active dynamic kind")?;
    let vertical = tables.dynamic_vertical[index];
    Ok(object.placement.world[0]
        .wrapping_sub(truck[0])
        .wrapping_add(X_OFFSET)
        < WIDTH
        && object.placement.world[1]
            .wrapping_add(Y_LIFT)
            .wrapping_sub(truck[1])
            .wrapping_add(vertical)
            < WIDTH)
}
pub fn service_dynamic(
    objects: &mut [DynamicObject; DYNAMIC_SLOTS],
    trucks: &mut [Truck; TRUCKS],
    animation: u8,
    active_humans: u8,
    tables: &ObjectTables,
) -> Result<Vec<u16>> {
    const CONTACT_AGAIN: u16 = 0x12;
    const TYPE_ONE_REACTION: u16 = 0x22;
    const TYPE_TWO_REACTION: u16 = 0x11;
    const TYPE_THREE_REACTION: u16 = 0x1d;
    const REACTION_PALETTE: u16 = 0x1000;
    const HOLD_REACTION_THRESHOLD: u16 = 0x1f;
    let slot = usize::from(animation & 3);
    let mut sounds = Vec::new();
    for object in objects.iter_mut().rev() {
        if object.stage == HazardStage::Inactive {
            continue;
        }
        if object.animation == 0 || object.animation == CONTACT_AGAIN {
            if !dynamic_near(object, trucks[slot].world, tables)? {
                continue;
            }
            match object.stage {
                HazardStage::Reactive => {
                    object.response = ResponseClock::start(DYNAMIC_RESPONSE_TICKS);
                    object.animation = TYPE_ONE_REACTION;
                    trucks[slot].speed = 0;
                    if active_humans != 0 {
                        sounds.push(DYNAMIC_CONTACT_SOUND);
                    }
                }
                HazardStage::Depleted => {
                    object.stage = HazardStage::Struck;
                    trucks[slot].speed = 0;
                    object.animation = TYPE_TWO_REACTION;
                }
                HazardStage::Struck => {
                    if trucks[slot].speed == 0 {
                        continue;
                    }
                    object.animation = TYPE_THREE_REACTION;
                }
                _ => anyhow::bail!("invalid dynamic kind"),
            }
        }
        object.sprite.tile_attributes = *tables
            .dynamic_tiles
            .get(usize::from(object.animation))
            .context("invalid dynamic animation")?
            | if object.response.highlighted() {
                REACTION_PALETTE
            } else {
                0
            };
        if animation & 3 == 0 && object.animation != 0 {
            object.animation -= 1;
        }
        if object.stage != HazardStage::Reactive {
            continue;
        }
        match object.response.tick() {
            ResponseTick::Complete(legacy_timer) => {
                if object.animation == TYPE_THREE_REACTION {
                    object.response.defer_completion(legacy_timer);
                    continue;
                }
                object.animation = 0;
                object.response = ResponseClock::default();
                object.sprite.tile_attributes = DYNAMIC_IDLE_TILE;
                if !object.reactions.finish_response() {
                    object.sprite.tile_attributes = DYNAMIC_EXHAUSTED_TILE;
                    object.stage = HazardStage::Depleted;
                }
            }
            ResponseTick::Remaining(ticks) => {
                if object.animation < HOLD_REACTION_THRESHOLD {
                    object.animation = TYPE_ONE_REACTION;
                }
                let delta = *tables
                    .dynamic_y_deltas
                    .get(usize::from(ticks))
                    .context("invalid dynamic response timer")?;
                let [x, y] = object.sprite.coordinates.to_le_bytes();
                object.sprite.coordinates = u16::from_le_bytes([x, y.wrapping_add(delta as u8)]);
            }
        }
    }
    Ok(sounds)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContactAnimation {
    pub cursor: u8,
    pub sprite: ObjectSprite,
}
/// Uses staged previous screen position, while mutating current proposed speed.
/// All four probe marker bits must occupy history's high nibble.
pub fn special_surface(
    truck: &Truck,
    working: &mut WorkingMotion,
    history: u8,
    contact: &mut ContactAnimation,
    active_humans: u8,
) -> Vec<u16> {
    const SURFACE_MARKERS: u8 = 0xf0;
    const FIRST_EFFECT: u8 = 10;
    const RESUMABLE_CURSORS: [u8; 2] = [18, 23];
    const RESUME_EFFECT: u8 = 19;
    if truck.contact_marker == 0 || history & SURFACE_MARKERS != SURFACE_MARKERS {
        return Vec::new();
    }
    let starting = contact.cursor == 0;
    if !starting && !RESUMABLE_CURSORS.contains(&contact.cursor) {
        return Vec::new();
    }
    if (working.speed as u8 as i8) >= 0 {
        working.speed =
            (working.speed & !u16::from(u8::MAX)) | u16::from((working.speed as u8) >> 1);
    }
    contact.sprite.coordinates = u16::from_le_bytes([truck.screen_x as u8, truck.screen_y]);
    contact.cursor = if starting {
        FIRST_EFFECT
    } else {
        RESUME_EFFECT
    };
    if starting && active_humans != 0 {
        vec![SURFACE_CONTACT_SOUND]
    } else {
        Vec::new()
    }
}
pub fn track_three_sprites(
    enabled: bool,
    tables: &ObjectTables,
    staging: &mut [ObjectSprite; 3],
    upper: &mut [ObjectSprite; 3],
) {
    const BASE_TILE: u16 = 0x228c;
    const UPPER_TILE: u16 = 0x208a;
    const UPPER_OFFSET: u8 = 0x10;
    if !enabled {
        return;
    }
    for part in 0..3 {
        let [x, y] = tables.track_three_coordinates[part];
        staging[part] = ObjectSprite {
            coordinates: u16::from_le_bytes([x, y]),
            tile_attributes: BASE_TILE,
        };
        upper[part] = ObjectSprite {
            coordinates: u16::from_le_bytes([x, y.wrapping_sub(UPPER_OFFSET)]),
            tile_attributes: UPPER_TILE,
        };
    }
}
pub fn placement_near(placement: Placement, truck: &Truck) -> bool {
    const MOTION_MASK: u16 = 0x7f;
    const X_OFFSET: u16 = 0x140;
    const X_WIDTH: u16 = 0x280;
    const Y_OFFSET: u16 = 0x80;
    const Y_WIDTH: u16 = 0x1c0;
    truck.speed & MOTION_MASK != 0
        && placement.world[0]
            .wrapping_sub(truck.world[0])
            .wrapping_add(X_OFFSET)
            < X_WIDTH
        && placement.world[1]
            .wrapping_sub(truck.world[1])
            .wrapping_add(Y_OFFSET)
            < Y_WIDTH
}
pub fn service_placements(
    objects: &mut [PlacedObject],
    trucks: &[Truck; TRUCKS],
    animation: u8,
    projection: Projection,
    tables: &ObjectTables,
) -> Result<()> {
    for object in objects {
        let tile = *tables
            .placement_tiles
            .get(usize::from(object.animation))
            .context("invalid placement reaction")?;
        object.sprite = ObjectSprite {
            coordinates: screen_coordinates(object.placement, projection),
            tile_attributes: tile,
        };
        if animation & 3 == 0 && object.animation != 0 {
            object.animation -= 1;
        }
        if object.animation == 0
            && placement_near(object.placement, &trucks[usize::from(animation & 3)])
        {
            object.animation = PLACEMENT_REACTION;
        }
    }
    Ok(())
}
/// Four source-defined records; the loop's extra iteration reads a derived tail
/// view of the first record's adjacent fields, not an emulated RAM image.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Composites {
    #[serde(with = "crate::snapshot::array")]
    pub records: [PlacedObject; COMPOSITE_RECORDS],
    pub count: u16,
    pub sprites: Vec<[ObjectSprite; 3]>,
}
pub fn composite_near(placement: Placement, truck: &Truck) -> bool {
    const MOTION_MASK: u16 = 0x7f;
    const X_SHIFT: u16 = 0x200;
    const X_OFFSET: u16 = 0x300;
    const X_WIDTH: u16 = 0x600;
    const Y_SHIFT: u16 = 0x100;
    const Y_OFFSET: u16 = 0x80;
    const Y_WIDTH: u16 = 0x1c0;
    truck.speed & MOTION_MASK != 0
        && placement.world[0]
            .wrapping_add(X_SHIFT)
            .wrapping_sub(truck.world[0])
            .wrapping_add(X_OFFSET)
            < X_WIDTH
        && placement.world[1]
            .wrapping_sub(Y_SHIFT)
            .wrapping_sub(truck.world[1])
            .wrapping_add(Y_OFFSET)
            < Y_WIDTH
}
pub fn composite_tail(state: &Composites) -> PlacedObject {
    let first = &state.records[0];
    PlacedObject {
        placement: Placement {
            world: [first.placement.world[1], first.placement.height],
            height: first.animation,
        },
        animation: state.count,
        sprite: ObjectSprite::default(),
    }
}
pub fn service_composites(
    state: &mut Composites,
    trucks: &[Truck; TRUCKS],
    animation: u8,
    projection: Projection,
    tables: &ObjectTables,
) -> Result<()> {
    const PART_SPACING: u8 = 0x10;
    ensure!(
        usize::from(state.count) <= COMPOSITE_RECORDS,
        "unsupported composite count"
    );
    let count = usize::from(state.count) + 1;
    state.sprites.resize(count, [ObjectSprite::default(); 3]);
    for index in 0..count {
        let mut object = if index == COMPOSITE_RECORDS {
            composite_tail(state)
        } else {
            state.records[index].clone()
        };
        let tiles = *tables
            .composite_tiles
            .get(usize::from(object.animation))
            .context("invalid composite animation")?;
        let [x, y] = screen_coordinates(object.placement, projection).to_le_bytes();
        state.sprites[index] = std::array::from_fn(|part| ObjectSprite {
            coordinates: u16::from_le_bytes([x.wrapping_add(PART_SPACING * part as u8), y]),
            tile_attributes: tiles[part],
        });
        if animation & 3 == 0 && object.animation != 0 {
            object.animation -= 1;
        }
        if object.animation == 0
            && composite_near(object.placement, &trucks[usize::from(animation & 3)])
        {
            object.animation = COMPOSITE_REACTION;
        }
        if index == COMPOSITE_RECORDS {
            state.count = object.animation;
        } else {
            state.records[index].animation = object.animation;
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldObjects {
    pub pickup: Pickup,
    #[serde(with = "crate::snapshot::array")]
    pub pending_awards: [u8; TRUCKS],
    #[serde(with = "crate::snapshot::array")]
    pub dynamic: [DynamicObject; DYNAMIC_SLOTS],
    pub placements: Vec<PlacedObject>,
    pub composites: Option<Composites>,
    #[serde(with = "crate::snapshot::array")]
    pub contacts: [ContactAnimation; TRUCKS],
    #[serde(with = "crate::snapshot::array")]
    pub special_staging: [ObjectSprite; 3],
    #[serde(with = "crate::snapshot::array")]
    pub special_upper: [ObjectSprite; 3],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectPhase {
    Composite,
    Pickup,
    Dynamic,
    Placements,
}
/// Mutate the actual object/truck/award owners, not merely a phase log. Original
/// initialization calls this once before clearing pickup counts/awards.
// Explicit disjoint owner borrows keep mutation authority visible at this boundary.
#[allow(clippy::too_many_arguments)]
pub fn service_world(
    world: &mut WorldObjects,
    content: &ObjectContent,
    trucks: &mut [Truck; TRUCKS],
    animation: u8,
    active_humans: u8,
    random: &mut RandomState,
    projection: Projection,
    tables: &ObjectTables,
) -> Result<(Vec<ObjectPhase>, Vec<u16>)> {
    let mut phases = vec![ObjectPhase::Composite];
    if content.composites.is_some()
        && let Some(composites) = &mut world.composites
    {
        service_composites(composites, trucks, animation, projection, tables)?;
    }
    phases.push(ObjectPhase::Pickup);
    let mut sounds = service_pickup(
        &mut world.pickup,
        &content.pickups,
        trucks,
        &mut world.pending_awards,
        animation,
        active_humans,
        random,
        projection,
        tables,
    )?;
    phases.push(ObjectPhase::Dynamic);
    sounds.extend(service_dynamic(
        &mut world.dynamic,
        trucks,
        animation,
        active_humans,
        tables,
    )?);
    phases.push(ObjectPhase::Placements);
    service_placements(&mut world.placements, trucks, animation, projection, tables)?;
    Ok((phases, sounds))
}
// Explicit disjoint owner borrows keep mutation authority visible at this boundary.
#[allow(clippy::too_many_arguments)]
pub fn initialize_world(
    world: &mut WorldObjects,
    content: &ObjectContent,
    trucks: &mut [Truck; TRUCKS],
    animation: u8,
    active_humans: u8,
    random: &mut RandomState,
    projection: Projection,
    tables: &ObjectTables,
) -> Result<Vec<u16>> {
    validate_content(content)?;
    apply_placements(&mut world.placements, &content.placements)?;
    track_three_sprites(
        content.track_three_sprites,
        tables,
        &mut world.special_staging,
        &mut world.special_upper,
    );
    if let Some(records) = content.composites {
        let mut composites = Composites {
            records: records.map(|placement| PlacedObject {
                placement,
                animation: 0,
                sprite: ObjectSprite::default(),
            }),
            count: COMPOSITE_RECORDS as u16,
            sprites: world
                .composites
                .as_ref()
                .map(|old| old.sprites.clone())
                .unwrap_or_default(),
        };
        service_composites(&mut composites, trucks, animation, projection, tables)?;
        world.composites = Some(composites);
    }
    initialize_dynamic(&mut world.dynamic, &content.dynamic, random, projection);
    let (_, sounds) = service_world(
        world,
        content,
        trucks,
        animation,
        active_humans,
        random,
        projection,
        tables,
    )?;
    for sprite in &mut world.pickup.sprites {
        sprite.coordinates = HIDDEN_COORDINATES;
    }
    world.pickup.phase = PickupPhase::Absent;
    world.pickup.counts.fill(0);
    world.pending_awards.fill(0);
    Ok(sounds)
}

const RECORD_BYTES: usize = 6;
fn read_word(bytes: &[u8], at: usize) -> Result<u16> {
    let pair = bytes.get(at..at + 2).context("truncated object word")?;
    Ok(u16::from_le_bytes([pair[0], pair[1]]))
}
fn read_placement(bytes: &[u8], at: usize) -> Result<Placement> {
    Ok(Placement {
        world: [read_word(bytes, at)?, read_word(bytes, at + 2)?],
        height: read_word(bytes, at + 4)?,
    })
}
fn screen_coordinates(placement: Placement, projection: Projection) -> u16 {
    const X_OFFSET: u16 = 0x40;
    const Y_OFFSET: u8 = 0x28;
    let position = super::terrain::project(projection, placement.world, placement.height as u8);
    u16::from_le_bytes([
        position.x.wrapping_sub(X_OFFSET) as u8,
        position.y.wrapping_add(Y_OFFSET),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    const PICKUP_SPAWNS: std::ops::Range<usize> = 0x46f3..0x4873;
    const OBJECT_POINTERS: usize = 0x4984;
    const PLACEMENT_POINTERS: usize = 0x5299;
    use super::super::tracks::{ORDER_SLOTS, ORIGINAL_TRACK_COUNT};
    const CENTER: [u16; 2] = [0x1000, 0x2000];
    const ALL_HUMANS: u8 = 15;
    fn rom() -> Vec<u8> {
        std::fs::read("../input/game.rom").unwrap()
    }
    fn word(bytes: &[u8], offset: usize) -> u16 {
        u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap())
    }
    // Fixture construction reads fixed ROM table fields only; no behavior oracle.
    fn tables() -> ObjectTables {
        let b = rom();
        ObjectTables {
            pickups: PickupAssets::decode(
                [7, 3],
                std::array::from_fn(|i| word(&b, 0x48f1 + i * 2)),
                std::array::from_fn(|i| word(&b, 0x48df + i * 2)),
            ),
            dynamic_vertical: [0x1a0, 0x1a0, 0x30],
            dynamic_y_deltas: std::array::from_fn(|i| b[0x4dc6 + i] as i8),
            dynamic_tiles: std::array::from_fn(|i| word(&b, 0x4df2 + i * 2)),
            placement_tiles: std::array::from_fn(|i| word(&b, 0x5229 + i * 2)),
            composite_tiles: std::array::from_fn(|i| {
                std::array::from_fn(|j| word(&b, 0x502e + i * 6 + j * 2))
            }),
            track_three_coordinates: [[0xa4, 0xa0], [0x2c, 0x98], [0xb8, 0x48]],
        }
    }
    fn projection() -> Projection {
        Projection {
            offset: [0, 0],
            scale: [0, 0],
        }
    }
    fn placement() -> Placement {
        Placement {
            world: CENTER,
            height: 0,
        }
    }
    fn trucks() -> [Truck; TRUCKS] {
        std::array::from_fn(|_| Truck {
            world: CENTER,
            speed: 0x30,
            ..Truck::default()
        })
    }
    fn pickup(kind: u16) -> Pickup {
        Pickup {
            phase: PickupPhase::from_encoded(kind),
            placement: placement(),
            ..Pickup::default()
        }
    }
    fn dynamic(kind: u16) -> DynamicObject {
        DynamicObject {
            stage: HazardStage::from_encoded(kind),
            placement: placement(),
            reactions: ReactionBudget::from_encoded(2),
            ..DynamicObject::default()
        }
    }
    fn no_dynamic() -> [DynamicObject; DYNAMIC_SLOTS] {
        std::array::from_fn(|_| DynamicObject::default())
    }
    fn content() -> ObjectContent {
        ObjectContent {
            placements: PlacementList {
                declared_count: 1,
                prefix: vec![placement()],
            },
            pickups: [placement(); PICKUP_CHOICES],
            dynamic: [Placement::default(); DYNAMIC_SLOTS],
            composites: None,
            track_three_sprites: false,
        }
    }
    fn world() -> WorldObjects {
        WorldObjects {
            pickup: Pickup::default(),
            pending_awards: [0; TRUCKS],
            dynamic: no_dynamic(),
            placements: vec![PlacedObject::default()],
            composites: None,
            contacts: [ContactAnimation::default(); TRUCKS],
            special_staging: [ObjectSprite::default(); 3],
            special_upper: [ObjectSprite::default(); 3],
        }
    }
    #[test]
    fn original_tables_decode_every_consumer_extent_and_signed_reaction_delta() {
        let actual = load_object_tables(&rom()).unwrap();
        assert_eq!(actual, tables());
        assert_eq!(actual.dynamic_y_deltas[0], 4);
        assert_eq!(actual.dynamic_y_deltas[43], -4);
        assert_eq!(actual.dynamic_tiles[34], 0x256c);
        assert_eq!(actual.placement_tiles[55], 0x3242);
    }
    #[test]
    fn all_sixteen_track_lists_and_sixtyfour_order_groups_have_distinct_consumer_bounds() {
        let b = rom();
        let counts = [4, 3, 5, 2, 4, 3, 2, 4, 5, 2, 3, 4, 3, 4, 4, 4];
        for (track, count) in counts.into_iter().enumerate() {
            let actual = load_original_content(&b, track as u8, 0).unwrap();
            assert_eq!(actual.placements.declared_count, count);
            let start = usize::from(word(&b, PLACEMENT_POINTERS + track * 2)) - 0x8000;
            for (i, p) in actual.placements.prefix.iter().enumerate() {
                let at = start + 2 + i * 6;
                assert_eq!(
                    *p,
                    Placement {
                        world: [word(&b, at), word(&b, at + 2)],
                        height: word(&b, at + 4)
                    }
                );
            }
            assert_eq!(actual.placements.prefix.len(), usize::from(count));
            for (choice, p) in actual.pickups.iter().enumerate() {
                let at = PICKUP_SPAWNS.start + track * 24 + choice * 6;
                assert_eq!(
                    *p,
                    Placement {
                        world: [word(&b, at), word(&b, at + 2)],
                        height: word(&b, at + 4)
                    }
                );
            }
        }
        for slot in 0..ORDER_SLOTS {
            let actual = load_original_content(&b, 0, slot as u8).unwrap();
            let start = usize::from(word(&b, OBJECT_POINTERS + slot * 2)) - 0x8000;
            // Source first record fills slot5, not slot0.
            for (i, p) in actual.dynamic.iter().rev().enumerate() {
                let at = start + i * 6;
                assert_eq!(
                    *p,
                    Placement {
                        world: [word(&b, at), word(&b, at + 2)],
                        height: word(&b, at + 4)
                    }
                );
            }
        }
        assert_ne!(
            load_original_content(&b, 0, 0).unwrap().dynamic,
            load_original_content(&b, 0, 11).unwrap().dynamic
        );
        assert!(load_original_content(&b, ORIGINAL_TRACK_COUNT as u8, 0).is_err());
        assert!(load_original_content(&b, 0, ORDER_SLOTS as u8).is_err());
    }
    #[test]
    fn loaders_reject_truncated_tables_pointers_and_records_without_fabricating_zeros() {
        let b = rom();
        assert!(load_object_tables(&b[..PICKUP_SPAWNS.end]).is_err());
        let mut bad = b.clone();
        bad[PLACEMENT_POINTERS..PLACEMENT_POINTERS + 2].copy_from_slice(&0x7fffu16.to_le_bytes());
        assert!(load_original_content(&bad, 0, 0).is_err());
        assert!(decode_placements(&[2, 0, 1, 0]).is_err());
        assert!(decode_placements(&[0, 0]).is_err());
    }
    #[test]
    fn negative_x_stops_record_writes_but_keeps_declared_count_and_retained_tail() {
        let bytes = [3, 0, 1, 0, 2, 0, 3, 0, 0, 128];
        let decoded = decode_placements(&bytes).unwrap();
        assert_eq!(
            decoded,
            PlacementList {
                declared_count: 3,
                prefix: vec![Placement {
                    world: [1, 2],
                    height: 3
                }]
            }
        );
        let old = PlacedObject {
            placement: Placement {
                world: [9, 8],
                height: 7,
            },
            animation: 55,
            sprite: ObjectSprite::default(),
        };
        let mut state = vec![old.clone(); 3];
        apply_placements(&mut state, &decoded).unwrap();
        assert_eq!(state[0].placement, decoded.prefix[0]);
        assert_eq!(state[0].animation, 0);
        assert_eq!(state[1], old);
        assert_eq!(state[2], old);
    }
    #[test]
    fn named_added_object_content_is_not_limited_by_original_track_count_and_validates_before_apply()
     {
        let mut added = content();
        added.placements.prefix.push(Placement {
            world: [0x1234, 0x5678],
            height: 0xabcd,
        });
        added.placements.declared_count = 2;
        validate_content(&added).unwrap();
        let mut objects = vec![];
        apply_placements(&mut objects, &added.placements).unwrap();
        assert_eq!(objects.len(), 2);
        let saved = objects.clone();
        let bad = PlacementList {
            declared_count: 1,
            prefix: vec![placement(); 2],
        };
        assert!(apply_placements(&mut objects, &bad).is_err());
        assert_eq!(objects, saved);
        added.placements.declared_count = 0;
        assert!(validate_content(&added).is_err());
    }
    #[test]
    fn pickup_wrapped_rectangle_excludes_lower_and_includes_upper_world_edges() {
        let p = placement();
        assert!(pickup_near(p, CENTER));
        assert!(!pickup_near(p, [CENTER[0] - 0x180, CENTER[1]]));
        assert!(pickup_near(p, [CENTER[0] + 0x180, CENTER[1]]));
        assert!(!pickup_near(p, [CENTER[0] + 0x181, CENTER[1]]));
        assert!(!pickup_near(p, [CENTER[0], CENTER[1] - 0x1c0]));
        assert!(pickup_near(p, [CENTER[0], CENTER[1] + 0x340]));
        assert!(!pickup_near(p, [CENTER[0], CENTER[1] + 0x341]));
        assert!(pickup_near(
            Placement {
                world: [0, 0],
                height: 0
            },
            [0xffff, 0xffff]
        ));
    }
    #[test]
    fn spawn_requires_zero_gate_uses_four_choices_and_frame_parity_not_choice_type() {
        let choices = std::array::from_fn(|i| Placement {
            world: [0x1000 + i as u16, 0x2000],
            height: 0,
        });
        for gate in [1, 255] {
            let mut p = pickup(0);
            let old = p.clone();
            try_spawn_pickup(&mut p, &choices, gate, 3, 0, projection(), &tables());
            assert_eq!(p, old);
        }
        for choice in 0..4 {
            for frame in [0, 1] {
                let mut p = pickup(0);
                try_spawn_pickup(&mut p, &choices, 0, choice, frame, projection(), &tables());
                assert_eq!(p.placement, choices[choice as usize]);
                assert_eq!(p.phase.encoded(), u16::from(frame) + 1);
            }
        }
    }
    #[test]
    fn capped_type_suppresses_activation_after_spawn_coordinates_are_already_loaded() {
        let mut p = pickup(0);
        p.counts = [4, 0];
        p.lifetime = 123;
        let choices = [Placement {
            world: [9, 10],
            height: 11,
        }; 4];
        try_spawn_pickup(&mut p, &choices, 0, 0, 0, projection(), &tables());
        assert_eq!(p.phase.encoded(), 0);
        assert_eq!(p.placement, choices[0]);
        assert_eq!(p.lifetime, 123);
        try_spawn_pickup(&mut p, &choices, 0, 0, 1, projection(), &tables());
        assert_eq!(p.phase.encoded(), 2);
    }
    #[test]
    fn pickup_award_uses_old_count_then_saturates_count_and_byte_awards_wrap() {
        for old in 0..=4 {
            let mut p = pickup(1);
            p.counts[0] = old;
            let mut ts = trucks();
            let mut awards = [0; 4];
            awards[0] = 255;
            let sounds = collect_pickup(
                &mut p,
                &mut ts,
                &mut awards,
                0,
                ALL_HUMANS,
                projection(),
                &tables(),
            )
            .unwrap();
            assert_eq!(p.phase.encoded(), 7 + old);
            assert_eq!(p.counts[0], (old + 1).min(4));
            assert_eq!(awards[0], (255u16 + 1 + old) as u8);
            assert_eq!(sounds, [PICKUP_MONEY_SOUND]);
            assert_eq!(p.lifetime, 79);
        }
    }
    #[test]
    fn iteration_three_pickup_override_can_change_reward_category_and_retained_phase() {
        // The source chooses the reward from the resulting state, not from the
        // spawned type. Overrides may cross the money/nitro threshold or return
        // to a collectable state; preserve that behavior at the asset boundary.
        for (base, old_count, expected_state, nitro, money) in [
            (6, 0, 6, 4, 0),
            (6, 1, 7, 0, 1),
            (2, 0, 2, 0, 0),
            (7, 4, 11, 0, 5),
            (3, 4, 7, 0, 1),
        ] {
            let mut data = tables();
            let b = rom();
            data.pickups = PickupAssets::decode(
                [base, 3],
                std::array::from_fn(|i| word(&b, 0x48f1 + i * 2)),
                std::array::from_fn(|i| word(&b, 0x48df + i * 2)),
            );
            let mut p = pickup(1);
            p.counts[0] = old_count;
            let mut ts = trucks();
            let mut awards = [0; TRUCKS];
            collect_pickup(&mut p, &mut ts, &mut awards, 0, 1, projection(), &data).unwrap();
            assert_eq!(p.phase.encoded(), expected_state);
            assert_eq!(ts[0].nitro.encoded(), nitro);
            assert_eq!(awards[0], money);
            assert_eq!(p.lifetime, 79);
        }
    }
    #[test]
    fn conditional_already_capped_type_one_uses_state_eleven_and_adjacent_rom_tileword() {
        // Natural spawn rejects count4; direct award at count4 reaches11. The
        // primary tile read then aliases sequence-initializer bytes 08C2, extra
        // tile uses the otherwise-unowned preceding CA24 word at48EF.
        let mut p = pickup(1);
        p.counts[0] = 4;
        let mut ts = trucks();
        let mut a = [0; 4];
        collect_pickup(
            &mut p,
            &mut ts,
            &mut a,
            1,
            ALL_HUMANS,
            projection(),
            &tables(),
        )
        .unwrap();
        assert_eq!(p.phase.encoded(), 11);
        assert_eq!(p.sprites[0].tile_attributes, 0xc208);
        assert_eq!(p.sprites[1].tile_attributes, 0x24ca);
    }
    #[test]
    fn nitro_pickup_bcd_overflow_retains_old_stock_and_rotates_selected_truck() {
        for (old_count, stock, expected) in [(0, 0x09, 0x10), (3, 0x95, 0x99), (3, 0x96, 0x96)] {
            let mut p = pickup(2);
            p.counts[1] = old_count;
            let mut ts = trucks();
            ts[3].nitro = crate::trucks::nitro::NitroStock::from_encoded(stock);
            let mut a = [0; 4];
            let sound = collect_pickup(
                &mut p,
                &mut ts,
                &mut a,
                3,
                ALL_HUMANS,
                projection(),
                &tables(),
            )
            .unwrap();
            assert_eq!(ts[3].nitro.encoded(), expected);
            assert_eq!(ts[0].nitro.encoded(), 0);
            assert_eq!(p.phase.encoded(), 3 + old_count);
            assert_eq!(sound, [PICKUP_NITRO_SOUND]);
        }
    }
    #[test]
    fn pickup_sound_uses_any_human_mask_and_not_collector_human_status() {
        for mask in [0, 1] {
            let mut p = pickup(2);
            let mut ts = trucks();
            let mut a = [0; 4];
            let sounds =
                collect_pickup(&mut p, &mut ts, &mut a, 3, mask, projection(), &tables()).unwrap();
            assert_eq!(
                sounds,
                if mask == 0 {
                    vec![]
                } else {
                    vec![PICKUP_NITRO_SOUND]
                }
            );
            assert_eq!(ts[3].nitro.encoded(), 1);
        }
    }
    #[test]
    fn pickup_expiry_moves_y_each_four_frames_and_zero_timer_wraps_instead_of_expiring() {
        let mut p = pickup(3);
        p.lifetime = 2;
        p.sprites = [ObjectSprite {
            coordinates: 0x0012,
            tile_attributes: 9,
        }; 2];
        tick_pickup_award(&mut p, 0);
        assert_eq!(p.sprites[0].coordinates, 0xff12);
        assert_eq!(p.lifetime, 1);
        tick_pickup_award(&mut p, 1);
        assert_eq!(p.phase.encoded(), 0);
        assert_eq!(p.sprites[0].coordinates, HIDDEN_COORDINATES);
        assert_eq!(p.sprites[0].tile_attributes, 9);
        p.phase = PickupPhase::from_encoded(3);
        p.lifetime = 0;
        tick_pickup_award(&mut p, 1);
        assert_eq!(p.lifetime, 0xffff);
        assert_eq!(p.phase.encoded(), 3);
    }
    #[test]
    fn pickup_packed_xy_pair_centering_propagates_x_borrow_into_y() {
        // Y2000 contributes68 to projectedX even when both variable scales are zero;
        // offsetFFFF yields67, hence displayed X3/Y232. Packed subtraction borrows Y.
        let mut p = pickup(3);
        let proj = Projection {
            offset: [0xffff, 0],
            scale: [0, 0],
        };
        project_pickup(&mut p, proj, &tables()).unwrap();
        assert_eq!(p.sprites[0].coordinates, 0xe7fb);
        assert_eq!(p.sprites[1].coordinates, 0xe80b);
        assert_eq!(p.sprites[0].tile_attributes, 0x36e8);
        assert_eq!(p.sprites[1].tile_attributes, 0x36ea);
    }
    #[test]
    fn inactive_pickup_service_consumes_one_rng_at_failed_gate_and_does_not_tick_lifetime() {
        let mut p = pickup(0);
        p.lifetime = 17;
        let mut ts = trucks();
        let mut a = [0; 4];
        let mut rng = RandomState([0x12, 0x34, 0x56, 0x78, 0x9b]);
        service_pickup(
            &mut p,
            &[placement(); 4],
            &mut ts,
            &mut a,
            0,
            1,
            &mut rng,
            projection(),
            &tables(),
        )
        .unwrap();
        assert_eq!(rng.0, [0xdf, 0x12, 0x34, 0x56, 0x78]);
        assert_eq!(p.phase.encoded(), 0);
        assert_eq!(p.lifetime, 17);
    }
    #[test]
    fn zero_spawn_gate_consumes_second_rng_and_initial_spawn_does_not_install_award_lifetime() {
        let mut p = pickup(0);
        p.lifetime = 17;
        let mut ts = trucks();
        let mut a = [0; 4];
        // FF nibble combine XOR FF feedback =>0; next F0 XOR FF =>0F.
        let mut rng = RandomState([1, 255, 255, 255, 255]);
        service_pickup(
            &mut p,
            &[placement(); 4],
            &mut ts,
            &mut a,
            1,
            1,
            &mut rng,
            projection(),
            &tables(),
        )
        .unwrap();
        assert_eq!(rng.0, [0x0f, 0, 1, 255, 255]);
        assert_eq!(p.phase.encoded(), 2);
        assert_eq!(p.lifetime, 17);
        assert_eq!(a, [0; 4]);
    }
    #[test]
    fn active_pickup_tests_only_rotating_truck_and_does_not_draw_random_numbers() {
        let mut p = pickup(1);
        let mut ts = trucks();
        ts[1].world = [0, 0];
        let mut a = [0; 4];
        let mut rng = RandomState([1, 2, 3, 4, 5]);
        let old = rng;
        service_pickup(
            &mut p,
            &[placement(); 4],
            &mut ts,
            &mut a,
            1,
            1,
            &mut rng,
            projection(),
            &tables(),
        )
        .unwrap();
        assert_eq!(p.phase.encoded(), 1);
        assert_eq!(rng, old);
        assert_eq!(a, [0; 4]);
    }
    #[test]
    fn dynamic_initialization_is_descending_consumes_rng_only_nonzero_x_and_preserves_inactive_fields()
     {
        let mut ds = no_dynamic();
        for d in &mut ds {
            d.response = ResponseClock::from_encoded(99);
            d.sprite.coordinates = 0x8877;
        }
        let mut records = [Placement::default(); 6];
        records[5] = placement();
        records[2] = Placement {
            world: [9, 10],
            height: 11,
        };
        let mut rng = RandomState([0x12, 0x34, 0x56, 0x78, 0x9b]);
        initialize_dynamic(&mut ds, &records, &mut rng, projection());
        assert_eq!(ds[5].reactions.encoded(), 7);
        assert_eq!(ds[2].reactions.encoded(), 2);
        assert_eq!(rng.0, [0x6a, 0xdf, 0x12, 0x34, 0x56]);
        assert_eq!(ds[5].stage.encoded(), 1);
        assert_eq!(ds[5].response.encoded(), 0);
        assert_eq!(ds[5].sprite.tile_attributes, DYNAMIC_IDLE_TILE);
        assert_eq!(ds[0].stage.encoded(), 0);
        assert_eq!(ds[0].response.encoded(), 99);
        assert_eq!(ds[0].sprite.coordinates, 0x8877);
    }
    #[test]
    fn dynamic_rectangles_have_asymmetric_edges_and_type_three_height_adjustment() {
        let t = tables();
        let d = dynamic(1);
        assert!(!dynamic_near(&d, [CENTER[0] - 0x200, CENTER[1]], &t).unwrap());
        assert!(dynamic_near(&d, [CENTER[0] + 0x100, CENTER[1]], &t).unwrap());
        assert!(!dynamic_near(&d, [CENTER[0], CENTER[1] - 0xa0], &t).unwrap());
        assert!(dynamic_near(&d, [CENTER[0], CENTER[1] + 0x260], &t).unwrap());
        let d3 = dynamic(3);
        assert!(!dynamic_near(&d3, [CENTER[0], CENTER[1] - 0x210], &t).unwrap());
        assert!(dynamic_near(&d3, [CENTER[0], CENTER[1] + 0xf0], &t).unwrap());
    }
    #[test]
    fn dynamic_contact_is_only_tested_at_zero_or_eighteen_animation() {
        for animation in [1, 17, 19, 29, 30, 31, 34] {
            let mut ds = no_dynamic();
            ds[5] = dynamic(2);
            ds[5].animation = animation;
            let mut ts = trucks();
            service_dynamic(&mut ds, &mut ts, 1, ALL_HUMANS, &tables()).unwrap();
            assert_eq!(ds[5].stage.encoded(), 2);
            assert_eq!(ts[1].speed, 0x30);
        }
        for animation in [0, 18] {
            let mut ds = no_dynamic();
            ds[5] = dynamic(2);
            ds[5].animation = animation;
            let mut ts = trucks();
            service_dynamic(&mut ds, &mut ts, 1, ALL_HUMANS, &tables()).unwrap();
            assert_eq!(ds[5].stage.encoded(), 3);
            assert_eq!(ds[5].animation, 17);
            assert_eq!(ts[1].speed, 0);
        }
    }
    #[test]
    fn type_one_contact_arms_then_decrements_timer_moves_y_and_zeroes_full_speed_word() {
        let mut ds = no_dynamic();
        ds[5] = dynamic(1);
        ds[5].sprite.coordinates = 0x4010;
        let mut ts = trucks();
        ts[1].speed = 0xab30;
        let sounds = service_dynamic(&mut ds, &mut ts, 1, ALL_HUMANS, &tables()).unwrap();
        assert_eq!(ts[1].speed, 0);
        assert_eq!(ds[5].response.encoded(), 43);
        assert_eq!(ds[5].animation, 34);
        assert_eq!(ds[5].sprite.coordinates, 0x3c10);
        assert_eq!(ds[5].sprite.tile_attributes, 0x356c);
        assert_eq!(sounds, [DYNAMIC_CONTACT_SOUND]);
    }
    #[test]
    fn dynamic_tile_uses_predecrement_animation_and_timer_palette_threshold() {
        for (timer, tile) in [(3, 0x2522), (4, 0x3522)] {
            let mut ds = no_dynamic();
            ds[0] = dynamic(2);
            ds[0].response = ResponseClock::from_encoded(timer);
            ds[0].animation = 17;
            let mut ts = trucks();
            service_dynamic(&mut ds, &mut ts, 0, 0, &tables()).unwrap();
            assert_eq!(ds[0].sprite.tile_attributes, tile);
            assert_eq!(ds[0].animation, 16);
            assert_eq!(ds[0].response.encoded(), timer);
        }
    }
    #[test]
    fn idle_type_one_underflows_timer_and_consumes_uses_even_without_contact() {
        let mut ds = no_dynamic();
        ds[0] = dynamic(1);
        ds[0].reactions = ReactionBudget::from_encoded(0);
        let mut ts = trucks();
        for truck in &mut ts {
            truck.world = [0, 0];
        }
        // At animation0, a miss jumps next and does NOT reach lifetime. A prior
        // nonzero animation enters lifetime even when no contact test occurs.
        ds[0].animation = 1;
        service_dynamic(&mut ds, &mut ts, 1, 0, &tables()).unwrap();
        assert_eq!(ds[0].stage.encoded(), 2);
        assert_eq!(ds[0].reactions.encoded(), 0xffff);
        assert_eq!(ds[0].response.encoded(), 0);
        assert_eq!(ds[0].animation, 0);
        assert_eq!(ds[0].sprite.tile_attributes, DYNAMIC_EXHAUSTED_TILE);
    }
    #[test]
    fn type_one_underflow_retains_use_count_when_animation_is_twenty_nine_after_tick() {
        let mut ds = no_dynamic();
        ds[0] = dynamic(1);
        ds[0].animation = 30;
        ds[0].response = ResponseClock::from_encoded(0);
        ds[0].reactions = ReactionBudget::from_encoded(2);
        let mut ts = trucks();
        service_dynamic(&mut ds, &mut ts, 0, 0, &tables()).unwrap();
        assert_eq!(ds[0].animation, 29);
        assert_eq!(ds[0].response.encoded(), 0xffff);
        assert_eq!(ds[0].reactions.encoded(), 2);
    }
    #[test]
    fn type_three_response_requires_full_word_motion_and_does_not_stop_truck() {
        for speed in [0, 0x8000, 0x80] {
            let mut ds = no_dynamic();
            ds[0] = dynamic(3);
            let mut ts = trucks();
            ts[1].speed = speed;
            service_dynamic(&mut ds, &mut ts, 1, 0, &tables()).unwrap();
            assert_eq!(ds[0].animation, if speed == 0 { 0 } else { 29 });
            assert_eq!(ts[1].speed, speed);
        }
    }
    #[test]
    fn descending_dynamic_scan_later_type_three_sees_speed_stopped_by_higher_slot() {
        let mut ds = no_dynamic();
        ds[5] = dynamic(2);
        ds[0] = dynamic(3);
        let mut ts = trucks();
        service_dynamic(&mut ds, &mut ts, 1, 0, &tables()).unwrap();
        assert_eq!(ds[5].stage.encoded(), 3);
        assert_eq!(ts[1].speed, 0);
        assert_eq!(ds[0].animation, 0);
    }
    #[test]
    fn dynamic_missed_idle_contact_skips_shadow_timer_and_animation_updates() {
        let mut ds = no_dynamic();
        ds[0] = dynamic(1);
        ds[0].sprite.tile_attributes = 0xbeef;
        let mut ts = trucks();
        ts[0].world = [0, 0];
        service_dynamic(&mut ds, &mut ts, 0, 0, &tables()).unwrap();
        assert_eq!(ds[0].response.encoded(), 0);
        assert_eq!(ds[0].reactions.encoded(), 2);
        assert_eq!(ds[0].sprite.tile_attributes, 0xbeef);
    }
    #[test]
    fn special_surface_requires_marker_and_all_four_high_history_bits() {
        let mut t = trucks()[0].clone();
        for (marker, history) in [(0, 0xf0), (1, 0x70), (1, 0x0f)] {
            t.contact_marker = marker;
            let mut w = WorkingMotion {
                world: CENTER,
                speed: 0xab40,
                heading: 0,
            };
            let mut c = ContactAnimation::default();
            assert!(special_surface(&t, &mut w, history, &mut c, 1).is_empty());
            assert_eq!(w.speed, 0xab40);
            assert_eq!(c.cursor, 0);
        }
    }
    #[test]
    fn new_surface_contact_halves_only_nonnegative_low_speed_and_sets_cursor_ten_with_sound() {
        let mut t = trucks()[0].clone();
        t.contact_marker = 1;
        t.screen_x = 0x0123;
        t.screen_y = 0x45;
        for (speed, expected) in [(0xab41, 0xab20), (0xab80, 0xab80)] {
            let mut w = WorkingMotion {
                world: CENTER,
                speed,
                heading: 0,
            };
            let mut c = ContactAnimation::default();
            assert_eq!(
                special_surface(&t, &mut w, 0xff, &mut c, 1),
                [SURFACE_CONTACT_SOUND]
            );
            assert_eq!(w.speed, expected);
            assert_eq!(c.cursor, 10);
            assert_eq!(c.sprite.coordinates, 0x4523);
        }
    }
    #[test]
    fn surface_resume_cursors_eighteen_or_twenty_three_set_nineteen_without_sound() {
        let mut t = trucks()[0].clone();
        t.contact_marker = 1;
        for cursor in [18, 23] {
            let mut w = WorkingMotion {
                world: CENTER,
                speed: 0x40,
                heading: 0,
            };
            let mut c = ContactAnimation {
                cursor,
                ..ContactAnimation::default()
            };
            assert!(special_surface(&t, &mut w, 0xf0, &mut c, 1).is_empty());
            assert_eq!(c.cursor, 19);
            assert_eq!(w.speed, 0x20);
        }
        let mut w = WorkingMotion {
            world: CENTER,
            speed: 0x40,
            heading: 0,
        };
        let mut c = ContactAnimation {
            cursor: 19,
            ..ContactAnimation::default()
        };
        special_surface(&t, &mut w, 0xf0, &mut c, 1);
        assert_eq!(w.speed, 0x40);
        assert_eq!(c.cursor, 19);
    }
    #[test]
    fn special_surface_attract_suppresses_sound_only_not_slowdown_or_cursor() {
        let mut t = trucks()[0].clone();
        t.contact_marker = 1;
        let mut w = WorkingMotion {
            world: CENTER,
            speed: 0x40,
            heading: 0,
        };
        let mut c = ContactAnimation::default();
        assert!(special_surface(&t, &mut w, 0xf0, &mut c, 0).is_empty());
        assert_eq!(w.speed, 0x20);
        assert_eq!(c.cursor, 10);
    }
    #[test]
    fn track_three_initialization_expands_parallel_coordinates_and_other_tracks_preserve_staging() {
        let sentinel = ObjectSprite {
            coordinates: 0xbeef,
            tile_attributes: 0xcafe,
        };
        let mut stage = [sentinel; 3];
        let mut upper = [sentinel; 3];
        track_three_sprites(false, &tables(), &mut stage, &mut upper);
        assert_eq!(stage, [sentinel; 3]);
        track_three_sprites(true, &tables(), &mut stage, &mut upper);
        assert_eq!(
            stage,
            [
                ObjectSprite {
                    coordinates: 0xa0a4,
                    tile_attributes: 0x228c
                },
                ObjectSprite {
                    coordinates: 0x982c,
                    tile_attributes: 0x228c
                },
                ObjectSprite {
                    coordinates: 0x48b8,
                    tile_attributes: 0x228c
                }
            ]
        );
        assert_eq!(
            upper[0],
            ObjectSprite {
                coordinates: 0x90a4,
                tile_attributes: 0x208a
            }
        );
        assert_eq!(upper[2].coordinates, 0x38b8);
    }
    #[test]
    fn placement_motion_gate_uses_low_seven_bits_and_asymmetric_rectangle_edges() {
        let p = placement();
        let mut t = trucks()[0].clone();
        for speed in [0, 0x80, 0x100] {
            t.speed = speed;
            assert!(!placement_near(p, &t));
        }
        t.speed = 0x81;
        assert!(placement_near(p, &t));
        t.world = [CENTER[0] - 0x140, CENTER[1]];
        assert!(!placement_near(p, &t));
        t.world = [CENTER[0] + 0x140, CENTER[1]];
        assert!(placement_near(p, &t));
        t.world = [CENTER[0], CENTER[1] - 0x140];
        assert!(!placement_near(p, &t));
        t.world = [CENTER[0], CENTER[1] + 0x80];
        assert!(placement_near(p, &t));
    }
    #[test]
    fn placement_draws_old_timer_then_decrements_and_can_retrigger_same_frame() {
        let mut objects = [PlacedObject {
            placement: placement(),
            animation: 1,
            sprite: ObjectSprite::default(),
        }];
        let ts = trucks();
        service_placements(&mut objects, &ts, 0, projection(), &tables()).unwrap();
        assert_eq!(objects[0].sprite.tile_attributes, 0x7260);
        assert_eq!(objects[0].animation, PLACEMENT_REACTION);
        service_placements(&mut objects, &ts, 1, projection(), &tables()).unwrap();
        assert_eq!(objects[0].sprite.tile_attributes, 0x3242);
        assert_eq!(objects[0].animation, PLACEMENT_REACTION);
    }
    #[test]
    fn composite_motion_rectangle_uses_shifted_center_and_distinct_y_extent() {
        let p = placement();
        let mut t = trucks()[0].clone();
        assert!(!composite_near(p, &t));
        t.world = [CENTER[0], CENTER[1] - 0x100];
        assert!(composite_near(p, &t));
        t.world = [CENTER[0] - 0x100, CENTER[1] - 0x100];
        assert!(!composite_near(p, &t));
        t.world = [CENTER[0] + 0x500, CENTER[1] - 0x100];
        assert!(composite_near(p, &t));
        t.world = [CENTER[0], CENTER[1] - 0x240];
        assert!(!composite_near(p, &t));
        t.world = [CENTER[0], CENTER[1] - 0x80];
        assert!(composite_near(p, &t));
        t.speed = 0x80;
        assert!(!composite_near(p, &t));
    }
    fn composites() -> Composites {
        Composites {
            records: std::array::from_fn(|_| PlacedObject {
                placement: placement(),
                animation: 0,
                sprite: ObjectSprite::default(),
            }),
            count: 4,
            sprites: vec![],
        }
    }
    #[test]
    fn inclusive_composite_loop_derives_extra_record_from_adjacent_first_fields() {
        let mut c = composites();
        c.records[0].placement = Placement {
            world: [0x1240, 0x0f40],
            height: 0x50,
        };
        c.records[0].animation = 7;
        let extra = composite_tail(&c);
        assert_eq!(
            extra.placement,
            Placement {
                world: [0x0f40, 0x50],
                height: 7
            }
        );
        assert_eq!(extra.animation, 4);
    }
    #[test]
    fn composite_count_four_visits_five_then_fourth_frame_decrements_count_alias_to_three() {
        let mut c = composites();
        let mut ts = trucks();
        for t in &mut ts {
            t.world = [0, 0];
            t.speed = 0;
        }
        service_composites(&mut c, &ts, 1, projection(), &tables()).unwrap();
        assert_eq!(c.sprites.len(), 5);
        assert_eq!(c.count, 4);
        service_composites(&mut c, &ts, 0, projection(), &tables()).unwrap();
        assert_eq!(c.sprites.len(), 5);
        assert_eq!(c.count, 3);
        service_composites(&mut c, &ts, 1, projection(), &tables()).unwrap();
        assert_eq!(c.sprites.len(), 4);
        assert_eq!(c.count, 3);
    }
    #[test]
    fn composite_triplets_draw_before_timer_tick_and_byte_x_positions_wrap() {
        let mut c = composites();
        c.count = 3;
        c.records[0].animation = 1;
        let mut ts = trucks();
        for t in &mut ts {
            t.speed = 0;
        }
        // projectedX304 -64=240; sprite Xs240,0,16 (byte additions).
        let proj = Projection {
            offset: [236, 0],
            scale: [0, 0],
        };
        service_composites(&mut c, &ts, 0, proj, &tables()).unwrap();
        assert_eq!(c.records[0].animation, 0);
        assert_eq!(
            c.sprites[0].map(|s| s.coordinates),
            [0xe8f0, 0xe800, 0xe810]
        );
        assert_eq!(
            c.sprites[0].map(|s| s.tile_attributes),
            [0x22ae, 0x24c0, 0x60ae]
        );
    }
    #[test]
    fn world_service_performs_composite_pickup_dynamic_then_placements_on_actual_state() {
        let mut w = world();
        w.pickup = pickup(1);
        w.dynamic[5] = dynamic(2);
        w.placements[0].placement = placement();
        let mut ts = trucks();
        let mut rng = RandomState([1, 2, 3, 4, 5]);
        let old = rng;
        let (phases, sounds) = service_world(
            &mut w,
            &content(),
            &mut ts,
            1,
            1,
            &mut rng,
            projection(),
            &tables(),
        )
        .unwrap();
        assert_eq!(
            phases,
            [
                ObjectPhase::Composite,
                ObjectPhase::Pickup,
                ObjectPhase::Dynamic,
                ObjectPhase::Placements
            ]
        );
        assert_eq!(w.pending_awards[1], 1);
        assert_eq!(w.pickup.phase.encoded(), 7);
        assert_eq!(ts[1].speed, 0);
        assert_eq!(w.dynamic[5].stage.encoded(), 3);
        assert_eq!(w.placements[0].animation, 0);
        assert_eq!(rng, old);
        assert_eq!(sounds, [PICKUP_MONEY_SOUND]);
    }
    #[test]
    fn world_initialization_services_old_pickup_before_resetting_counts_and_pending_awards() {
        let mut w = world();
        w.pickup = pickup(2);
        w.pending_awards = [99; 4];
        w.pickup.counts = [2, 1];
        let mut ts = trucks();
        let mut rng = RandomState([1, 2, 3, 4, 5]);
        initialize_world(
            &mut w,
            &content(),
            &mut ts,
            1,
            1,
            &mut rng,
            projection(),
            &tables(),
        )
        .unwrap();
        assert_eq!(ts[1].nitro.encoded(), 2);
        assert_eq!(w.pickup.phase.encoded(), 0);
        assert_eq!(w.pickup.counts, [0, 0]);
        assert_eq!(w.pending_awards, [0; 4]);
        assert_eq!(
            w.pickup.sprites.map(|s| s.coordinates),
            [HIDDEN_COORDINATES; 2]
        );
    }
}
