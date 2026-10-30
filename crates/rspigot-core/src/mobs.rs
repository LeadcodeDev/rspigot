use rand::Rng;
use rspigot_ai::{AiWorld, BlockView as AiView, Living, MobAi, MobView, TargetKind};
use rspigot_domain::entity::living::{self, EQUIPMENT_SLOTS};
use rspigot_domain::entity::{
    Aabb, ActivationType, Entities, Entity, EntityState, FLAG_INVISIBLE, activation_type,
};
use rspigot_domain::item::HOTBAR_START;
use rspigot_domain::player::{GameMode, PlayerId};
use rspigot_domain::world::WorldHeight;
use rspigot_domain::{BlockPos, BlockStateId, ChunkPos, EntityId, Vec3};
use rspigot_entities::EntityKind as Kind;
use rspigot_items::EquipmentSlot;
use rustc_hash::FxHashMap;

use crate::chunks::ChunkManager;
use crate::config::ActivationRanges;
use crate::player::Player;
use crate::ports::ClientLink;
use crate::server::Server;

const OFFHAND_SLOT: usize = 45;
const ARMOR_SLOTS: std::ops::Range<usize> = 5..9;
const NO_DESPAWN_DISTANCE: f64 = 32.0;
const ACTIVATION_HEIGHT: f64 = 256.0;
const IMMUNITY_RECHECK: i64 = 20;
const INACTIVE_SKIP: i32 = 4;
const WORLD_BORDER: i32 = 30_000_000;
const DAY_LENGTH: i64 = 24_000;
const SKY_LIGHT_KEYS: [(i64, f32); 4] = [
    (133, 1.0),
    (11_867, 1.0),
    (13_670, 0.266_666_68),
    (22_330, 0.266_666_68),
];
const MAX_LIGHT: f32 = 15.0;
const BRIGHT_LIMIT: f32 = 0.5;

#[derive(Debug)]
pub(crate) struct MobEntry {
    pub ai: MobAi,
    activated_tick: i64,
    default_active: bool,
    activation: ActivationType,
}

#[derive(Debug, Default)]
pub(crate) struct MobTable {
    pub entries: FxHashMap<EntityId, MobEntry>,
    pub players: Vec<Living>,
}

fn range_of(activation: ActivationType, ranges: &ActivationRanges) -> i32 {
    match activation {
        ActivationType::Monster => ranges.monsters,
        ActivationType::Animal => ranges.animals,
        ActivationType::Raider => ranges.raiders,
        ActivationType::Misc => ranges.misc,
    }
}

fn default_active(kind: Kind, activation: ActivationType, ranges: &ActivationRanges) -> bool {
    range_of(activation, ranges) == 0 || matches!(kind, Kind::Wither | Kind::EnderDragon)
}

fn inflate(area: &Aabb, horizontal: f64, vertical: f64) -> Aabb {
    rspigot_ai::inflate(area, horizontal, vertical, horizontal)
}

fn extra_flag(entity: &Entity, key: &str) -> bool {
    entity.extra.get_bool(key).unwrap_or(false)
}

fn is_baby(entity: &Entity) -> bool {
    entity.base.kind.extends("AgeableMob") && entity.extra.get_i32("Age").is_some_and(|age| age < 0)
}

fn immune(entity: &Entity, target: Option<EntityId>) -> bool {
    let base = &entity.base;
    if base.fluid.in_water || base.remaining_fire_ticks > 0 || !base.on_ground {
        return true;
    }
    let Some(state) = entity.living() else {
        return false;
    };
    if state.hurt_time > 0 || !state.effects.is_empty() {
        return true;
    }
    let kind = base.kind;
    if kind.extends("PathfinderMob") && target.is_some() {
        return true;
    }
    if kind.extends("Animal") {
        if is_baby(entity) || entity.extra.get_i32("InLove").is_some_and(|love| love > 0) {
            return true;
        }
        if kind == Kind::Sheep && extra_flag(entity, "Sheared") {
            return true;
        }
    }
    kind == Kind::Creeper && extra_flag(entity, "ignited")
}

fn armor_cover(filled: usize) -> f32 {
    filled as f32 / ARMOR_SLOTS.len() as f32
}

fn player_target<L>(player: &Player<L>) -> Living {
    let abilities = player.abilities();
    let held = player
        .inventory
        .get(HOTBAR_START + usize::from(player.inventory.selected_hotbar))
        .map(|stack| stack.item);
    let armor = ARMOR_SLOTS
        .filter(|slot| player.inventory.get(*slot).is_some())
        .count();
    Living {
        id: player.entity_id,
        player: true,
        kind: Kind::Player,
        position: player.position,
        eye_height: player.eye_height(),
        bounds: Aabb::of_size(player.position, 0.6, player.height() as f32),
        alive: !player.vitals.removed && !player.vitals.dead && player.vitals.health > 0.0,
        spectator: player.game_mode == GameMode::Spectator,
        creative: player.game_mode == GameMode::Creative,
        invulnerable: abilities.invulnerable,
        discrete: player.sneaking,
        invisible: player.effects.has("minecraft:invisibility"),
        armor_cover: armor_cover(armor),
        main_hand: held,
        off_hand: player.inventory.get(OFFHAND_SLOT).map(|stack| stack.item),
        yaw: player.rotation.yaw,
        pitch: player.rotation.pitch,
        in_water: false,
        baby: false,
    }
}

fn entity_target(entity: &Entity) -> Option<Living> {
    let state = entity.living()?;
    let base = &entity.base;
    let dimensions = base.kind.dimensions();
    let armor = EQUIPMENT_SLOTS
        .iter()
        .filter(|slot| {
            matches!(
                slot,
                EquipmentSlot::Feet
                    | EquipmentSlot::Legs
                    | EquipmentSlot::Chest
                    | EquipmentSlot::Head
            )
        })
        .filter(|slot| state.equipment(**slot).is_some())
        .count();
    Some(Living {
        id: base.id,
        player: false,
        kind: base.kind,
        position: base.position,
        eye_height: f64::from(dimensions.eye_height),
        bounds: base.bounding_box(),
        alive: !base.is_removed() && !state.dead && living::health(base) > 0.0,
        spectator: false,
        creative: false,
        invulnerable: base.invulnerable,
        discrete: false,
        invisible: base
            .data
            .flag(rspigot_domain::entity::DATA_SHARED_FLAGS, FLAG_INVISIBLE),
        armor_cover: armor_cover(armor),
        main_hand: state
            .equipment(EquipmentSlot::Mainhand)
            .map(|stack| stack.item),
        off_hand: state
            .equipment(EquipmentSlot::Offhand)
            .map(|stack| stack.item),
        yaw: base.yaw,
        pitch: base.pitch,
        in_water: base.fluid.in_water,
        baby: is_baby(entity),
    })
}

pub(crate) fn sky_darken(day_time: i64) -> i32 {
    let time = day_time.rem_euclid(DAY_LENGTH);
    let count = SKY_LIGHT_KEYS.len();
    let mut factor = SKY_LIGHT_KEYS[0].1;
    for index in 0..count {
        let (start, from) = SKY_LIGHT_KEYS[index];
        let (end, to) = SKY_LIGHT_KEYS[(index + 1) % count];
        let end = if index + 1 == count {
            end + DAY_LENGTH
        } else {
            end
        };
        let at = if time < start {
            time + DAY_LENGTH
        } else {
            time
        };
        if at >= start && at <= end {
            let alpha = (at - start) as f32 / (end - start) as f32;
            factor = from + alpha * (to - from);
            break;
        }
    }
    (MAX_LIGHT - MAX_LIGHT * factor) as i32
}

pub(crate) struct ChunkView<'a> {
    pub chunks: &'a ChunkManager,
    pub height: WorldHeight,
}

impl AiView for ChunkView<'_> {
    fn state(&self, pos: BlockPos) -> Option<BlockStateId> {
        self.chunks.block(pos)
    }

    fn min_y(&self) -> i32 {
        self.height.min_y
    }

    fn max_y(&self) -> i32 {
        self.height.max_y()
    }

    fn chunk_loaded(&self, chunk: ChunkPos) -> bool {
        self.chunks.is_loaded(chunk)
    }
}

pub(crate) struct MobWorld<'a> {
    view: ChunkView<'a>,
    store: &'a Entities,
    players: &'a [Living],
    difficulty: i32,
    game_time: i64,
    sky_darken: i32,
}

impl AiView for MobWorld<'_> {
    fn state(&self, pos: BlockPos) -> Option<BlockStateId> {
        self.view.state(pos)
    }

    fn min_y(&self) -> i32 {
        self.view.min_y()
    }

    fn max_y(&self) -> i32 {
        self.view.max_y()
    }

    fn chunk_loaded(&self, chunk: ChunkPos) -> bool {
        self.view.chunk_loaded(chunk)
    }
}

impl AiWorld for MobWorld<'_> {
    fn players(&self) -> &[Living] {
        self.players
    }

    fn living(&self, id: EntityId) -> Option<Living> {
        if let Some(player) = self.players.iter().find(|player| player.id == id) {
            return Some(*player);
        }
        self.store.get(id).and_then(entity_target)
    }

    fn living_of(&self, kind: TargetKind, area: &Aabb) -> Vec<Living> {
        self.store
            .iter()
            .filter(|entity| kind.matches(entity.base.kind))
            .filter(|entity| entity.base.bounding_box().intersects(area))
            .filter_map(entity_target)
            .collect()
    }

    fn difficulty(&self) -> i32 {
        self.difficulty
    }

    fn game_time(&self) -> i64 {
        self.game_time
    }

    fn raw_brightness(&self, pos: BlockPos) -> i32 {
        if pos.x < -WORLD_BORDER
            || pos.z < -WORLD_BORDER
            || pos.x >= WORLD_BORDER
            || pos.z >= WORLD_BORDER
        {
            return MAX_LIGHT as i32;
        }
        let Some(chunk) = self.view.chunks.get(pos.chunk()) else {
            return 0;
        };
        let sections = chunk.light.sky.len().max(1);
        let index = ((pos.y >> 4) - self.view.height.min_section() + 1)
            .clamp(0, sections as i32 - 1) as usize;
        let (x, y, z) = (
            (pos.x & 15) as usize,
            (pos.y & 15) as usize,
            (pos.z & 15) as usize,
        );
        let level = |layers: &[Option<rspigot_domain::chunk::LightArray>]| {
            layers
                .get(index)
                .and_then(Option::as_ref)
                .map_or(0, |layer| i32::from(layer.get(x, y, z)))
        };
        (level(&chunk.light.sky) - self.sky_darken).max(level(&chunk.light.block))
    }
}

impl<L: ClientLink> Server<L> {
    fn chunk_view(&self) -> ChunkView<'_> {
        ChunkView {
            chunks: &self.world.chunks,
            height: self.world.dimension.height,
        }
    }

    pub(crate) fn register_mob(&mut self, id: EntityId) {
        let Some(entity) = self.entities.store.get(id) else {
            return;
        };
        if !entity.living().is_some_and(|state| state.is_mob()) {
            return;
        }
        let kind = entity.base.kind;
        let activation = activation_type(kind);
        let ranges = self.config.activation;
        let seed = self.rng.random::<i64>();
        self.entities.mobs.entries.insert(
            id,
            MobEntry {
                ai: MobAi::new(kind, seed),
                activated_tick: i64::from(i32::MIN),
                default_active: default_active(kind, activation, &ranges),
                activation,
            },
        );
    }

    pub(crate) fn refresh_mob_players(&mut self) {
        let mut players: Vec<Living> = self
            .players
            .values()
            .filter(|player| !player.vitals.removed)
            .map(player_target)
            .collect();
        players.sort_by_key(|player| player.id);
        self.entities.mobs.players = players;
    }

    pub(crate) fn activate_entities(&mut self) {
        let ranges = self.config.activation;
        let current = self.current_tick() as i64;
        let simulation = i32::from(self.config.simulation_distance) * 16 - 8;
        let max_range = ranges
            .monsters
            .max(ranges.animals)
            .max(ranges.raiders)
            .max(ranges.misc)
            .min(simulation);
        let boxes: Vec<Aabb> = self
            .entities
            .mobs
            .players
            .iter()
            .filter(|player| !(ranges.ignore_spectators && player.spectator))
            .map(|player| player.bounds)
            .collect();
        let store = &self.entities.store;
        for player in boxes {
            let reach = inflate(&player, f64::from(max_range), ACTIVATION_HEIGHT);
            for (id, entry) in &mut self.entities.mobs.entries {
                let Some(entity) = store.get(*id) else {
                    continue;
                };
                let bounds = entity.base.bounding_box();
                if !bounds.intersects(&reach) || current <= entry.activated_tick {
                    continue;
                }
                let range = f64::from(range_of(entry.activation, &ranges));
                if entry.default_active
                    || inflate(&player, range, ACTIVATION_HEIGHT).intersects(&bounds)
                {
                    entry.activated_tick = current;
                }
            }
        }
    }

    pub(crate) fn mob_active(&mut self, id: EntityId) -> bool {
        let current = self.current_tick() as i64;
        let Some(entry) = self.entities.mobs.entries.get_mut(&id) else {
            return true;
        };
        let Some(entity) = self.entities.store.get_mut(id) else {
            return false;
        };
        let position = entity.base.position;
        let near = self
            .entities
            .mobs
            .players
            .iter()
            .filter(|player| !player.spectator)
            .map(|player| player.position.distance_squared(position))
            .reduce(f64::min)
            .is_some_and(|distance| distance < NO_DESPAWN_DISTANCE * NO_DESPAWN_DISTANCE);
        if near && let Some(state) = entity.living_mut() {
            state.no_action_time = 0;
        }
        let target = entry.ai.target();
        let mut active = entry.activated_tick >= current || entry.default_active;
        if !active {
            if (current - entry.activated_tick - 1) % IMMUNITY_RECHECK == 0 {
                if immune(entity, target) {
                    entry.activated_tick = current + IMMUNITY_RECHECK;
                }
                active = true;
            }
        } else if !entry.default_active
            && entity.base.tick_count % INACTIVE_SKIP == 0
            && !immune(entity, target)
        {
            active = false;
        }
        if !active {
            entity.base.tick_count = entity.base.tick_count.wrapping_add(1);
            if let Some(state) = entity.living_mut() {
                state.no_action_time += 1;
            }
        }
        active
    }

    fn mob_world(&self, sky_darken: i32) -> MobWorld<'_> {
        MobWorld {
            view: self.chunk_view(),
            store: &self.entities.store,
            players: &self.entities.mobs.players,
            difficulty: i32::from(self.world.difficulty.id()),
            game_time: self.world.level.game_time,
            sky_darken,
        }
    }

    fn bright_at_eyes(&self, entity: &Entity, sky_darken: i32) -> bool {
        let world = self.mob_world(sky_darken);
        let base = &entity.base;
        if !world.chunk_loaded(base.position.chunk()) {
            return false;
        }
        let eye = base.position.y + f64::from(base.kind.dimensions().eye_height);
        let pos = Vec3::new(base.position.x, eye, base.position.z).block();
        rspigot_ai::light_magic_value(&world, pos) > BRIGHT_LIMIT
    }

    pub(crate) fn mob_server_ai_step(&mut self, id: EntityId) {
        let sky_darken = sky_darken(self.world.level.day_time);
        let bright = self.entities.store.get(id).is_some_and(|entity| {
            entity.base.kind.extends("Monster") && self.bright_at_eyes(entity, sky_darken)
        });
        let Some(entry) = self.entities.mobs.entries.get_mut(&id) else {
            return;
        };
        let Some(Entity {
            base,
            state: EntityState::Living(state),
            ..
        }) = self.entities.store.get_mut(id)
        else {
            return;
        };
        if bright {
            state.no_action_time += 2;
        }
        if living::health(base) <= 0.0 || !living::effective_ai(base, state) {
            return;
        }
        state.no_action_time += 1;
        if !state.mob.as_ref().is_some_and(|mob| mob.aware) {
            return;
        }
        let mob = MobView::of(base, state);
        let world = MobWorld {
            view: ChunkView {
                chunks: &self.world.chunks,
                height: self.world.dimension.height,
            },
            store: &self.entities.store,
            players: &self.entities.mobs.players,
            difficulty: i32::from(self.world.difficulty.id()),
            game_time: self.world.level.game_time,
            sky_darken,
        };
        entry.ai.think(&mob, &world);
        let view = ChunkView {
            chunks: &self.world.chunks,
            height: self.world.dimension.height,
        };
        if let Some(Entity {
            base,
            state: EntityState::Living(state),
            ..
        }) = self.entities.store.get_mut(id)
        {
            entry.ai.act(base, state, &view);
        }
    }

    pub(crate) fn mob_paths_block_changed(
        &mut self,
        pos: BlockPos,
        old: BlockStateId,
        new: BlockStateId,
    ) {
        if old.collision_shape_id() == new.collision_shape_id() {
            return;
        }
        let world = MobWorld {
            view: ChunkView {
                chunks: &self.world.chunks,
                height: self.world.dimension.height,
            },
            store: &self.entities.store,
            players: &self.entities.mobs.players,
            difficulty: i32::from(self.world.difficulty.id()),
            game_time: self.world.level.game_time,
            sky_darken: sky_darken(self.world.level.day_time),
        };
        for (id, entry) in &mut self.entities.mobs.entries {
            let Some(entity) = world.store.get(*id) else {
                continue;
            };
            let Some(state) = entity.living() else {
                continue;
            };
            let mob = MobView::of(&entity.base, state);
            entry.ai.block_changed(pos, &mob, &world);
        }
    }

    pub(crate) fn mob_can_float(&self, id: EntityId) -> bool {
        self.entities
            .mobs
            .entries
            .get(&id)
            .is_some_and(|entry| entry.ai.can_float())
    }

    pub(crate) fn mob_after_tick(&mut self, id: EntityId) {
        let Some(entry) = self.entities.mobs.entries.get_mut(&id) else {
            return;
        };
        let Some(Entity {
            base,
            state: EntityState::Living(state),
            ..
        }) = self.entities.store.get_mut(id)
        else {
            return;
        };
        entry.ai.turn_body(base, state);
        if base.tick_count % rspigot_ai::CONTROL_FLAG_INTERVAL == 0 {
            entry.ai.update_control_flags();
        }
        if base.kind.extends("Spider") {
            let climbing = base.horizontal_collision;
            living::set_climbing(base, climbing);
        }
    }

    pub fn player_entity_id(&self, player: PlayerId) -> Option<EntityId> {
        self.players.get(&player).map(|player| player.entity_id)
    }

    pub fn mob_target(&self, id: EntityId) -> Option<EntityId> {
        self.entities.mobs.entries.get(&id)?.ai.target()
    }

    pub fn mob_ai(&self, id: EntityId) -> Option<&MobAi> {
        self.entities.mobs.entries.get(&id).map(|entry| &entry.ai)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sky_darkens_at_night_following_the_day_timeline() {
        assert_eq!(sky_darken(6000), 0);
        assert_eq!(sky_darken(18_000), 11);
        assert_eq!(sky_darken(0), sky_darken(24_000));
        assert!(sky_darken(12_768) > 0 && sky_darken(12_768) < 11);
    }
}
