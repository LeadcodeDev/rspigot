use rand::Rng;
use rspigot_ai::{AiWorld, BlockView as AiView, Living, MobAi, MobView, TargetKind};
use rspigot_domain::damage::{DamageTag, DamageType};
use rspigot_domain::entity::living::{
    self, ATTACK_DAMAGE, ATTACK_KNOCKBACK, EQUIPMENT_SLOTS, HurtOutcome, HurtSource,
    KNOCKBACK_RESISTANCE,
};
use rspigot_domain::entity::physics::{Landing, water_height};
use rspigot_domain::entity::{
    Aabb, ActivationType, Entities, Entity, EntityState, FLAG_INVISIBLE, RemovalReason,
    activation_type, reward,
};
use rspigot_domain::item::{HOTBAR_START, ItemStack};
use rspigot_domain::mth;
use rspigot_domain::player::{GameMode, PlayerId};
use rspigot_domain::text::Component;
use rspigot_domain::world::WorldHeight;
use rspigot_domain::{BlockPos, BlockStateId, ChunkPos, EntityId, Vec3};
use rspigot_entities::{
    CreatureSound, CreatureSounds, EntityKind as Kind, SoundState, WaterContact,
};
use rspigot_items::EquipmentSlot;
use rustc_hash::FxHashMap;

use crate::chunks::ChunkManager;
use crate::commands::Target;
use crate::config::{ActivationRanges, LevelData};
use crate::events::{ClientEvent, EntityStatus};
use crate::fall::fall_impact;
use crate::health::{DamageSource, SourceEntity};
use crate::lighting::light_level;
use crate::loot::{self, LootContext, LootDamage, LootEntity, RaiderFacts};
use crate::player::Player;
use crate::ports::ClientLink;
use crate::server::{Audience, Server};

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
const STORM_SKY_LIGHT: f32 = 4.0;
const RAIN_SKY_ALPHA: f32 = 0.3125;
const THUNDER_SKY_ALPHA: f32 = 0.527_343_75;
const BRIGHT_LIMIT: f32 = 0.5;
pub const DEFAULT_KNOCKBACK: f64 = 0.400_000_005_960_464_5;
pub const MIN_KNOCKBACK_DIRECTION: f64 = 9.999_999_747_378_752e-6;
const RANDOM_DIRECTION_SCALE: f64 = 0.01;
const KNOCKBACK_VERTICAL_CAP: f64 = 0.4;
const VOID_DAMAGE: f32 = 4.0;
const VOID_DEPTH: i32 = 64;
const ITEM_PICKUP_DELAY: u16 = 10;
const ITEM_SPREAD: f64 = 0.2;
const ITEM_LIFT: f64 = 0.2;
const ATTACKER_SLOWDOWN: f64 = 0.6;
const BIG_FALL: i32 = 4;
const DYES: [&str; 16] = [
    "white",
    "orange",
    "magenta",
    "light_blue",
    "yellow",
    "lime",
    "pink",
    "gray",
    "light_gray",
    "cyan",
    "purple",
    "blue",
    "brown",
    "green",
    "red",
    "black",
];
const WOOL_COLOR_MASK: i64 = 0x0F;

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
        if entity.is_baby() || entity.extra.get_i32("InLove").is_some_and(|love| love > 0) {
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
        baby: entity.is_baby(),
    })
}

fn lerp(delta: f32, start: f32, end: f32) -> f32 {
    start + delta * (end - start)
}

pub(crate) fn sky_darken(level: &LevelData) -> i32 {
    let mut light = MAX_LIGHT * sky_light_factor(level.day_time);
    let rain = if level.raining { 1.0 } else { 0.0 };
    let thunder = if level.raining && level.thundering {
        1.0
    } else {
        0.0
    };
    let rain_only = rain - thunder;
    if rain_only > 0.0 {
        light = lerp(
            rain_only,
            light,
            lerp(RAIN_SKY_ALPHA, light, STORM_SKY_LIGHT),
        );
    }
    if thunder > 0.0 {
        light = lerp(
            thunder,
            light,
            lerp(THUNDER_SKY_ALPHA, light, STORM_SKY_LIGHT),
        );
    }
    (MAX_LIGHT - light) as i32
}

fn sky_light_factor(day_time: i64) -> f32 {
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
    factor
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
        (light_level(chunk, pos, true) - self.sky_darken).max(light_level(chunk, pos, false))
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
        let sky_darken = sky_darken(&self.world.level);
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
            sky_darken: sky_darken(&self.world.level),
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

pub(crate) struct ServerRandom<'a, R: Rng + ?Sized>(pub &'a mut R);

impl<R: Rng + ?Sized> rspigot_domain::random::RandomSource for ServerRandom<'_, R> {
    fn next_float(&mut self) -> f32 {
        self.0.random::<f32>()
    }

    fn next_int(&mut self, bound: i32) -> i32 {
        if bound <= 0 {
            0
        } else {
            self.0.random_range(0..bound)
        }
    }
}

pub fn knockback_velocity(current: Vec3, strength: f64, dx: f64, dz: f64, on_ground: bool) -> Vec3 {
    let push = Vec3::new(dx, 0.0, dz).normalize().scale(strength);
    Vec3::new(
        current.x / 2.0 - push.x,
        if on_ground {
            KNOCKBACK_VERTICAL_CAP.min(current.y / 2.0 + strength)
        } else {
            current.y
        },
        current.z / 2.0 - push.z,
    )
}

fn sound_state<V: rspigot_domain::entity::BlockView>(entity: &Entity, view: &V) -> SoundState {
    let base = &entity.base;
    let water = if !base.fluid.in_water {
        WaterContact::Dry
    } else {
        let eye = base.position.y + f64::from(base.kind.dimensions().eye_height)
            - 0.111_111_111_938_953_4;
        let pos = Vec3::new(base.position.x, eye, base.position.z).block();
        let submerged =
            water_height(view, pos).is_some_and(|height| eye < f64::from(pos.y) + height);
        if submerged {
            WaterContact::Submerged
        } else {
            WaterContact::Touching
        }
    };
    SoundState {
        baby: entity.is_baby(),
        tiny: reward::cube_size(entity) <= 1,
        water,
    }
}

fn loot_facts(entity: &Entity) -> LootEntity {
    let base = &entity.base;
    let kind = base.kind;
    let mut facts = LootEntity::of(kind);
    facts.on_fire = !kind.fire_immune() && base.remaining_fire_ticks > 0;
    facts.baby = entity.is_baby();
    if let Some(living) = entity.living() {
        facts.mainhand = living.equipment(EquipmentSlot::Mainhand).cloned();
    }
    if kind == Kind::Sheep {
        let wool = entity.extra.get_i64("Color").unwrap_or(0) & WOOL_COLOR_MASK;
        let dye = DYES.get(wool as usize).copied().unwrap_or(DYES[0]);
        facts
            .components
            .push(("minecraft:sheep/color", dye.to_owned()));
        facts.sheared = Some(entity.extra.get_bool("Sheared").unwrap_or(false));
    }
    if kind == Kind::Frog {
        let variant = entity
            .extra
            .get_str("variant")
            .unwrap_or("minecraft:temperate");
        facts
            .components
            .push(("minecraft:frog/variant", variant.to_owned()));
    }
    if kind.extends("AbstractCubeMob") {
        facts.cube_size = Some(reward::cube_size(entity));
    }
    if kind.extends("Raider") {
        facts.raider = Some(RaiderFacts {
            has_raid: false,
            captain: false,
        });
    }
    facts
}

impl<L: ClientLink> Server<L> {
    pub(crate) fn mob_source(&self, id: EntityId) -> Option<SourceEntity> {
        let entity = self.entities.store.get(id)?;
        Some(SourceEntity {
            entity: id,
            player: None,
            kind: entity.base.kind,
            creative: false,
            name: self.entity_name(id)?,
            position: entity.base.position,
        })
    }

    fn living_entity_alive(&self, id: EntityId) -> bool {
        if let Some(player) = self.players.values().find(|p| p.entity_id == id) {
            return !player.vitals.is_dead();
        }
        self.entities.store.get(id).is_some_and(|e| {
            !e.base.is_removed()
                && e.living()
                    .is_some_and(|l| !living::is_dead_or_dying(&e.base, l))
        })
    }

    fn entity_loot_facts(&self, id: EntityId) -> Option<LootEntity> {
        if let Some(player) = self.players.values().find(|p| p.entity_id == id) {
            let mut facts = LootEntity::of(Kind::Player);
            facts.mainhand = player.inventory.held().cloned();
            return Some(facts);
        }
        self.entities.store.get(id).map(loot_facts)
    }

    pub(crate) fn hurt_mob(&mut self, id: EntityId, source: &DamageSource, amount: f32) -> bool {
        let now = self.current_tick();
        let game_time = self.world.level.game_time;
        let kind = source.kind;
        let causing_living = source
            .causing
            .as_ref()
            .filter(|c| c.kind.living())
            .map(|c| c.entity);
        let hurt_source = HurtSource {
            kind,
            causing: source.causing.as_ref().map(|c| &c.name),
            direct: source.direct.as_ref().map(|d| &d.name),
            living_cause: source.living_cause(),
            creative_attacker: source.creative_attacker(),
        };
        let Some(Entity {
            base,
            state: EntityState::Living(state),
            ..
        }) = self.entities.store.get_mut(id)
        else {
            return false;
        };
        let outcome = living::hurt(base, state, &hurt_source, amount, now);
        if outcome == HurtOutcome::Ignored {
            return false;
        }
        living::remember_attacker(
            state,
            kind,
            causing_living,
            source.attacker(),
            base.tick_count,
        );
        if outcome == HurtOutcome::Full && !kind.is(DamageTag::NoImpact) {
            base.sync_velocity = true;
        }
        state.damage.last_source = Some((kind, game_time));
        let position = base.position;
        let dead = living::is_dead_or_dying(base, state);
        if let Some(attacker) = source.attacker()
            && let Some(player) = self.players.get_mut(&attacker)
        {
            player.vitals.attack_ticker = 0;
        }
        if outcome == HurtOutcome::Full {
            self.push(
                Audience::Trackers(id),
                ClientEvent::DamageEvent {
                    entity: id,
                    kind,
                    cause: source.causing.as_ref().map(|c| c.entity),
                    direct: source.direct.as_ref().map(|d| d.entity),
                    position: source.position,
                },
            );
            if !kind.is(DamageTag::NoKnockback) {
                let (dx, dz) = source
                    .source_position()
                    .map_or((0.0, 0.0), |s| (s.x - position.x, s.z - position.z));
                self.mob_knockback(id, DEFAULT_KNOCKBACK, dx, dz);
            }
        }
        if dead {
            if outcome == HurtOutcome::Full {
                self.make_entity_sound(id, |sounds| sounds.death);
            }
            self.die_mob(id, source);
        } else if outcome == HurtOutcome::Full {
            self.make_entity_sound(id, |sounds| sounds.hurt);
        }
        true
    }

    pub(crate) fn mob_knockback(&mut self, id: EntityId, strength: f64, dx: f64, dz: f64) {
        let Some(Entity {
            state: EntityState::Living(state),
            ..
        }) = self.entities.store.get_mut(id)
        else {
            return;
        };
        let strength = strength * (1.0 - state.attribute(KNOCKBACK_RESISTANCE));
        let (mut dx, mut dz) = (dx, dz);
        while dx * dx + dz * dz < MIN_KNOCKBACK_DIRECTION {
            dx = (self.rng.random::<f64>() - self.rng.random::<f64>()) * RANDOM_DIRECTION_SCALE;
            dz = (self.rng.random::<f64>() - self.rng.random::<f64>()) * RANDOM_DIRECTION_SCALE;
        }
        let Some(entity) = self.entities.store.get_mut(id) else {
            return;
        };
        let base = &mut entity.base;
        base.velocity = knockback_velocity(base.velocity, strength, dx, dz, base.on_ground);
        base.needs_sync = true;
    }

    pub(crate) fn make_entity_sound(
        &mut self,
        id: EntityId,
        pick: fn(CreatureSounds) -> CreatureSound,
    ) {
        let Some(entity) = self.entities.store.get(id) else {
            return;
        };
        if entity.base.is_silent() {
            return;
        }
        let Some(sounds) = entity.base.kind.sounds() else {
            return;
        };
        let state = sound_state(entity, &self.world.chunks);
        let Some(sound) = pick(sounds).get(state) else {
            return;
        };
        let position = entity.base.position;
        let volume = sounds.volume.get(state.baby, reward::cube_size(entity));
        let pitch = self.voice_pitch(sounds.pitch, state.baby);
        self.play_sound(None, position, sound, sounds.source, volume, pitch);
    }

    fn entity_sound(&mut self, id: EntityId, sound: &'static str, volume: f32, pitch: f32) {
        let Some(entity) = self.entities.store.get(id) else {
            return;
        };
        if entity.base.is_silent() {
            return;
        }
        let source = entity
            .base
            .kind
            .sounds()
            .map_or(rspigot_entities::SoundSource::Neutral, |s| s.source);
        let position = entity.base.position;
        self.play_sound(None, position, sound, source, volume, pitch);
    }

    fn die_mob(&mut self, id: EntityId, source: &DamageSource) {
        let level = self.world.level.name.clone();
        let credit = self.entities.store.get(id).and_then(|e| {
            let living = e.living()?;
            let player = living.damage.last_hurt_by_player?;
            self.players.get(&player).map(|p| Component::text(p.name()))
        });
        let victim = self.entity_name(id);
        let Some(Entity {
            base,
            state: EntityState::Living(state),
            ..
        }) = self.entities.store.get_mut(id)
        else {
            return;
        };
        if base.is_removed() || state.dead {
            return;
        }
        if let (Some(name), Some(victim)) = (base.custom_name(), victim) {
            let message = state.damage.combat.death_message(&victim, credit.as_ref());
            tracing::info!(
                "Named entity {}['{}'/{}, l='ServerLevel[{}]', x={:.2}, y={:.2}, z={:.2}] died: {}",
                base.kind.class(),
                name.plain_text(),
                id.0,
                level,
                base.position.x,
                base.position.y,
                base.position.z,
                crate::console::console_text(&message)
            );
        }
        state.dead = true;
        self.drop_all_death_loot(id, source);
        if let Some(entity) = self.entities.store.get_mut(id) {
            entity.base.set_pose(rspigot_domain::entity::Pose::Dying);
        }
        self.push(
            Audience::Trackers(id),
            ClientEvent::EntityStatus {
                entity: id,
                status: EntityStatus::Death,
            },
        );
    }

    fn damage_facts(&self, source: &DamageSource) -> LootDamage {
        LootDamage {
            kind: source.kind,
            source: source
                .causing
                .as_ref()
                .and_then(|c| self.entity_loot_facts(c.entity)),
            direct: source
                .direct
                .as_ref()
                .and_then(|d| self.entity_loot_facts(d.entity)),
            is_direct: source.causing.as_ref().map(|c| c.entity)
                == source.direct.as_ref().map(|d| d.entity),
        }
    }

    fn drop_all_death_loot(&mut self, id: EntityId, source: &DamageSource) {
        let mob_drops = self.config.rules.mob_drops;
        let Some(entity) = self.entities.store.get(id) else {
            return;
        };
        let Some(living) = entity.living() else {
            return;
        };
        let killed_by_player = living.damage.player_memory > 0;
        let player = living
            .damage
            .last_hurt_by_player
            .filter(|_| killed_by_player)
            .and_then(|p| self.players.get(&p));
        let player_facts = player.map(|p| {
            let mut facts = LootEntity::of(Kind::Player);
            facts.mainhand = p.inventory.held().cloned();
            facts
        });
        let position = entity.base.position;
        let drop_loot = reward::should_drop_loot(entity, mob_drops);
        let table = living
            .mob
            .as_ref()
            .and_then(|mob| mob.death_loot_table.as_deref())
            .and_then(rspigot_data::loot::table)
            .or_else(|| rspigot_data::loot::entity_table(entity.base.kind.id().0));
        let this = loot_facts(entity);
        let damage = self.damage_facts(source);
        let mut drops: Vec<ItemStack> = Vec::new();
        if drop_loot {
            if let Some(table) = table {
                let context = LootContext {
                    block: None,
                    tool: None,
                    this: Some(&this),
                    damage: Some(&damage),
                    player: player_facts.as_ref(),
                    luck: 0.0,
                };
                drops.extend(loot::table_drops(table, &context, &mut self.rng));
            }
            drops.extend(self.drop_equipment(id, killed_by_player));
        }
        for stack in drops {
            let velocity = Vec3::new(
                self.rng.random::<f64>() * ITEM_SPREAD - ITEM_SPREAD / 2.0,
                ITEM_LIFT,
                self.rng.random::<f64>() * ITEM_SPREAD - ITEM_SPREAD / 2.0,
            );
            self.spawn_item_entity(position, velocity, ITEM_PICKUP_DELAY, stack);
        }
        let experience = self.experience_reward(id, killed_by_player, mob_drops);
        self.award_experience(position, experience);
    }

    fn drop_equipment(&mut self, id: EntityId, killed_by_player: bool) -> Vec<ItemStack> {
        let rng = &mut self.rng;
        let Some(living) = self.entities.store.get_mut(id).and_then(|e| e.living_mut()) else {
            return Vec::new();
        };
        let drops = reward::equipment_drops(living, killed_by_player, &mut ServerRandom(rng));
        if drops.is_empty() {
            return Vec::new();
        }
        let slots = drops
            .iter()
            .map(|(slot, _)| (crate::entities::wire_slot(*slot), None))
            .collect();
        self.push(
            Audience::Trackers(id),
            ClientEvent::EntityEquipment { entity: id, slots },
        );
        drops.into_iter().map(|(_, stack)| stack).collect()
    }

    fn experience_reward(&mut self, id: EntityId, killed_by_player: bool, mob_drops: bool) -> i32 {
        let Some(entity) = self.entities.store.get(id) else {
            return 0;
        };
        if !(killed_by_player && reward::should_drop_experience(entity) && mob_drops) {
            return 0;
        }
        reward::base_experience_reward(entity, &mut ServerRandom(&mut self.rng))
    }

    pub(crate) fn kill_entity(&mut self, id: EntityId) -> bool {
        let living = self
            .entities
            .store
            .get(id)
            .is_some_and(|e| e.living().is_some());
        if living {
            self.hurt_mob(id, &DamageSource::new(DamageType::GenericKill), f32::MAX);
        } else {
            self.remove_entity(id, RemovalReason::Killed);
        }
        true
    }

    pub(crate) fn void_check(&mut self, id: EntityId) {
        let floor = f64::from(self.world.dimension.height.min_y - VOID_DEPTH);
        let below = self
            .entities
            .store
            .get(id)
            .is_some_and(|e| e.base.position.y < floor);
        if below {
            self.hurt_mob(id, &DamageSource::new(DamageType::OutOfWorld), VOID_DAMAGE);
        }
    }

    pub(crate) fn forget_dead_attackers(&mut self, id: EntityId) {
        let now = self.current_tick();
        let (mob, target) = match self.entities.store.get(id).and_then(|e| e.living()) {
            Some(living) => (living.damage.last_hurt_by_mob, living.damage.last_hurt_mob),
            None => return,
        };
        let mob_alive = mob.is_some_and(|m| self.living_entity_alive(m));
        let target_alive = target.is_some_and(|t| self.living_entity_alive(t));
        if let Some(Entity {
            base,
            state: EntityState::Living(state),
            ..
        }) = self.entities.store.get_mut(id)
        {
            if !target_alive {
                state.damage.last_hurt_mob = None;
            }
            if !mob_alive {
                state.damage.last_hurt_by_mob = None;
            }
            let alive = !living::is_dead_or_dying(base, state);
            state.damage.combat.recheck(now, alive);
        }
    }

    pub(crate) fn mob_landed(&mut self, id: EntityId, landing: Landing) {
        let Some(impact) = fall_impact(landing.state, landing.distance, false) else {
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
        let damage = living::fall_damage(base, state, impact.distance, impact.multiplier);
        if damage <= 0 {
            return;
        }
        if let Some(sounds) = base.kind.sounds() {
            let sound = if damage > BIG_FALL {
                sounds.big_fall
            } else {
                sounds.small_fall
            };
            self.entity_sound(id, sound, 1.0, 1.0);
        }
        self.hurt_mob(id, &DamageSource::new(impact.kind), damage as f32);
    }

    pub fn do_hurt_target(&mut self, attacker: EntityId, target: Target) -> bool {
        let Some(source_entity) = self.mob_source(attacker) else {
            return false;
        };
        let Some(Entity {
            base,
            state: EntityState::Living(state),
            ..
        }) = self.entities.store.get_mut(attacker)
        else {
            return false;
        };
        let damage = state.attribute(ATTACK_DAMAGE) as f32;
        let knockback = state.attribute(ATTACK_KNOCKBACK) as f32 / 2.0;
        let yaw = base.yaw;
        let source = DamageSource::by(DamageType::MobAttack, source_entity);
        let hurt = match target {
            Target::Player(player) => self.hurt(player, &source, damage),
            Target::Entity(entity) => self.hurt_mob(entity, &source, damage),
        };
        if !hurt {
            return false;
        }
        if knockback > 0.0 {
            let (dx, dz) = mth::yaw_direction(yaw);
            let living_target = match target {
                Target::Player(player) => {
                    self.knockback(player, f64::from(knockback), dx, dz);
                    true
                }
                Target::Entity(entity) => {
                    let living = self
                        .entities
                        .store
                        .get(entity)
                        .is_some_and(|e| e.living().is_some());
                    if living {
                        self.mob_knockback(entity, f64::from(knockback), dx, dz);
                    }
                    living
                }
            };
            if living_target && let Some(entity) = self.entities.store.get_mut(attacker) {
                entity.base.velocity =
                    entity
                        .base
                        .velocity
                        .multiply(ATTACKER_SLOWDOWN, 1.0, ATTACKER_SLOWDOWN);
            }
        }
        let target_entity = match target {
            Target::Player(player) => self.players.get(&player).map(|p| p.entity_id),
            Target::Entity(entity) => Some(entity),
        };
        let tick_count = self
            .entities
            .store
            .get(attacker)
            .map_or(0, |e| e.base.tick_count);
        if let Some(living) = self
            .entities
            .store
            .get_mut(attacker)
            .and_then(|e| e.living_mut())
        {
            living.damage.last_hurt_mob = target_entity;
            living.damage.last_hurt_mob_at = tick_count;
        }
        true
    }

    pub fn mob_damage(&self, id: EntityId) -> Option<MobDamageSnapshot> {
        let entity = self.entities.store.get(id)?;
        let living = entity.living()?;
        Some(MobDamageSnapshot {
            health: living::health(&entity.base),
            hurt_time: living.hurt_time,
            death_time: living.death_time,
            dead: living.dead,
            cooldown: living.damage.cooldown,
            last_hurt_by_player: living.damage.last_hurt_by_player,
            player_memory: living.damage.player_memory,
            last_hurt_by_mob: living.damage.last_hurt_by_mob,
            equipment: EQUIPMENT_SLOTS
                .iter()
                .filter_map(|slot| living.equipment(*slot).map(|stack| (*slot, stack.clone())))
                .collect(),
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MobDamageSnapshot {
    pub health: f32,
    pub hurt_time: i32,
    pub death_time: i32,
    pub dead: bool,
    pub cooldown: i32,
    pub last_hurt_by_player: Option<PlayerId>,
    pub player_memory: i32,
    pub last_hurt_by_mob: Option<EntityId>,
    pub equipment: Vec<(rspigot_items::EquipmentSlot, ItemStack)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sky_darkens_at_night_following_the_day_timeline() {
        let at = |day_time| LevelData {
            day_time,
            ..LevelData::default()
        };
        assert_eq!(sky_darken(&at(6000)), 0);
        assert_eq!(sky_darken(&at(18_000)), 11);
        assert_eq!(sky_darken(&at(0)), sky_darken(&at(24_000)));
        assert!(sky_darken(&at(12_768)) > 0 && sky_darken(&at(12_768)) < 11);
    }

    #[test]
    fn rain_and_thunder_darken_the_day_sky() {
        let weather = |raining, thundering, day_time| LevelData {
            day_time,
            raining,
            thundering,
            ..LevelData::default()
        };
        assert_eq!(sky_darken(&weather(true, false, 6000)), 3);
        assert_eq!(sky_darken(&weather(true, true, 6000)), 5);
        assert_eq!(sky_darken(&weather(false, true, 6000)), 0);
        assert_eq!(sky_darken(&weather(true, true, 18_000)), 11);
    }

    #[test]
    fn knockback_halves_motion_and_pushes_away_from_the_source() {
        let pushed = knockback_velocity(Vec3::ZERO, DEFAULT_KNOCKBACK, 1.0, 0.0, true);
        assert_eq!(pushed, Vec3::new(-DEFAULT_KNOCKBACK, 0.4, 0.0));
        let airborne = knockback_velocity(Vec3::new(0.2, -0.3, 0.0), 0.5, 0.0, 2.0, false);
        assert_eq!(airborne, Vec3::new(0.1, -0.3, -0.5));
        let resisted = knockback_velocity(Vec3::new(0.4, 0.2, -0.4), 0.0, 1.0, 1.0, true);
        assert_eq!(resisted, Vec3::new(0.2, 0.1, -0.2));
    }

    #[test]
    fn sheep_colour_and_cube_size_become_loot_facts() {
        use rspigot_domain::entity::{EntityBase, LivingState};
        let mut base = EntityBase::new(EntityId(1), uuid::Uuid::from_u128(1), Kind::Sheep);
        let living = LivingState::new(&mut base, 0.0);
        let mut sheep = Entity::new(base, EntityState::Living(Box::new(living)));
        sheep.extra.insert("Color", 14i8);
        let facts = loot_facts(&sheep);
        assert_eq!(
            facts.components,
            vec![("minecraft:sheep/color", "red".to_owned())]
        );
        assert_eq!(facts.sheared, Some(false));
        let mut base = EntityBase::new(EntityId(2), uuid::Uuid::from_u128(2), Kind::Slime);
        let living = LivingState::new(&mut base, 0.0);
        let mut slime = Entity::new(base, EntityState::Living(Box::new(living)));
        slime.extra.insert("Size", 3);
        assert_eq!(loot_facts(&slime).cube_size, Some(4));
    }
}
