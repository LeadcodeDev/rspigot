use std::collections::BTreeMap;
use std::num::NonZeroU8;

use rand::Rng;
use rspigot_blocks::{
    BlockKind, Face, Gravity, SCAFFOLDING_MAX_DISTANCE, StateFlags, scaffolding_settle,
    speleothem_thickness,
};
use rspigot_domain::item::ItemStack;
use rspigot_domain::{BlockPos, BlockStateId, EntityId, ItemId, Rotation, Vec3};
use rustc_hash::FxHashSet;
use uuid::Uuid;

use crate::events::{ClientEvent, EntityKind};
use crate::loot::{self, LootContext};
use crate::ports::ClientLink;
use crate::server::{Audience, Server};

const FALLING_DELAY: u64 = 2;
const STALACTITE_DELAY: u64 = 2;
const STALAGMITE_DELAY: u64 = 1;
const SCAFFOLDING_DELAY: u64 = 1;
const ACCELERATION: f64 = 0.04;
const DRAG: f64 = 0.98;
const MAX_FALL_TICKS: u32 = 600;
const OUT_OF_WORLD_TICKS: u32 = 100;
const TRACKING_RANGE: f64 = 160.0;
const POSITION_SYNC_TICKS: u32 = 20;
const STALAGMITE_LANDING_BONUS: f64 = 2.5;
const MIN_STALACTITE_SIZE: i32 = 6;
const MAX_FALL_DAMAGE: i32 = 40;

#[derive(Debug, Default)]
pub struct ScheduledFalls {
    due: BTreeMap<u64, Vec<(BlockPos, BlockKind)>>,
    pending: FxHashSet<(BlockPos, BlockKind)>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FallHurt {
    pub per_distance: f32,
    pub max: i32,
}

impl FallHurt {
    pub const ANVIL: Self = Self {
        per_distance: 2.0,
        max: MAX_FALL_DAMAGE,
    };

    pub fn stalactite(size: i32) -> Self {
        Self {
            per_distance: size.max(MIN_STALACTITE_SIZE) as f32,
            max: MAX_FALL_DAMAGE,
        }
    }

    pub fn damage(self, fall_distance: f64) -> f32 {
        let blocks = fall_blocks(fall_distance);
        if blocks < 0 {
            return 0.0;
        }
        ((blocks as f32 * self.per_distance).floor() as i32).min(self.max) as f32
    }
}

pub fn anvil_degrade_chance(fall_distance: f64) -> f32 {
    if FallHurt::ANVIL.damage(fall_distance) > 0.0 {
        0.05 + fall_blocks(fall_distance) as f32 * 0.05
    } else {
        0.0
    }
}

fn fall_blocks(fall_distance: f64) -> i32 {
    (fall_distance - 1.0).ceil() as i32
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallingBlock {
    pub id: EntityId,
    pub uuid: Uuid,
    pub state: BlockStateId,
    pub position: Vec3,
    pub velocity: Vec3,
    pub time: u32,
    pub fall_distance: f64,
    pub hurt: Option<FallHurt>,
    pub cancel_drop: bool,
    synced_position: Vec3,
}

impl FallingBlock {
    fn spawn_event(&self) -> ClientEvent {
        ClientEvent::SpawnEntity {
            entity: self.id,
            uuid: self.uuid,
            kind: EntityKind::FallingBlock,
            position: self.position,
            rotation: Rotation::default(),
            head_yaw: 0.0,
            data: i32::from(self.state.0),
            velocity: self.velocity,
        }
    }
}

fn is_free(state: BlockStateId) -> bool {
    state.is_air()
        || state.flags().contains(StateFlags::REPLACEABLE)
        || matches!(
            state.kind(),
            Some(BlockKind::Water | BlockKind::Lava | BlockKind::Fire | BlockKind::SoulFire)
        )
}

fn sturdy(state: BlockStateId) -> bool {
    let flags = state.flags();
    flags.contains(StateFlags::SOLID_RENDER) || flags.contains(StateFlags::FULL_COLLISION_CUBE)
}

fn has_water(state: BlockStateId) -> bool {
    state.flags().contains(StateFlags::WATER)
}

fn water_source(state: BlockStateId) -> bool {
    has_water(state)
        && (state.kind() != Some(BlockKind::Water) || state.property("level") == Some("0"))
}

fn gravity(state: BlockStateId) -> Gravity {
    state.kind().map_or(Gravity::None, BlockKind::gravity)
}

fn left_behind(state: BlockStateId) -> BlockStateId {
    if state.property("waterlogged") == Some("true") {
        BlockKind::Water.default_state()
    } else {
        BlockStateId::AIR
    }
}

fn waterlogged(state: BlockStateId, water: bool) -> BlockStateId {
    state
        .with_property("waterlogged", if water { "true" } else { "false" })
        .unwrap_or(state)
}

fn step(pos: BlockPos, face: Face) -> BlockPos {
    let (dx, dy, dz) = face.offset();
    pos.offset(dx, dy, dz)
}

impl<L: ClientLink> Server<L> {
    fn block_or_air(&self, pos: BlockPos) -> BlockStateId {
        self.world.chunks.block(pos).unwrap_or(BlockStateId::AIR)
    }

    pub(crate) fn notify_neighbours(&mut self, pos: BlockPos) {
        self.on_place(pos);
        for face in Face::ALL {
            self.update_shape(step(pos, face), face.opposite());
        }
    }

    pub(crate) fn placement_state(
        &self,
        pos: BlockPos,
        state: BlockStateId,
        pitch: f32,
        sneaking: bool,
    ) -> Option<BlockStateId> {
        let Some(kind) = state.kind() else {
            return Some(state);
        };
        let replaced = self.block_or_air(pos);
        match kind.gravity() {
            Gravity::ConcretePowder => Some(match kind.concrete() {
                Some(concrete) if has_water(replaced) || self.touches_liquid(pos) => {
                    concrete.default_state()
                }
                _ => state,
            }),
            Gravity::Scaffolding => Some(waterlogged(
                self.settle_scaffolding(pos, state),
                water_source(replaced),
            )),
            Gravity::Speleothem => {
                let looking = if pitch < 0.0 { Face::Up } else { Face::Down };
                let tip = [looking.opposite(), looking]
                    .into_iter()
                    .find(|&tip| self.speleothem_survives(pos, kind, tip))?;
                let thickness = self.speleothem_thickness_at(pos, kind, tip, !sneaking);
                let direction = if tip == Face::Up { "up" } else { "down" };
                let placed = kind
                    .state_with(&[("vertical_direction", direction), ("thickness", thickness)])?;
                Some(waterlogged(placed, water_source(replaced)))
            }
            _ => Some(state),
        }
    }

    fn schedule(&mut self, pos: BlockPos, kind: BlockKind, delay: u64) {
        if self.falls.pending.insert((pos, kind)) {
            let due = self.current_tick() + delay;
            self.falls.due.entry(due).or_default().push((pos, kind));
        }
    }

    fn on_place(&mut self, pos: BlockPos) {
        let Some(kind) = self.block_or_air(pos).kind() else {
            return;
        };
        match kind.gravity() {
            Gravity::Falling | Gravity::ConcretePowder | Gravity::Anvil => {
                self.schedule(pos, kind, FALLING_DELAY)
            }
            Gravity::Scaffolding => self.schedule(pos, kind, SCAFFOLDING_DELAY),
            Gravity::Speleothem | Gravity::None => {}
        }
    }

    fn update_shape(&mut self, pos: BlockPos, direction: Face) {
        let state = self.block_or_air(pos);
        let Some(kind) = state.kind() else {
            return;
        };
        match kind.gravity() {
            Gravity::Falling | Gravity::Anvil => self.schedule(pos, kind, FALLING_DELAY),
            Gravity::ConcretePowder => match kind.concrete() {
                Some(concrete) if self.touches_liquid(pos) => {
                    self.set_block(pos, concrete.default_state());
                }
                _ => self.schedule(pos, kind, FALLING_DELAY),
            },
            Gravity::Speleothem => self.update_speleothem(pos, state, kind, direction),
            Gravity::Scaffolding => self.schedule(pos, kind, SCAFFOLDING_DELAY),
            Gravity::None => {}
        }
    }

    fn touches_liquid(&self, pos: BlockPos) -> bool {
        let inside = has_water(self.block_or_air(pos));
        Face::ALL.into_iter().any(|face| {
            let neighbour = self.block_or_air(step(pos, face));
            (face != Face::Down || inside) && has_water(neighbour) && !sturdy(neighbour)
        })
    }

    fn update_speleothem(
        &mut self,
        pos: BlockPos,
        state: BlockStateId,
        kind: BlockKind,
        direction: Face,
    ) {
        let Some(tip) = state.tip_direction() else {
            return;
        };
        if direction.is_horizontal()
            || (tip == Face::Down && self.falls.pending.contains(&(pos, kind)))
        {
            return;
        }
        if direction == tip.opposite() && !self.speleothem_survives(pos, kind, tip) {
            let delay = if tip == Face::Down {
                STALACTITE_DELAY
            } else {
                STALAGMITE_DELAY
            };
            self.schedule(pos, kind, delay);
            return;
        }
        let merge = state.property("thickness") == Some("tip_merge");
        let thickness = self.speleothem_thickness_at(pos, kind, tip, merge);
        if let Some(next) = state.with_property("thickness", thickness)
            && next != state
        {
            self.set_block(pos, next);
        }
    }

    fn speleothem_survives(&self, pos: BlockPos, kind: BlockKind, tip: Face) -> bool {
        let support = self.block_or_air(step(pos, tip.opposite()));
        sturdy(support) || (support.is_speleothem_toward(tip) && support.kind() == Some(kind))
    }

    fn speleothem_thickness_at(
        &self,
        pos: BlockPos,
        kind: BlockKind,
        tip: Face,
        merge: bool,
    ) -> &'static str {
        let toward = self.block_or_air(step(pos, tip));
        let away = self.block_or_air(step(pos, tip.opposite()));
        speleothem_thickness(kind, tip, toward, away, merge)
    }

    fn settle_scaffolding(&self, pos: BlockPos, state: BlockStateId) -> BlockStateId {
        let below = self.block_or_air(step(pos, Face::Down));
        let sides = Face::HORIZONTAL.map(|face| self.block_or_air(step(pos, face)));
        scaffolding_settle(state, below, sturdy(below), sides)
    }

    fn survives(&self, pos: BlockPos, state: BlockStateId) -> bool {
        let Some(kind) = state.kind() else {
            return true;
        };
        match (kind.gravity(), state.tip_direction()) {
            (Gravity::Speleothem, Some(tip)) => self.speleothem_survives(pos, kind, tip),
            (Gravity::Scaffolding, _) => {
                self.settle_scaffolding(pos, state).scaffolding_distance()
                    < Some(SCAFFOLDING_MAX_DISTANCE)
            }
            _ => true,
        }
    }

    pub(crate) fn tick_falling_blocks(&mut self) {
        let now = self.current_tick();
        let due: Vec<(BlockPos, BlockKind)> = {
            let later = self.falls.due.split_off(&(now + 1));
            let ready = std::mem::replace(&mut self.falls.due, later);
            ready.into_values().flatten().collect()
        };
        for (pos, kind) in due {
            self.falls.pending.remove(&(pos, kind));
            let state = self.block_or_air(pos);
            if state.kind() == Some(kind) {
                self.block_tick(pos, state, kind);
            }
        }
        self.move_falling_blocks();
        self.update_falling_tracking();
    }

    fn block_tick(&mut self, pos: BlockPos, state: BlockStateId, kind: BlockKind) {
        match kind.gravity() {
            Gravity::Falling | Gravity::ConcretePowder => self.try_fall(pos, state, None),
            Gravity::Anvil => self.try_fall(pos, state, Some(FallHurt::ANVIL)),
            Gravity::Speleothem => {
                let stalagmite = state.tip_direction() == Some(Face::Up);
                if stalagmite && !self.survives(pos, state) {
                    self.destroy_block(pos, state);
                } else {
                    self.spawn_falling_stalactite(pos, state);
                }
            }
            Gravity::Scaffolding => {
                let settled = self.settle_scaffolding(pos, state);
                let limit = Some(SCAFFOLDING_MAX_DISTANCE);
                if settled.scaffolding_distance() == limit {
                    if state.scaffolding_distance() == limit {
                        self.fall(pos, settled, None);
                    } else {
                        self.destroy_block(pos, state);
                    }
                } else if settled != state {
                    self.set_block(pos, settled);
                }
            }
            Gravity::None => {}
        }
    }

    fn try_fall(&mut self, pos: BlockPos, state: BlockStateId, hurt: Option<FallHurt>) {
        let below = self.world.chunks.block(step(pos, Face::Down));
        if pos.y > self.world.dimension.height.min_y && below.is_some_and(is_free) {
            self.fall(pos, state, hurt);
        }
    }

    fn spawn_falling_stalactite(&mut self, top: BlockPos, state: BlockStateId) {
        let mut pos = top;
        let mut current = state;
        while current.is_speleothem_toward(Face::Down) {
            if current.is_speleothem_tip(true) {
                self.fall(pos, current, Some(FallHurt::stalactite(1 + top.y - pos.y)));
                return;
            }
            self.fall(pos, current, None);
            pos = step(pos, Face::Down);
            current = self.block_or_air(pos);
        }
    }

    fn destroy_block(&mut self, pos: BlockPos, state: BlockStateId) {
        let context = LootContext { state, tool: None };
        for stack in loot::block_drops(&context, &mut self.rng) {
            self.spawn_item(pos, stack);
        }
        self.set_block(pos, left_behind(state));
    }

    fn fall(&mut self, pos: BlockPos, state: BlockStateId, hurt: Option<FallHurt>) {
        let id = self.allocate_entity_id();
        let uuid = uuid::Builder::from_random_bytes(self.rng.random()).into_uuid();
        let position = Vec3::new(
            f64::from(pos.x) + 0.5,
            f64::from(pos.y),
            f64::from(pos.z) + 0.5,
        );
        self.set_block(pos, left_behind(state));
        self.falling.insert(
            id,
            FallingBlock {
                id,
                uuid,
                state: waterlogged(state, false),
                position,
                velocity: Vec3::ZERO,
                time: 0,
                fall_distance: 0.0,
                hurt,
                cancel_drop: false,
                synced_position: position,
            },
        );
    }

    fn floor_below(&self, x: f64, z: f64, from: f64, to: f64) -> Option<i32> {
        let (bx, bz) = (x.floor() as i32, z.floor() as i32);
        let top = from.floor() as i32 - 1;
        let bottom = to.floor() as i32;
        (bottom..=top).rev().find(|&y| {
            self.block_or_air(BlockPos::new(bx, y, bz))
                .flags()
                .contains(StateFlags::HAS_COLLISION)
        })
    }

    fn water_on_path(&self, from: BlockPos, to: BlockPos) -> Option<BlockPos> {
        (to.y..=from.y)
            .rev()
            .map(|y| BlockPos::new(from.x, y, from.z))
            .find(|&pos| {
                let state = self.block_or_air(pos);
                water_source(state) || state.flags().contains(StateFlags::HAS_COLLISION)
            })
            .filter(|&pos| has_water(self.block_or_air(pos)))
    }

    fn move_falling_blocks(&mut self) {
        let ids: Vec<EntityId> = self.falling.keys().copied().collect();
        for id in ids {
            let Some(mut entity) = self.falling.remove(&id) else {
                continue;
            };
            let original = entity.state;
            entity.time += 1;
            entity.velocity.y -= ACCELERATION;
            let from = entity.position;
            let target = from + entity.velocity;
            let floor = if entity.velocity.y < 0.0 {
                self.floor_below(target.x, target.z, from.y, target.y)
            } else {
                None
            };
            let landed_y = floor.map_or(target.y, |y| f64::from(y + 1));
            let moved = landed_y - from.y;
            if moved < 0.0 {
                entity.fall_distance -= f64::from(moved as f32);
            }
            entity.position = Vec3::new(target.x, landed_y, target.z);
            let mut pos = entity.position.block();
            if let Some(y) = floor {
                let below = self.block_or_air(BlockPos::new(pos.x, y, pos.z));
                let distance = entity.fall_distance + self.landing_bonus(below);
                if entity.fall_distance > 0.0 {
                    self.cause_fall_damage(&mut entity, distance);
                }
                entity.fall_distance = 0.0;
            }
            let concrete = gravity(original) == Gravity::ConcretePowder;
            let mut in_water = concrete && has_water(self.block_or_air(pos));
            let speed = entity.velocity.x * entity.velocity.x
                + entity.velocity.y * entity.velocity.y
                + entity.velocity.z * entity.velocity.z;
            if concrete
                && floor.is_none()
                && speed > 1.0
                && let Some(hit) = self.water_on_path(from.block(), pos)
            {
                pos = hit;
                in_water = true;
            }
            if floor.is_some() || in_water {
                self.land(entity, original, pos, in_water);
                continue;
            }
            let height = self.world.dimension.height;
            let outside = pos.y <= height.min_y || pos.y > height.max_y();
            if entity.time > MAX_FALL_TICKS || (entity.time > OUT_OF_WORLD_TICKS && outside) {
                self.drop_falling(entity.id, original, pos);
                continue;
            }
            entity.velocity = Vec3::new(
                entity.velocity.x * DRAG,
                entity.velocity.y * DRAG,
                entity.velocity.z * DRAG,
            );
            if entity.time.is_multiple_of(POSITION_SYNC_TICKS) {
                self.push(
                    Audience::Trackers(id),
                    ClientEvent::EntityMoved {
                        entity: id,
                        from: entity.synced_position,
                        to: entity.position,
                        rotation: None,
                        on_ground: false,
                    },
                );
                entity.synced_position = entity.position;
            }
            self.falling.insert(id, entity);
        }
    }

    fn landing_bonus(&self, below: BlockStateId) -> f64 {
        let stalagmite_tip = below.kind() == Some(BlockKind::PointedDripstone)
            && below.is_speleothem_toward(Face::Up)
            && below.property("thickness") == Some("tip");
        if stalagmite_tip {
            STALAGMITE_LANDING_BONUS
        } else {
            0.0
        }
    }

    fn cause_fall_damage(&mut self, entity: &mut FallingBlock, distance: f64) {
        if entity.hurt.is_none() || gravity(entity.state) != Gravity::Anvil {
            return;
        }
        let chance = anvil_degrade_chance(distance);
        if chance > 0.0 && self.rng.random::<f32>() < chance {
            match entity.state.damaged_anvil() {
                Some(damaged) => entity.state = damaged,
                None => entity.cancel_drop = true,
            }
        }
    }

    fn land(
        &mut self,
        entity: FallingBlock,
        original: BlockStateId,
        pos: BlockPos,
        in_water: bool,
    ) {
        if entity.cancel_drop {
            self.remove_falling(entity.id);
            return;
        }
        let replaced = self.block_or_air(pos);
        let replaceable = replaced.is_air() || replaced.flags().contains(StateFlags::REPLACEABLE);
        let concrete = gravity(original) == Gravity::ConcretePowder;
        let free_below = self
            .world
            .chunks
            .block(step(pos, Face::Down))
            .is_some_and(is_free)
            && !(concrete && in_water);
        if replaceable && !free_below && self.survives(pos, entity.state) {
            let state = if entity.state.property("waterlogged").is_some() && water_source(replaced)
            {
                waterlogged(entity.state, true)
            } else {
                entity.state
            };
            self.remove_falling(entity.id);
            if self.set_block(pos, state) {
                self.on_land(pos, original, replaced);
            } else {
                self.drop_item(original, pos);
            }
            return;
        }
        self.drop_falling(entity.id, original, pos);
    }

    fn on_land(&mut self, pos: BlockPos, original: BlockStateId, replaced: BlockStateId) {
        if let Some(concrete) = original.kind().and_then(BlockKind::concrete)
            && (has_water(replaced) || self.touches_liquid(pos))
        {
            self.set_block(pos, concrete.default_state());
        }
    }

    fn drop_falling(&mut self, id: EntityId, original: BlockStateId, pos: BlockPos) {
        self.remove_falling(id);
        self.drop_item(original, pos);
    }

    fn drop_item(&mut self, original: BlockStateId, pos: BlockPos) {
        if let Some(item) = original.kind().and_then(BlockKind::item) {
            self.spawn_item(pos, ItemStack::new(ItemId(item), NonZeroU8::MIN));
        }
    }

    fn remove_falling(&mut self, id: EntityId) {
        self.falling.remove(&id);
        for player in self.players.values_mut() {
            if player.tracked.remove(&id) {
                player.link.send(ClientEvent::RemoveEntities(vec![id]));
            }
        }
    }

    fn update_falling_tracking(&mut self) {
        let range = TRACKING_RANGE * TRACKING_RANGE;
        for viewer in self.players.values_mut() {
            for entity in self.falling.values() {
                let visible = viewer.position.distance_squared(entity.position) <= range
                    && viewer.streamer.has_sent(entity.position.chunk());
                let tracking = viewer.tracked.contains(&entity.id);
                if visible && !tracking {
                    viewer.tracked.insert(entity.id);
                    viewer.link.send(entity.spawn_event());
                } else if !visible && tracking {
                    viewer.tracked.remove(&entity.id);
                    viewer
                        .link
                        .send(ClientEvent::RemoveEntities(vec![entity.id]));
                }
            }
        }
    }

    pub fn falling_blocks(&self) -> usize {
        self.falling.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anvil_damage_grows_two_per_block_after_the_first_and_caps_at_forty() {
        assert_eq!(FallHurt::ANVIL.damage(0.0), 0.0);
        assert_eq!(FallHurt::ANVIL.damage(1.0), 0.0);
        assert_eq!(FallHurt::ANVIL.damage(1.5), 2.0);
        assert_eq!(FallHurt::ANVIL.damage(3.0), 4.0);
        assert_eq!(FallHurt::ANVIL.damage(21.0), 40.0);
        assert_eq!(FallHurt::ANVIL.damage(80.0), 40.0);
    }

    #[test]
    fn stalactite_damage_scales_with_column_size_of_at_least_six() {
        assert_eq!(FallHurt::stalactite(1).damage(3.0), 12.0);
        assert_eq!(FallHurt::stalactite(4).damage(3.0), 12.0);
        assert_eq!(FallHurt::stalactite(8).damage(3.0), 16.0);
        assert_eq!(FallHurt::stalactite(8).damage(10.0), 40.0);
        assert_eq!(FallHurt::stalactite(8).damage(0.5), 0.0);
    }

    #[test]
    fn anvil_degrade_chance_follows_fall_distance() {
        assert_eq!(anvil_degrade_chance(0.5), 0.0);
        assert_eq!(anvil_degrade_chance(1.0), 0.0);
        assert!((anvil_degrade_chance(2.0) - 0.1).abs() < 1e-6);
        assert!((anvil_degrade_chance(5.5) - 0.3).abs() < 1e-6);
        assert!(anvil_degrade_chance(20.0) >= 1.0);
    }
}
