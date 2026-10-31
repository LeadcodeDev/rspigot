use rand::Rng;
use rspigot_domain::attribute::{
    ARMOR, ARMOR_TOUGHNESS, Attributes, MAX_ABSORPTION, MAX_HEALTH, equipment_modifiers,
    is_syncable,
};
use rspigot_domain::combat::{CombatEntry, CombatTracker};
use rspigot_domain::damage::{self, DamageTag, DamageType, java_clamp};
use rspigot_domain::effect::EffectId;
use rspigot_domain::experience::PlayerExperience;
use rspigot_domain::food::{FoodData, FoodEffect};
use rspigot_domain::item::{HOTBAR_START, Inventory, ItemStack, PLAYER_INVENTORY_SIZE};
use rspigot_domain::player::{GameMode, PlayerId};
use rspigot_domain::text::Component;
use rspigot_domain::world::Difficulty;
use rspigot_domain::{BlockPos, EntityId, ItemId, Rotation, Vec3};
use rspigot_entities::EntityKind as Kind;
use rspigot_items::EquipmentSlot as ItemSlot;

use crate::events::{ClientEvent, EntityStatus, GameEvent, MetadataEntry, MetadataValue};
use crate::player::Player;
use crate::ports::ClientLink;
use crate::server::{Audience, Server};

pub const HEAD_SLOT: usize = 5;
pub const CHEST_SLOT: usize = 6;
pub const LEGS_SLOT: usize = 7;
pub const FEET_SLOT: usize = 8;
pub const OFFHAND_SLOT: usize = 45;
const DAMAGE_COOLDOWN: u32 = 20;
const COOLDOWN_THRESHOLD: u32 = 10;
const HURT_DURATION: u32 = 10;
const KILL_CREDIT_TICKS: u32 = 100;
const DEATH_ANIMATION_TICKS: u32 = 20;
const CLIENT_LOAD_TIMEOUT: u32 = 60;
const VOID_DAMAGE: f32 = 4.0;
const VOID_DEPTH: i32 = 64;
const HELMET_FACTOR: f32 = 0.75;
const PEACEFUL_HEAL_INTERVAL: u32 = 20;
const PEACEFUL_FOOD_INTERVAL: u32 = 10;
const DROP_PICKUP_DELAY: u16 = 40;
const DROP_EYE_OFFSET: f64 = 0.3;
const DROP_SPEED: f32 = 0.5;
const DROP_LIFT: f64 = 0.2;
const HEALTH_DATA_INDEX: u8 = 9;
const ABSORPTION_DATA_INDEX: u8 = 17;
const ABSORPTION_PER_LEVEL: f32 = 4.0;

#[derive(Debug, Clone, PartialEq)]
pub struct SourceEntity {
    pub entity: EntityId,
    pub player: Option<PlayerId>,
    pub kind: Kind,
    pub creative: bool,
    pub name: Component,
    pub position: Vec3,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DamageSource {
    pub kind: DamageType,
    pub causing: Option<SourceEntity>,
    pub direct: Option<SourceEntity>,
    pub position: Option<Vec3>,
    pub knockback: Option<(f64, f64)>,
}

impl DamageSource {
    pub fn new(kind: DamageType) -> Self {
        Self {
            kind,
            causing: None,
            direct: None,
            position: None,
            knockback: None,
        }
    }

    pub fn by(kind: DamageType, source: SourceEntity) -> Self {
        Self {
            kind,
            causing: Some(source.clone()),
            direct: Some(source),
            position: None,
            knockback: None,
        }
    }

    pub fn attacker(&self) -> Option<PlayerId> {
        self.causing.as_ref().and_then(|c| c.player)
    }

    pub fn living_cause(&self) -> bool {
        self.causing.as_ref().is_some_and(|c| c.kind.living())
    }

    pub fn scales_with_difficulty(&self) -> bool {
        self.kind.scales_always()
            || self
                .causing
                .as_ref()
                .is_some_and(|c| c.kind.living() && c.kind != Kind::Player)
    }

    pub fn creative_attacker(&self) -> bool {
        self.causing.as_ref().is_some_and(|c| c.creative)
    }

    pub(crate) fn source_position(&self) -> Option<Vec3> {
        self.position
            .or_else(|| self.direct.as_ref().map(|d| d.position))
    }

    pub(crate) fn knockback_from(&self, victim: Vec3) -> (f64, f64) {
        if let Some(direction) = self.knockback {
            return direction;
        }
        self.source_position()
            .map_or((0.0, 0.0), |s| (s.x - victim.x, s.z - victim.z))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KillCredit {
    pub player: PlayerId,
    pub name: Component,
    pub memory: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct SentVitals {
    health: f32,
    food: i32,
    saturation_zero: bool,
    absorption: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Vitals {
    pub health: f32,
    pub absorption: f32,
    pub food: FoodData,
    pub fall_distance: f64,
    pub damage_cooldown: u32,
    pub last_hurt: f32,
    pub hurt_time: u32,
    pub dead: bool,
    pub death_time: u32,
    pub removed: bool,
    pub combat: CombatTracker,
    pub kill_credit: Option<KillCredit>,
    pub attack_ticker: u32,
    pub held_item: Option<ItemId>,
    pub velocity: Vec3,
    pub sync_velocity: bool,
    pub tick_count: u32,
    pub waiting_for_respawn: bool,
    pub load_timeout: u32,
    pub death_location: Option<BlockPos>,
    pub hurt_dir: f32,
    pub equipment: Vec<(ItemSlot, ItemId)>,
    pub experience: PlayerExperience,
    pub last_hurt_mob: Option<EntityId>,
    pub fire_ticks: i32,
    pub arrow_count: i32,
    pub remove_arrow_time: i32,
    sent: Option<SentVitals>,
}

impl Default for Vitals {
    fn default() -> Self {
        Self {
            health: 20.0,
            absorption: 0.0,
            food: FoodData::default(),
            fall_distance: 0.0,
            damage_cooldown: 0,
            last_hurt: 0.0,
            hurt_time: 0,
            dead: false,
            death_time: 0,
            removed: false,
            combat: CombatTracker::default(),
            kill_credit: None,
            attack_ticker: 0,
            held_item: None,
            velocity: Vec3::ZERO,
            sync_velocity: false,
            tick_count: 0,
            waiting_for_respawn: false,
            load_timeout: CLIENT_LOAD_TIMEOUT,
            death_location: None,
            hurt_dir: 0.0,
            equipment: Vec::new(),
            experience: PlayerExperience::default(),
            last_hurt_mob: None,
            fire_ticks: 0,
            arrow_count: 0,
            remove_arrow_time: 0,
            sent: None,
        }
    }
}

impl Vitals {
    pub fn is_dead(&self) -> bool {
        self.dead || self.health <= 0.0
    }

    pub fn client_loaded(&self) -> bool {
        !self.waiting_for_respawn && self.load_timeout == 0
    }

    pub(crate) fn mark_sent(&mut self) {
        self.sent = Some(self.snapshot());
        self.experience.last_sent = self.experience.total;
    }

    fn snapshot(&self) -> SentVitals {
        SentVitals {
            health: self.health,
            food: self.food.food,
            saturation_zero: self.food.saturation == 0.0,
            absorption: self.absorption,
        }
    }
}

pub(crate) fn equipment(inventory: &Inventory) -> Vec<(ItemSlot, ItemId)> {
    let main = HOTBAR_START + usize::from(inventory.selected_hotbar);
    [
        (ItemSlot::Mainhand, main),
        (ItemSlot::Offhand, OFFHAND_SLOT),
        (ItemSlot::Head, HEAD_SLOT),
        (ItemSlot::Chest, CHEST_SLOT),
        (ItemSlot::Legs, LEGS_SLOT),
        (ItemSlot::Feet, FEET_SLOT),
    ]
    .into_iter()
    .filter_map(|(slot, index)| inventory.get(index).map(|stack| (slot, stack.item)))
    .collect()
}

pub(crate) fn attributes<L>(player: &Player<L>) -> Attributes {
    Attributes::of(&player.effects, &equipment(&player.inventory))
}

pub(crate) fn max_health<L>(player: &Player<L>) -> f32 {
    attributes(player).get(MAX_HEALTH) as f32
}

fn effect(name: &str) -> Option<EffectId> {
    EffectId::from_name(name)
}

fn amplifier<L>(player: &Player<L>, name: &str) -> Option<u8> {
    effect(name).and_then(|id| player.effects.get(id).map(|e| e.amplifier))
}

fn invulnerable_to<L>(player: &Player<L>, kind: DamageType, fall_damage: bool) -> bool {
    (kind.is(DamageTag::IsFall) && !fall_damage) || !player.vitals.client_loaded()
}

fn heal<L>(player: &mut Player<L>, amount: f32) {
    let max = max_health(player);
    let vitals = &mut player.vitals;
    if vitals.health > 0.0 {
        vitals.health = java_clamp(vitals.health + amount, 0.0, max);
    }
}

fn actually_hurt<L>(
    player: &mut Player<L>,
    source: &DamageSource,
    amount: f32,
    attributes: &Attributes,
    now: u64,
) {
    let kind = source.kind;
    let mut amount = amount;
    if !kind.is(DamageTag::BypassesArmor) {
        amount = damage::after_armor(
            amount,
            attributes.get(ARMOR).floor() as f32,
            attributes.get(ARMOR_TOUGHNESS) as f32,
        );
    }
    if !kind.is(DamageTag::BypassesEffects) {
        if let Some(level) = amplifier(player, "minecraft:resistance")
            && !kind.is(DamageTag::BypassesResistance)
        {
            amount = damage::after_resistance(amount, level);
        }
        if amount <= 0.0 {
            amount = 0.0;
        }
    }
    let invulnerable = player.abilities().invulnerable;
    let vitals = &mut player.vitals;
    let absorbed = damage::absorb(amount, vitals.absorption);
    vitals.absorption = java_clamp(
        absorbed.absorption,
        0.0,
        attributes.get(MAX_ABSORPTION) as f32,
    );
    let amount = absorbed.damage;
    if amount == 0.0 {
        return;
    }
    if !invulnerable {
        vitals.food.add_exhaustion(kind.exhaustion());
    }
    let alive = vitals.health > 0.0;
    vitals.combat.record(
        CombatEntry {
            kind,
            causing: source.causing.as_ref().map(|c| c.name.clone()),
            direct: source.direct.as_ref().map(|d| d.name.clone()),
            living_cause: source.living_cause(),
            damage: amount,
            fall_distance: vitals.fall_distance as f32,
            location: None,
        },
        now,
        alive,
    );
    vitals.health = java_clamp(
        vitals.health - amount,
        0.0,
        attributes.get(MAX_HEALTH) as f32,
    );
}

impl<L: ClientLink> Server<L> {
    pub(crate) fn source_of(&self, id: PlayerId) -> Option<SourceEntity> {
        self.players.get(&id).map(|p| SourceEntity {
            entity: p.entity_id,
            player: Some(id),
            kind: Kind::Player,
            creative: p.game_mode == GameMode::Creative,
            name: Component::text(p.name()),
            position: p.position,
        })
    }

    pub(crate) fn hurt(&mut self, id: PlayerId, source: &DamageSource, amount: f32) -> bool {
        let pvp = self.config.pvp;
        let fall_damage = self.config.rules.fall_damage;
        let difficulty = self.world.difficulty;
        let now = self.current_tick();
        let kind = source.kind;
        let Some(player) = self.players.get_mut(&id) else {
            return false;
        };
        if invulnerable_to(player, kind, fall_damage) {
            return false;
        }
        if source.attacker().is_some() && !pvp {
            return false;
        }
        if player.abilities().invulnerable && !kind.is(DamageTag::BypassesInvulnerability) {
            return false;
        }
        if player.vitals.is_dead() {
            return false;
        }
        let amount = if source.scales_with_difficulty() {
            damage::scaled_for_difficulty(amount, difficulty)
        } else {
            amount
        };
        if amount == 0.0 {
            return false;
        }
        if kind.is(DamageTag::IsFire) && player.effects.has("minecraft:fire_resistance") {
            return false;
        }
        let mut amount = if amount < 0.0 { 0.0 } else { amount };
        if kind.is(DamageTag::DamagesHelmet) && player.inventory.get(HEAD_SLOT).is_some() {
            amount *= HELMET_FACTOR;
        }
        if !amount.is_finite() {
            amount = f32::MAX;
        }
        let attributes = attributes(player);
        let full = if player.vitals.damage_cooldown > COOLDOWN_THRESHOLD
            && !kind.is(DamageTag::BypassesCooldown)
        {
            if amount <= player.vitals.last_hurt {
                return false;
            }
            let extra = amount - player.vitals.last_hurt;
            actually_hurt(player, source, extra, &attributes, now);
            player.vitals.last_hurt = amount;
            false
        } else {
            player.vitals.last_hurt = amount;
            player.vitals.damage_cooldown = DAMAGE_COOLDOWN;
            actually_hurt(player, source, amount, &attributes, now);
            player.vitals.hurt_time = HURT_DURATION;
            true
        };
        if let Some(cause) = &source.causing
            && let Some(attacker) = cause.player
        {
            player.vitals.kill_credit = Some(KillCredit {
                player: attacker,
                name: cause.name.clone(),
                memory: KILL_CREDIT_TICKS,
            });
        }
        if let Some(cause) = &source.causing
            && cause.player.is_none()
            && cause.kind.living()
        {
            player.vitals.last_hurt_mob = Some(cause.entity);
        }
        if let Some(attacker) = source.attacker()
            && let Some(other) = self.players.get_mut(&attacker)
        {
            other.vitals.attack_ticker = 0;
        }
        let Some(player) = self.players.get(&id) else {
            return false;
        };
        let entity = player.entity_id;
        let position = player.position;
        let dead = player.vitals.health <= 0.0;
        if full {
            let event = ClientEvent::DamageEvent {
                entity,
                kind,
                cause: source.causing.as_ref().map(|c| c.entity),
                direct: source.direct.as_ref().map(|d| d.entity),
                position: source.position,
            };
            self.push(Audience::One(id), event.clone());
            self.push(Audience::Trackers(entity), event);
            if !kind.is(DamageTag::NoKnockback) {
                let (dx, dz) = source.knockback_from(position);
                self.knockback(id, crate::combat::DEFAULT_KNOCKBACK, dx, dz);
                self.indicate_damage(id, dx, dz);
            }
        }
        if dead {
            if full {
                self.play_player_sound(id, Kind::Player.sounds().map(|s| s.death));
            }
            self.die(id);
        } else if full {
            self.play_player_sound(id, Kind::Player.sounds().map(|s| s.hurt));
        }
        true
    }

    fn play_player_sound(&mut self, id: PlayerId, sound: Option<rspigot_entities::CreatureSound>) {
        let Some(player) = self.players.get(&id) else {
            return;
        };
        let Some(sound) = sound.and_then(|s| s.get(rspigot_entities::SoundState::default())) else {
            return;
        };
        let position = player.position;
        let Some(sounds) = Kind::Player.sounds() else {
            return;
        };
        let pitch = self.voice_pitch(sounds.pitch, false);
        let volume = sounds.volume.get(false, 1);
        self.play_sound(Some(id), position, sound, sounds.source, volume, pitch);
    }

    pub(crate) fn heal_player(&mut self, id: PlayerId, amount: f32) {
        if let Some(player) = self.players.get_mut(&id) {
            heal(player, amount);
        }
    }

    pub(crate) fn exhaust(&mut self, id: PlayerId, amount: f32) {
        if let Some(player) = self.players.get_mut(&id)
            && !player.abilities().invulnerable
        {
            player.vitals.food.add_exhaustion(amount);
        }
    }

    pub(crate) fn kill(&mut self, id: PlayerId) -> bool {
        self.hurt(id, &DamageSource::new(DamageType::GenericKill), f32::MAX)
    }

    fn die(&mut self, id: PlayerId) {
        self.close_open_menu(id, true);
        self.decisions.forget(id);
        let show = self.config.rules.show_death_messages;
        let keep = self.config.rules.keep_inventory;
        let now = self.current_tick();
        let Some(player) = self.players.get_mut(&id) else {
            return;
        };
        if player.vitals.dead {
            return;
        }
        let vitals = &mut player.vitals;
        vitals.dead = true;
        vitals.death_time = 0;
        vitals.waiting_for_respawn = true;
        vitals.death_location = Some(player.position.block());
        let victim = Component::text(player.profile.name.clone());
        let message = if show {
            let credit = vitals.kill_credit.as_ref().map(|c| &c.name);
            vitals.combat.death_message(&victim, credit)
        } else {
            Component::text("")
        };
        vitals.combat.recheck(now, false);
        let entity = player.entity_id;
        let origin = Vec3::new(
            player.position.x,
            player.position.y + player.eye_height() - DROP_EYE_OFFSET,
            player.position.z,
        );
        let spectator = player.game_mode == GameMode::Spectator;
        let experience = if keep || spectator {
            0
        } else {
            player.vitals.experience.death_reward()
        };
        let feet = player.position;
        let drops: Vec<ItemStack> = if keep || spectator {
            Vec::new()
        } else {
            (0..PLAYER_INVENTORY_SIZE)
                .filter_map(|slot| {
                    let stack = player.inventory.get(slot).cloned();
                    player.inventory.set(slot, None);
                    stack
                })
                .collect()
        };
        self.push(
            Audience::One(id),
            ClientEvent::CombatKill {
                player: entity,
                message: message.clone(),
            },
        );
        if show {
            tracing::info!("{}", crate::console::console_text(&message));
            self.push(Audience::All, ClientEvent::system(message));
        }
        if !drops.is_empty() {
            for stack in drops {
                self.drop_randomly(origin, stack);
            }
            self.send_inventory(id);
            self.broadcast_held_item(id);
        }
        self.award_experience(feet, experience);
        let status = ClientEvent::EntityStatus {
            entity,
            status: EntityStatus::Death,
        };
        self.push(Audience::One(id), status.clone());
        self.push(Audience::Trackers(entity), status);
    }

    fn drop_randomly(&mut self, origin: Vec3, stack: ItemStack) {
        let speed = self.rng.random::<f32>() * DROP_SPEED;
        let angle = self.rng.random::<f32>() * std::f32::consts::TAU;
        let velocity = Vec3::new(
            f64::from(-angle.sin() * speed),
            DROP_LIFT,
            f64::from(angle.cos() * speed),
        );
        self.spawn_item_entity(origin, velocity, DROP_PICKUP_DELAY, stack);
    }

    pub(crate) fn respawn(&mut self, id: PlayerId) {
        let hardcore = self.config.hardcore;
        let keep_experience = self.config.rules.keep_inventory;
        let spawn = self.world.spawn;
        let angle = self.world.level.spawn_angle;
        let dimension = self.world.dimension.key.clone();
        let Some(player) = self.players.get_mut(&id) else {
            return;
        };
        if !player.vitals.is_dead() {
            return;
        }
        let death_location = player.vitals.death_location;
        let mut experience = PlayerExperience::default();
        if keep_experience {
            let kept = player.vitals.experience;
            experience.level = kept.level;
            experience.progress = kept.progress;
            experience.total = kept.total;
        }
        experience.last_sent = -1;
        player.vitals = Vitals {
            death_location,
            experience,
            ..Vitals::default()
        };
        let _ = player.effects.clear();
        player.sneaking = false;
        player.sprinting = false;
        player.loaded = false;
        player.rotation = Rotation::new(angle, 0.0);
        let entity = player.entity_id;
        for other in self.players.values_mut() {
            if other.tracked.remove(&entity) {
                other.link.send(ClientEvent::RemoveEntities(vec![entity]));
            }
        }
        let Some(player) = self.players.get(&id) else {
            return;
        };
        let mut spawn_info = self.spawn_info(player);
        spawn_info.death_location = death_location.map(|pos| (dimension.clone(), pos));
        player.link.send(ClientEvent::Respawn {
            spawn: spawn_info,
            keep_data: 0,
        });
        crate::commands::CommandContext::teleport(self, id, spawn);
        let time = self.time_event();
        let difficulty = self.world.difficulty;
        let raining = self.world.level.raining;
        let thundering = self.world.level.thundering;
        let tree = self.command_tree(self.players.get(&id).is_some_and(|p| p.operator));
        let Some(player) = self.players.get_mut(&id) else {
            return;
        };
        let link = &player.link;
        link.send(ClientEvent::SpawnPosition {
            dimension,
            pos: spawn.block(),
            yaw: angle,
            pitch: 0.0,
        });
        link.send(ClientEvent::Difficulty {
            difficulty,
            locked: false,
        });
        let experience = player.vitals.experience;
        link.send(ClientEvent::Experience {
            progress: experience.progress,
            level: experience.level,
            total: experience.total,
        });
        link.send(ClientEvent::OperatorLevel {
            entity,
            level: if player.operator { 4 } else { 0 },
        });
        link.send(ClientEvent::Commands(tree));
        link.send(time);
        if raining {
            link.send(ClientEvent::GameEvent(GameEvent::StartRaining));
            link.send(ClientEvent::GameEvent(GameEvent::RainLevelChange(1.0)));
            link.send(ClientEvent::GameEvent(GameEvent::ThunderLevelChange(
                if thundering { 1.0 } else { 0.0 },
            )));
        }
        link.send(ClientEvent::GameEvent(GameEvent::LevelChunksLoadStart));
        link.send(ClientEvent::Abilities(player.abilities()));
        link.send(ClientEvent::HeldSlot(player.inventory.selected_hotbar));
        let state_id = player.next_inventory_state();
        player.link.send(ClientEvent::InventoryContent {
            state_id,
            slots: player.inventory.slots().to_vec(),
            carried: None,
        });
        if hardcore {
            self.apply_game_mode(id, GameMode::Spectator);
        }
    }

    pub(crate) fn tick_health(&mut self) {
        let now = self.current_tick();
        let floor = f64::from(self.world.dimension.height.min_y - VOID_DEPTH);
        let ids: Vec<PlayerId> = self.players.keys().copied().collect();
        for &id in &ids {
            let Some(player) = self.players.get_mut(&id) else {
                continue;
            };
            let vitals = &mut player.vitals;
            vitals.tick_count = vitals.tick_count.wrapping_add(1);
            vitals.damage_cooldown = vitals.damage_cooldown.saturating_sub(1);
            vitals.load_timeout = if player.loaded {
                0
            } else {
                vitals.load_timeout.saturating_sub(1)
            };
            if vitals.is_dead() {
                vitals.death_time += 1;
                if vitals.death_time == DEATH_ANIMATION_TICKS && !vitals.removed {
                    vitals.removed = true;
                    let entity = player.entity_id;
                    self.remove_body(entity);
                }
                continue;
            }
            vitals.hurt_time = vitals.hurt_time.saturating_sub(1);
            if let Some(credit) = &mut vitals.kill_credit {
                credit.memory = credit.memory.saturating_sub(1);
                if credit.memory == 0 {
                    vitals.kill_credit = None;
                }
            }
            vitals.combat.recheck(now, true);
            if vitals.arrow_count > 0 {
                if vitals.remove_arrow_time <= 0 {
                    vitals.remove_arrow_time = 20 * (30 - vitals.arrow_count);
                }
                vitals.remove_arrow_time -= 1;
                if vitals.remove_arrow_time <= 0 {
                    vitals.arrow_count -= 1;
                    self.broadcast_metadata(id);
                }
            }
            self.tick_player_fire(id);
            if self.players.get(&id).is_some_and(|p| p.position.y < floor) {
                self.hurt(id, &DamageSource::new(DamageType::OutOfWorld), VOID_DAMAGE);
            }
        }
        self.tick_effects();
        let difficulty = self.world.difficulty;
        let regeneration = self.config.rules.natural_regeneration;
        let mut changed_equipment = Vec::new();
        for &id in &ids {
            let Some(player) = self.players.get_mut(&id) else {
                continue;
            };
            if player.vitals.is_dead() {
                continue;
            }
            let max = max_health(player);
            let resets_fall = player.abilities().flying
                || player.effects.has("minecraft:slow_falling")
                || player.effects.has("minecraft:levitation");
            let held = player.inventory.held().map(|s| s.item);
            let worn = equipment(&player.inventory);
            if worn != player.vitals.equipment {
                let names: Vec<&'static str> = worn
                    .iter()
                    .chain(player.vitals.equipment.iter())
                    .flat_map(|(slot, item)| {
                        equipment_modifiers(*slot, *item).map(|(name, _)| name)
                    })
                    .filter(|name| is_syncable(name))
                    .collect();
                player.vitals.equipment = worn;
                changed_equipment.push((id, names));
            }
            let vitals = &mut player.vitals;
            if resets_fall {
                vitals.fall_distance = 0.0;
            }
            if difficulty == Difficulty::Peaceful && regeneration {
                if vitals.tick_count.is_multiple_of(PEACEFUL_HEAL_INTERVAL) {
                    if vitals.health < max {
                        vitals.health = java_clamp(vitals.health + 1.0, 0.0, max);
                    }
                    if vitals.food.saturation < rspigot_domain::food::MAX_SATURATION {
                        vitals.food.saturation += 1.0;
                    }
                }
                if vitals.tick_count.is_multiple_of(PEACEFUL_FOOD_INTERVAL)
                    && vitals.food.needs_food()
                {
                    vitals.food.food += 1;
                }
            }
            vitals.attack_ticker = vitals.attack_ticker.saturating_add(1);
            if held != vitals.held_item {
                vitals.attack_ticker = 0;
                vitals.held_item = held;
            }
            match vitals
                .food
                .tick(difficulty, regeneration, vitals.health, max)
            {
                FoodEffect::None => {}
                FoodEffect::Heal { amount, exhaustion } => {
                    heal(player, amount);
                    self.exhaust(id, exhaustion);
                }
                FoodEffect::Starve => {
                    self.hurt(id, &DamageSource::new(DamageType::Starve), 1.0);
                }
            }
        }
        for (id, names) in changed_equipment {
            self.sync_attributes(id, &names);
            self.clamp_vitals(id);
        }
    }

    fn remove_body(&mut self, entity: EntityId) {
        let poof = ClientEvent::EntityStatus {
            entity,
            status: EntityStatus::Poof,
        };
        for viewer in self.players.values_mut() {
            if viewer.tracked.remove(&entity) {
                viewer.link.send(poof.clone());
                viewer.link.send(ClientEvent::RemoveEntities(vec![entity]));
            }
        }
    }

    pub(crate) fn clamp_vitals(&mut self, id: PlayerId) {
        let Some(player) = self.players.get_mut(&id) else {
            return;
        };
        let attributes = attributes(player);
        let max_health = attributes.get(MAX_HEALTH) as f32;
        let max_absorption = attributes.get(MAX_ABSORPTION) as f32;
        let vitals = &mut player.vitals;
        if vitals.health > max_health {
            vitals.health = max_health;
        }
        if vitals.absorption > max_absorption {
            vitals.absorption = max_absorption;
        }
    }

    pub(crate) fn start_absorption(&mut self, id: PlayerId, amplifier: u8) {
        let Some(player) = self.players.get_mut(&id) else {
            return;
        };
        let max = attributes(player).get(MAX_ABSORPTION) as f32;
        let wanted = ABSORPTION_PER_LEVEL * (f32::from(amplifier) + 1.0);
        let vitals = &mut player.vitals;
        vitals.absorption = java_clamp(vitals.absorption.max(wanted), 0.0, max);
    }

    pub(crate) fn health_metadata(player: &Player<L>) -> Vec<MetadataEntry> {
        let mut entries = vec![MetadataEntry {
            index: HEALTH_DATA_INDEX,
            value: MetadataValue::Float(player.vitals.health),
        }];
        if player.vitals.absorption > 0.0 {
            entries.push(MetadataEntry {
                index: ABSORPTION_DATA_INDEX,
                value: MetadataValue::Float(player.vitals.absorption),
            });
        }
        entries
    }

    pub(crate) fn sync_vitals(&mut self) {
        let mut updates = Vec::new();
        for player in self.players.values_mut() {
            let current = player.vitals.snapshot();
            let previous = player.vitals.sent;
            let entity = player.entity_id;
            let changed = previous.is_none_or(|sent| {
                sent.health != current.health
                    || sent.food != current.food
                    || sent.saturation_zero != current.saturation_zero
            });
            if changed {
                updates.push((
                    Audience::One(player.id),
                    ClientEvent::Health {
                        health: current.health,
                        food: current.food,
                        saturation: player.vitals.food.saturation,
                    },
                ));
            }
            let mut entries = Vec::new();
            if previous.is_none_or(|sent| sent.health != current.health) {
                entries.push(MetadataEntry {
                    index: HEALTH_DATA_INDEX,
                    value: MetadataValue::Float(current.health),
                });
            }
            if previous.is_some_and(|sent| sent.absorption != current.absorption) {
                entries.push(MetadataEntry {
                    index: ABSORPTION_DATA_INDEX,
                    value: MetadataValue::Float(current.absorption),
                });
            }
            if !entries.is_empty() {
                let event = ClientEvent::EntityMetadata { entity, entries };
                updates.push((Audience::One(player.id), event.clone()));
                updates.push((Audience::Trackers(entity), event));
            }
            if let Some((progress, level, total)) = player.vitals.experience.take_unsent() {
                updates.push((
                    Audience::One(player.id),
                    ClientEvent::Experience {
                        progress,
                        level,
                        total,
                    },
                ));
            }
            if player.vitals.sync_velocity {
                let event = ClientEvent::EntityVelocity {
                    entity,
                    velocity: player.vitals.velocity,
                };
                updates.push((Audience::One(player.id), event.clone()));
                updates.push((Audience::Trackers(entity), event));
                player.vitals.sync_velocity = false;
                player.vitals.velocity = Vec3::ZERO;
            }
            player.vitals.sent = Some(current);
        }
        for (audience, event) in updates {
            self.push(audience, event);
        }
    }

    pub fn player_health(&self, id: PlayerId) -> Option<f32> {
        self.players.get(&id).map(|p| p.vitals.health)
    }

    pub fn player_food(&self, id: PlayerId) -> Option<(i32, f32, f32)> {
        self.players.get(&id).map(|p| {
            (
                p.vitals.food.food,
                p.vitals.food.saturation,
                p.vitals.food.exhaustion,
            )
        })
    }

    pub fn player_absorption(&self, id: PlayerId) -> Option<f32> {
        self.players.get(&id).map(|p| p.vitals.absorption)
    }

    pub fn player_location(&self, id: PlayerId) -> Option<Vec3> {
        self.players.get(&id).map(|p| p.position)
    }

    pub fn player_is_dead(&self, id: PlayerId) -> Option<bool> {
        self.players.get(&id).map(|p| p.vitals.is_dead())
    }
}
