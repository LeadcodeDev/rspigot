use std::borrow::Cow;

use rspigot_entities::EntityKind;
use rspigot_items::EquipmentSlot;

use super::base::EntityBase;
use super::data::{Pose, SyncedValue};
use super::physics::{
    self, BlockView, Landing, MoveTraits, apply_inside_blocks, friction, has_tag, move_entity,
    movement_affecting_pos,
};
use crate::attribute;
use crate::effect::{
    ActiveEffects, AttributeModifier, AttributeOperation, AttributeState, EffectId, EffectInstance,
};
use crate::item::ItemStack;
use crate::{BlockPos, Vec3};

pub const DATA_LIVING_FLAGS: u8 = 8;
pub const DATA_HEALTH: u8 = 9;
pub const DATA_EFFECT_AMBIENCE: u8 = 11;
pub const DATA_ARROW_COUNT: u8 = 12;
pub const DATA_STINGER_COUNT: u8 = 13;
pub const DATA_MOB_FLAGS: u8 = 15;
pub const MOB_FLAG_NO_AI: u8 = 1;
pub const MOB_FLAG_LEFT_HANDED: u8 = 2;
pub const DEATH_DURATION: i32 = 20;
pub const HURT_DURATION: i32 = 10;
pub const DEFAULT_DROP_CHANCE: f32 = 0.085;
pub const RANDOM_SPAWN_BONUS: &str = "minecraft:random_spawn_bonus";
pub const MAX_HEALTH: &str = "minecraft:max_health";
pub const FOLLOW_RANGE: &str = "minecraft:follow_range";
pub const GRAVITY: &str = "minecraft:gravity";
pub const STEP_HEIGHT: &str = "minecraft:step_height";
pub const FRICTION_MODIFIER: &str = "minecraft:friction_modifier";
pub const AIR_DRAG_MODIFIER: &str = "minecraft:air_drag_modifier";
pub const BOUNCINESS: &str = "minecraft:bounciness";
pub const MOVEMENT_EFFICIENCY: &str = "minecraft:movement_efficiency";
const VELOCITY_THRESHOLD: f64 = 0.003;
const SPAWN_BONUS_DEVIATION: f64 = 0.114_850_000_000_000_01;
const LEFT_HANDED_CHANCE: f32 = 0.05;
const AIR_FRICTION: f32 = 0.91;
const AIR_DRAG: f32 = 0.98;
const FLYING_SPEED: f32 = 0.02;
const SLOW_FALLING_GRAVITY: f64 = 0.01;
const LEVITATION_SPEED: f64 = 0.05;
const LEVITATION_BLEND: f64 = 0.2;
const CLIMB_SPEED: f64 = 0.15000000596046448;
const CLIMB_BOOST: f64 = 0.2;
const APPLIED_INPUT_DECAY: f32 = 0.98;

const SYNCABLE: [&str; 32] = [
    "minecraft:air_drag_modifier",
    "minecraft:armor",
    "minecraft:armor_toughness",
    "minecraft:attack_speed",
    "minecraft:below_name_distance",
    "minecraft:block_break_speed",
    "minecraft:block_interaction_range",
    "minecraft:bounciness",
    "minecraft:burning_time",
    "minecraft:camera_distance",
    "minecraft:explosion_knockback_resistance",
    "minecraft:entity_interaction_range",
    "minecraft:fall_damage_multiplier",
    "minecraft:flying_speed",
    "minecraft:friction_modifier",
    "minecraft:gravity",
    "minecraft:jump_strength",
    "minecraft:luck",
    "minecraft:max_absorption",
    "minecraft:max_health",
    "minecraft:mining_efficiency",
    "minecraft:movement_efficiency",
    "minecraft:movement_speed",
    "minecraft:name_tag_distance",
    "minecraft:oxygen_bonus",
    "minecraft:safe_fall_distance",
    "minecraft:scale",
    "minecraft:sneaking_speed",
    "minecraft:step_height",
    "minecraft:submerged_mining_speed",
    "minecraft:sweeping_damage_ratio",
    "minecraft:water_movement_efficiency",
];

pub fn is_client_syncable(attribute: &str) -> bool {
    SYNCABLE.contains(&attribute)
}

pub fn compute_modified_friction(friction: f32, modifier: f32) -> f32 {
    (1.0 - (1.0 - friction) * modifier).clamp(0.0, 1.0)
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttributeInstance {
    pub attribute: &'static str,
    pub base: f64,
    pub modifiers: Vec<AttributeModifier>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LivingAttributes {
    kind: EntityKind,
    instances: Vec<AttributeInstance>,
    to_sync: Vec<&'static str>,
}

impl LivingAttributes {
    pub fn new(kind: EntityKind) -> Self {
        Self {
            kind,
            instances: Vec::new(),
            to_sync: Vec::new(),
        }
    }

    fn supplier(&self, name: &str) -> Option<(&'static str, f64)> {
        self.kind
            .default_attributes()
            .iter()
            .find(|(candidate, _)| *candidate == name)
            .copied()
    }

    pub fn instance(&mut self, name: &str) -> Option<&mut AttributeInstance> {
        if let Some(index) = self.instances.iter().position(|i| i.attribute == name) {
            return self.instances.get_mut(index);
        }
        let (attribute, base) = self.supplier(name)?;
        self.instances.push(AttributeInstance {
            attribute,
            base,
            modifiers: Vec::new(),
        });
        self.instances.last_mut()
    }

    pub fn instances(&self) -> &[AttributeInstance] {
        &self.instances
    }

    pub fn state(&self, name: &str, effects: &ActiveEffects) -> Option<AttributeState> {
        let instance = self.instances.iter().find(|i| i.attribute == name)?;
        let mut modifiers = instance.modifiers.clone();
        modifiers.extend(effect_modifiers(effects, instance.attribute));
        Some(AttributeState {
            attribute: instance.attribute,
            base: instance.base,
            modifiers,
        })
    }

    pub fn value(&mut self, name: &str, effects: &ActiveEffects) -> f64 {
        if self.instance(name).is_none() {
            return 0.0;
        }
        self.state(name, effects)
            .map_or(0.0, |state| attribute::value(&state))
    }

    pub fn set_base(&mut self, name: &str, base: f64) -> bool {
        let Some(instance) = self.instance(name) else {
            return false;
        };
        if instance.base == base {
            return true;
        }
        instance.base = base;
        let attribute = instance.attribute;
        self.mark_dirty(attribute);
        true
    }

    pub fn add_modifier(&mut self, name: &str, modifier: AttributeModifier) -> bool {
        let Some(instance) = self.instance(name) else {
            return false;
        };
        if instance.modifiers.iter().any(|m| m.id == modifier.id) {
            return false;
        }
        instance.modifiers.push(modifier);
        let attribute = instance.attribute;
        self.mark_dirty(attribute);
        true
    }

    pub fn has_modifier(&self, name: &str, id: &str) -> bool {
        self.instances
            .iter()
            .any(|i| i.attribute == name && i.modifiers.iter().any(|m| m.id == id))
    }

    pub fn mark_dirty(&mut self, attribute: &'static str) {
        if is_client_syncable(attribute) && !self.to_sync.contains(&attribute) {
            self.to_sync.push(attribute);
        }
    }

    pub fn take_dirty(&mut self) -> Vec<&'static str> {
        std::mem::take(&mut self.to_sync)
    }

    pub fn syncable(&self, effects: &ActiveEffects) -> Vec<AttributeState> {
        self.instances
            .iter()
            .filter(|i| is_client_syncable(i.attribute))
            .filter_map(|i| self.state(i.attribute, effects))
            .collect()
    }
}

fn effect_modifiers(
    effects: &ActiveEffects,
    attribute: &str,
) -> impl Iterator<Item = AttributeModifier> {
    let found: Vec<AttributeModifier> = effects
        .iter()
        .filter_map(|e| Some((e.amplifier, e.effect.kind()?.modifiers())))
        .flat_map(|(amplifier, templates)| templates.iter().map(move |t| (amplifier, t)))
        .filter(|(_, t)| t.attribute == attribute)
        .map(|(amplifier, t)| AttributeModifier {
            id: Cow::Borrowed(t.modifier),
            amount: t.amount_for(amplifier),
            operation: t.operation,
        })
        .collect();
    found.into_iter()
}

#[derive(Debug, Clone, PartialEq)]
pub struct MobState {
    pub can_pick_up_loot: bool,
    pub persistence_required: bool,
    pub drop_chances: [f32; 8],
    pub death_loot_table: Option<String>,
    pub death_loot_table_seed: i64,
    pub aware: bool,
}

impl Default for MobState {
    fn default() -> Self {
        Self {
            can_pick_up_loot: false,
            persistence_required: false,
            drop_chances: [DEFAULT_DROP_CHANCE; 8],
            death_loot_table: None,
            death_loot_table_seed: 0,
            aware: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LivingState {
    pub absorption: f32,
    pub hurt_time: i32,
    pub death_time: i32,
    pub dead: bool,
    pub body_yaw: f32,
    pub attributes: LivingAttributes,
    pub effects: ActiveEffects,
    pub equipment: [Option<ItemStack>; 8],
    pub fall_flying: bool,
    pub impulse_grace: i32,
    pub speed: f32,
    pub jumping: bool,
    pub xxa: f32,
    pub yya: f32,
    pub zza: f32,
    pub no_jump_delay: i32,
    pub last_hurt: f32,
    pub mob: Option<MobState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HurtOutcome {
    Ignored,
    Hurt,
    Died,
}

pub fn slot_index(slot: EquipmentSlot) -> usize {
    slot as usize
}

pub const EQUIPMENT_SLOTS: [EquipmentSlot; 8] = [
    EquipmentSlot::Mainhand,
    EquipmentSlot::Offhand,
    EquipmentSlot::Feet,
    EquipmentSlot::Legs,
    EquipmentSlot::Chest,
    EquipmentSlot::Head,
    EquipmentSlot::Body,
    EquipmentSlot::Saddle,
];

impl LivingState {
    pub fn new(base: &mut EntityBase, initial_yaw: f32) -> Self {
        let kind = base.kind;
        base.data.define(DATA_LIVING_FLAGS, SyncedValue::Byte(0));
        base.data.define(DATA_HEALTH, SyncedValue::Float(1.0));
        base.data
            .define(DATA_EFFECT_AMBIENCE, SyncedValue::Boolean(false));
        base.data.define(DATA_ARROW_COUNT, SyncedValue::Int(0));
        base.data.define(DATA_STINGER_COUNT, SyncedValue::Int(0));
        let is_mob = kind.extends("Mob");
        if is_mob {
            base.data.define(DATA_MOB_FLAGS, SyncedValue::Byte(0));
        }
        let mut state = Self {
            absorption: 0.0,
            hurt_time: 0,
            death_time: 0,
            dead: false,
            body_yaw: 0.0,
            attributes: LivingAttributes::new(kind),
            effects: ActiveEffects::default(),
            equipment: Default::default(),
            fall_flying: false,
            impulse_grace: 0,
            speed: 0.0,
            jumping: false,
            xxa: 0.0,
            yya: 0.0,
            zza: 0.0,
            no_jump_delay: 0,
            last_hurt: 0.0,
            mob: is_mob.then(MobState::default),
        };
        let max = state.max_health() as f32;
        base.data.set(DATA_HEALTH, SyncedValue::Float(max));
        state.attributes.take_dirty();
        base.set_yaw(initial_yaw);
        base.head_yaw = base.yaw;
        if is_mob {
            let _ = state.attributes.instance(FOLLOW_RANGE);
        }
        state
    }

    pub fn attribute(&mut self, name: &str) -> f64 {
        self.attributes.value(name, &self.effects)
    }

    pub fn max_health(&mut self) -> f64 {
        self.attribute(MAX_HEALTH)
    }

    pub fn is_mob(&self) -> bool {
        self.mob.is_some()
    }

    pub fn equipment(&self, slot: EquipmentSlot) -> Option<&ItemStack> {
        self.equipment[slot_index(slot)].as_ref()
    }

    pub fn has_effect(&self, name: &str) -> bool {
        self.effects.has(name)
    }

    pub fn effect(&self, name: &str) -> Option<&EffectInstance> {
        EffectId::from_name(name).and_then(|id| self.effects.get(id))
    }

    pub fn effective_gravity(&mut self, base: &EntityBase) -> f64 {
        let gravity = if base.is_no_gravity() {
            0.0
        } else {
            self.attribute(GRAVITY)
        };
        if base.velocity.y > 0.0 || !self.has_effect("minecraft:slow_falling") {
            gravity
        } else {
            gravity.min(SLOW_FALLING_GRAVITY)
        }
    }

    pub fn move_traits(&mut self, base: &EntityBase) -> MoveTraits {
        let air_drag =
            compute_modified_friction(AIR_DRAG, self.attribute(AIR_DRAG_MODIFIER) as f32);
        MoveTraits {
            max_up_step: self.attribute(STEP_HEIGHT) as f32,
            living: true,
            bounciness: self.attribute(BOUNCINESS),
            suppress_bounce: false,
            gravity: self.effective_gravity(base),
            air_drag,
            movement_efficiency: Some(self.attribute(MOVEMENT_EFFICIENCY) as f32),
        }
    }
}

pub fn health(base: &EntityBase) -> f32 {
    base.data.float(DATA_HEALTH)
}

pub fn set_health(base: &mut EntityBase, living: &mut LivingState, health: f32) {
    let max = living.max_health() as f32;
    base.data
        .set(DATA_HEALTH, SyncedValue::Float(health.clamp(0.0, max)));
}

pub fn is_no_ai(base: &EntityBase) -> bool {
    base.data.flag(DATA_MOB_FLAGS, MOB_FLAG_NO_AI)
}

pub fn set_no_ai(base: &mut EntityBase, no_ai: bool) {
    base.data.set_flag(DATA_MOB_FLAGS, MOB_FLAG_NO_AI, no_ai);
}

pub fn is_left_handed(base: &EntityBase) -> bool {
    base.data.flag(DATA_MOB_FLAGS, MOB_FLAG_LEFT_HANDED)
}

pub fn set_left_handed(base: &mut EntityBase, left: bool) {
    base.data
        .set_flag(DATA_MOB_FLAGS, MOB_FLAG_LEFT_HANDED, left);
}

pub fn finalize_spawn(
    base: &mut EntityBase,
    living: &mut LivingState,
    next_double: &mut impl FnMut() -> f64,
    next_float: &mut impl FnMut() -> f32,
) {
    if living.mob.is_none() {
        return;
    }
    if !living
        .attributes
        .has_modifier(FOLLOW_RANGE, RANDOM_SPAWN_BONUS)
    {
        let bonus = SPAWN_BONUS_DEVIATION * (next_double() - next_double());
        living.attributes.add_modifier(
            FOLLOW_RANGE,
            AttributeModifier {
                id: Cow::Borrowed(RANDOM_SPAWN_BONUS),
                amount: bonus,
                operation: AttributeOperation::AddMultipliedBase,
            },
        );
    }
    set_left_handed(base, next_float() < LEFT_HANDED_CHANCE);
}

pub fn hurt(
    base: &mut EntityBase,
    living: &mut LivingState,
    amount: f32,
    bypasses_invulnerability: bool,
) -> HurtOutcome {
    if base.is_removed() || living.dead || health(base) <= 0.0 {
        return HurtOutcome::Ignored;
    }
    if base.invulnerable && !bypasses_invulnerability {
        return HurtOutcome::Ignored;
    }
    let amount = if amount.is_finite() {
        amount.max(0.0)
    } else {
        f32::MAX
    };
    living.last_hurt = amount;
    living.hurt_time = HURT_DURATION;
    let current = health(base);
    set_health(base, living, current - amount);
    if health(base) <= 0.0 {
        die(base, living);
        HurtOutcome::Died
    } else {
        HurtOutcome::Hurt
    }
}

pub fn die(base: &mut EntityBase, living: &mut LivingState) {
    if base.is_removed() || living.dead {
        return;
    }
    living.dead = true;
    base.set_pose(Pose::Dying);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LivingTick {
    pub died: bool,
    pub removed: bool,
    pub expired_effects: bool,
    pub below_world: bool,
}

pub fn base_tick(base: &mut EntityBase, living: &mut LivingState) -> LivingTick {
    let mut outcome = LivingTick::default();
    if base.remaining_fire_ticks > 0 {
        if base.kind.fire_immune() {
            base.remaining_fire_ticks = 0;
        } else {
            base.remaining_fire_ticks -= 1;
        }
    }
    base.set_shared_flag(super::base::FLAG_ON_FIRE, base.remaining_fire_ticks > 0);
    let air = base.air_supply();
    let max_air = base.max_air_ticks;
    if air < max_air {
        base.set_air_supply((air + 4).min(max_air));
    }
    if living.hurt_time > 0 {
        living.hurt_time -= 1;
    }
    if base.invulnerable_time > 0 {
        base.invulnerable_time -= 1;
    }
    if health(base) <= 0.0 {
        living.death_time += 1;
        if living.death_time >= DEATH_DURATION && !base.is_removed() {
            outcome.removed = true;
        }
    }
    let expired = living.effects.tick();
    for effect in &expired {
        mark_effect_attributes(living, *effect);
    }
    outcome.expired_effects = !expired.is_empty();
    outcome
}

pub fn mark_effect_attributes(living: &mut LivingState, effect: EffectId) {
    let Some(kind) = effect.kind() else {
        return;
    };
    for template in kind.modifiers() {
        if let Some(instance) = living.attributes.instance(template.attribute) {
            let attribute = instance.attribute;
            living.attributes.mark_dirty(attribute);
        }
    }
}

pub fn on_climbable(base: &EntityBase, view: &impl BlockView) -> bool {
    has_tag(
        view.block_or_air(base.position.block()),
        "minecraft:climbable",
    )
}

fn input_vector(input: Vec3, speed: f32, yaw: f32) -> Vec3 {
    let length = input.length_sqr();
    if length < 1.0e-7 {
        return Vec3::ZERO;
    }
    let scaled = if length > 1.0 {
        input.normalize()
    } else {
        input
    }
    .scale(f64::from(speed));
    let radians = yaw * 0.017_453_292;
    let sin = f64::from(radians.sin());
    let cos = f64::from(radians.cos());
    Vec3::new(
        scaled.x * cos - scaled.z * sin,
        scaled.y,
        scaled.z * cos + scaled.x * sin,
    )
}

fn friction_influenced_speed(base: &EntityBase, living: &LivingState, friction: f32) -> f32 {
    if base.on_ground {
        if friction <= 0.6 {
            living.speed
        } else {
            living.speed * (0.216_000_02 / (friction * friction * friction))
        }
    } else {
        FLYING_SPEED
    }
}

pub fn travel_in_air<V: BlockView>(
    base: &mut EntityBase,
    living: &mut LivingState,
    view: &V,
    input: Vec3,
) -> Option<Landing> {
    let below = movement_affecting_pos(base, view);
    let block_friction = if base.on_ground {
        compute_modified_friction(
            friction(view.block_or_air(below)),
            living.attribute(FRICTION_MODIFIER) as f32,
        )
    } else {
        1.0
    };
    let speed = friction_influenced_speed(base, living, block_friction);
    base.velocity = base.velocity + input_vector(input, speed, base.yaw);
    let climbing = on_climbable(base, view);
    if climbing {
        base.fall_distance = 0.0;
        let v = base.velocity;
        base.velocity = Vec3::new(
            v.x.clamp(-CLIMB_SPEED, CLIMB_SPEED),
            v.y.max(-CLIMB_SPEED),
            v.z.clamp(-CLIMB_SPEED, CLIMB_SPEED),
        );
    }
    let traits = living.move_traits(base);
    let landing = move_entity(base, view, base.velocity, &traits);
    let mut movement = base.velocity;
    if (base.horizontal_collision || living.jumping) && climbing {
        movement = Vec3::new(movement.x, CLIMB_BOOST, movement.z);
    }
    let mut y = movement.y;
    match living.effect("minecraft:levitation").map(|e| e.amplifier) {
        Some(amplifier) => {
            y += (LEVITATION_SPEED * (f64::from(amplifier) + 1.0) - movement.y) * LEVITATION_BLEND;
        }
        None => y -= living.effective_gravity(base),
    }
    let drag_modifier = living.attribute(AIR_DRAG_MODIFIER) as f32;
    let horizontal_drag = compute_modified_friction(AIR_FRICTION, drag_modifier);
    let horizontal = block_friction * horizontal_drag;
    let vertical = compute_modified_friction(AIR_DRAG, drag_modifier);
    base.velocity = Vec3::new(
        movement.x * f64::from(horizontal),
        y * f64::from(vertical),
        movement.z * f64::from(horizontal),
    );
    landing
}

pub fn ai_step<V: BlockView>(
    base: &mut EntityBase,
    living: &mut LivingState,
    view: &V,
) -> Option<Landing> {
    if living.no_jump_delay > 0 {
        living.no_jump_delay -= 1;
    }
    let v = base.velocity;
    let mut x = v.x;
    let mut y = v.y;
    let mut z = v.z;
    if x.abs() < VELOCITY_THRESHOLD {
        x = 0.0;
    }
    if z.abs() < VELOCITY_THRESHOLD {
        z = 0.0;
    }
    if y.abs() < VELOCITY_THRESHOLD {
        y = 0.0;
    }
    base.velocity = Vec3::new(x, y, z);
    living.xxa *= APPLIED_INPUT_DECAY;
    living.zza *= APPLIED_INPUT_DECAY;
    if health(base) <= 0.0 {
        living.jumping = false;
        living.xxa = 0.0;
        living.zza = 0.0;
    }
    if living.has_effect("minecraft:slow_falling") || living.has_effect("minecraft:levitation") {
        base.fall_distance = 0.0;
    }
    let effective_ai = !living.is_mob() || !is_no_ai(base);
    let input = Vec3::new(
        f64::from(living.xxa),
        f64::from(living.yya),
        f64::from(living.zza),
    );
    let landing = if effective_ai {
        travel_in_air(base, living, view, input)
    } else {
        None
    };
    let weaving = living.has_effect("minecraft:weaving");
    apply_inside_blocks(base, view, weaving);
    landing
}

pub fn below_world(base: &EntityBase, min_y: i32) -> bool {
    base.position.y < f64::from(min_y - 64)
}

pub fn supporting_ground(base: &EntityBase, view: &impl BlockView) -> BlockPos {
    physics::ground_pos(base, view)
}
