use std::borrow::Cow;
use std::num::NonZeroU8;

use rspigot_blocks::BlockKind;
use rspigot_entities::EntityKind;
use rspigot_items::ItemKind;
use rspigot_nbt::{Compound, Tag};
use uuid::Uuid;

use super::base::{EntityBase, MAX_AIR_SUPPLY, RemovalReason};
use super::falling::FallingState;
use super::item::{self, ItemState};
use super::living::{
    self, EQUIPMENT_SLOTS, LivingState, MobState, health, is_left_handed, is_no_ai, set_health,
    set_left_handed, set_no_ai,
};
use super::orb::{self, OrbState};
use super::{Entity, EntityState};
use crate::effect::{
    AttributeModifier, AttributeOperation, EffectDuration, EffectId, EffectInstance,
};
use crate::item::ItemStack;
use crate::text::{Component, Content};
use crate::{BlockPos, BlockStateId, EntityId, ItemId, Vec3};

const MOTION_LIMIT: f64 = 10.0;
const HORIZONTAL_LIMIT: f64 = 30_000_512.0;
const VERTICAL_LIMIT: f64 = 20_000_000.0;
const BUKKIT_UPDATE_LEVEL: i32 = 2;
const ENTITY_KEYS: [&str; 29] = [
    "id",
    "Pos",
    "Motion",
    "Rotation",
    "fall_distance",
    "Fire",
    "Air",
    "OnGround",
    "Invulnerable",
    "PortalCooldown",
    "invulnerable_time",
    "UUID",
    "WorldUUIDLeast",
    "WorldUUIDMost",
    "Bukkit.updateLevel",
    "Bukkit.persist",
    "Bukkit.visibleByDefault",
    "Bukkit.invisible",
    "Bukkit.MaxAirSupply",
    "Spigot.ticksLived",
    "CustomName",
    "CustomNameVisible",
    "Silent",
    "NoGravity",
    "Glowing",
    "TicksFrozen",
    "HasVisualFire",
    "Tags",
    "data",
];
const LIVING_KEYS: [&str; 18] = [
    "Health",
    "HurtTime",
    "DeathTime",
    "AbsorptionAmount",
    "current_impulse_context_reset_grace_time",
    "attributes",
    "active_effects",
    "FallFlying",
    "equipment",
    "CanPickUpLoot",
    "PersistenceRequired",
    "drop_chances",
    "LeftHanded",
    "DeathLootTable",
    "DeathLootTableSeed",
    "NoAI",
    "home_pos",
    "home_radius",
];
const ITEM_KEYS: [&str; 6] = ["Health", "Age", "PickupDelay", "Thrower", "Owner", "Item"];
const ORB_KEYS: [&str; 4] = ["Health", "Age", "Value", "Count"];
const FALLING_KEYS: [&str; 8] = [
    "BlockState",
    "Time",
    "DropItem",
    "HurtEntities",
    "FallHurtAmount",
    "FallHurtMax",
    "TileEntityData",
    "CancelDrop",
];

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LoadError {
    #[error("entity has no id")]
    MissingId,
    #[error("unknown entity type {0}")]
    UnknownType(String),
    #[error("entity type {0} cannot be serialized")]
    NotSerializable(&'static str),
    #[error("entity has an invalid position")]
    InvalidPosition,
    #[error("item entity without an item")]
    EmptyItem,
}

pub fn is_simulated(kind: EntityKind) -> bool {
    matches!(
        kind,
        EntityKind::Item | EntityKind::FallingBlock | EntityKind::ExperienceOrb
    ) || (kind.extends("Mob") && kind != EntityKind::EnderDragon)
}

fn doubles(v: Vec3) -> Tag {
    Tag::List(vec![Tag::Double(v.x), Tag::Double(v.y), Tag::Double(v.z)])
}

fn read_vec3(compound: &Compound, key: &str) -> Option<Vec3> {
    let list = compound.get_list(key)?;
    if list.len() != 3 {
        return None;
    }
    let value = |i: usize| list[i].as_f64();
    Some(Vec3::new(value(0)?, value(1)?, value(2)?))
}

fn read_rotation(compound: &Compound) -> Option<(f32, f32)> {
    let list = compound.get_list("Rotation")?;
    if list.len() != 2 {
        return None;
    }
    Some((list[0].as_f64()? as f32, list[1].as_f64()? as f32))
}

pub fn uuid_tag(uuid: Uuid) -> Tag {
    let value = uuid.as_u128();
    Tag::IntArray(vec![
        (value >> 96) as i32,
        (value >> 64) as i32,
        (value >> 32) as i32,
        value as i32,
    ])
}

pub fn read_uuid(tag: &Tag) -> Option<Uuid> {
    match tag {
        Tag::IntArray(parts) if parts.len() == 4 => {
            let value = parts
                .iter()
                .fold(0u128, |acc, part| (acc << 32) | u128::from(*part as u32));
            Some(Uuid::from_u128(value))
        }
        _ => None,
    }
}

fn bool_tag(value: bool) -> Tag {
    Tag::Byte(i8::from(value))
}

pub fn item_stack_to_nbt(stack: &ItemStack) -> Compound {
    let mut out = Compound::new();
    out.insert(
        "id",
        stack.item.kind().map_or("minecraft:air", ItemKind::name),
    );
    out.insert("count", i32::from(stack.count.get()));
    if !stack.components.is_empty() {
        out.insert("components", stack.components.clone());
    }
    out
}

pub fn item_stack_from_nbt(compound: &Compound) -> Option<ItemStack> {
    let kind = ItemKind::from_name(compound.get_str("id")?)?;
    if kind == ItemKind::Air {
        return None;
    }
    let count = compound.get_i32("count").unwrap_or(1).clamp(1, 99);
    let mut stack = ItemStack::new(kind.id(), NonZeroU8::new(count as u8)?);
    if let Some(components) = compound.get_compound("components") {
        stack.components = components.clone();
    }
    Some(stack)
}

pub fn block_state_to_nbt(state: BlockStateId) -> Compound {
    let mut out = Compound::new();
    out.insert(
        "Name",
        state.kind().map_or("minecraft:air", BlockKind::name),
    );
    let properties: Compound = state
        .properties()
        .map(|(name, value)| (name.to_owned(), Tag::String(value.to_owned())))
        .collect();
    if !properties.is_empty() {
        out.insert("Properties", properties);
    }
    out
}

pub fn block_state_from_nbt(compound: &Compound) -> BlockStateId {
    let Some(kind) = compound.get_str("Name").and_then(BlockKind::from_name) else {
        return BlockStateId::AIR;
    };
    let mut state = kind.default_state();
    if let Some(properties) = compound.get_compound("Properties") {
        for (name, value) in properties.iter() {
            if let Some(value) = value.as_str()
                && let Some(next) = state.with_property(name, value)
            {
                state = next;
            }
        }
    }
    state
}

pub fn component_from_tag(tag: &Tag) -> Option<Component> {
    match tag {
        Tag::String(text) => Some(Component::text(text.clone())),
        Tag::Compound(compound) => {
            let mut component = if let Some(text) = compound.get_str("text") {
                Component::text(text)
            } else if let Some(key) = compound.get_str("translate") {
                let args = compound
                    .get_list("with")
                    .map(|list| list.iter().filter_map(component_from_tag).collect())
                    .unwrap_or_default();
                Component::translatable(key, args)
            } else {
                Component::text("")
            };
            if let Some(extra) = compound.get_list("extra") {
                component.extra = extra.iter().filter_map(component_from_tag).collect();
            }
            Some(component)
        }
        Tag::List(list) => {
            let mut parts = list.iter().filter_map(component_from_tag);
            let mut first = parts.next()?;
            first.extra.extend(parts);
            Some(first)
        }
        other => other
            .as_i64()
            .map(|v| Component::text(v.to_string()))
            .or_else(|| other.as_f64().map(|v| Component::text(v.to_string()))),
    }
}

pub fn component_to_tag(component: &Component) -> Tag {
    let plain = component.extra.is_empty() && component.style == crate::text::Style::default();
    match &component.content {
        Content::Text(text) if plain => Tag::String(text.clone()),
        Content::Text(text) => {
            let mut out = Compound::new().with("text", text.as_str());
            if !component.extra.is_empty() {
                out.insert(
                    "extra",
                    Tag::List(component.extra.iter().map(component_to_tag).collect()),
                );
            }
            Tag::Compound(out)
        }
        Content::Translatable { key, args } => {
            let mut out = Compound::new().with("translate", key.as_str());
            if !args.is_empty() {
                out.insert(
                    "with",
                    Tag::List(args.iter().map(component_to_tag).collect()),
                );
            }
            Tag::Compound(out)
        }
    }
}

fn operation_name(operation: AttributeOperation) -> &'static str {
    match operation {
        AttributeOperation::AddValue => "add_value",
        AttributeOperation::AddMultipliedBase => "add_multiplied_base",
        AttributeOperation::AddMultipliedTotal => "add_multiplied_total",
    }
}

fn operation_from(name: &str) -> Option<AttributeOperation> {
    Some(match name {
        "add_value" => AttributeOperation::AddValue,
        "add_multiplied_base" => AttributeOperation::AddMultipliedBase,
        "add_multiplied_total" => AttributeOperation::AddMultipliedTotal,
        _ => return None,
    })
}

fn effect_to_nbt(effect: &EffectInstance) -> Compound {
    let mut out = Compound::new();
    out.insert("id", effect.effect.name().unwrap_or_default());
    if effect.amplifier != 0 {
        out.insert("amplifier", effect.amplifier as i8);
    }
    let duration = effect.duration.wire_ticks();
    if duration != 0 {
        out.insert("duration", duration);
    }
    if effect.ambient {
        out.insert("ambient", true);
    }
    if !effect.visible {
        out.insert("show_particles", false);
    }
    out.insert("show_icon", effect.show_icon);
    out
}

fn effect_from_nbt(compound: &Compound) -> Option<EffectInstance> {
    let effect = EffectId::from_name(compound.get_str("id")?)?;
    let amplifier = compound.get_i64("amplifier").unwrap_or(0) as u8;
    let duration = match compound.get_i64("duration").unwrap_or(0) {
        -1 => EffectDuration::Infinite,
        ticks => EffectDuration::Ticks(ticks.max(0) as u32),
    };
    let visible = compound.get_bool("show_particles").unwrap_or(true);
    let mut instance = EffectInstance::new(effect, duration, amplifier, visible);
    instance.ambient = compound.get_bool("ambient").unwrap_or(false);
    instance.show_icon = compound.get_bool("show_icon").unwrap_or(visible);
    Some(instance)
}

fn read_base(nbt: &Compound, base: &mut EntityBase) -> Result<(), LoadError> {
    let position = read_vec3(nbt, "Pos").unwrap_or(Vec3::ZERO);
    let motion = read_vec3(nbt, "Motion").unwrap_or(Vec3::ZERO);
    let (yaw, pitch) = read_rotation(nbt).unwrap_or((0.0, 0.0));
    let clamp_motion = |v: f64| if v.abs() > MOTION_LIMIT { 0.0 } else { v };
    base.velocity = Vec3::new(
        clamp_motion(motion.x),
        clamp_motion(motion.y),
        clamp_motion(motion.z),
    );
    base.needs_sync = true;
    base.position = Vec3::new(
        position.x.clamp(-HORIZONTAL_LIMIT, HORIZONTAL_LIMIT),
        position.y.clamp(-VERTICAL_LIMIT, VERTICAL_LIMIT),
        position.z.clamp(-HORIZONTAL_LIMIT, HORIZONTAL_LIMIT),
    );
    if !base.position.is_finite() {
        return Err(LoadError::InvalidPosition);
    }
    base.set_yaw(yaw);
    base.set_pitch(pitch);
    base.old_position = base.position;
    base.head_yaw = base.yaw;
    base.fall_distance = nbt.get_f64("fall_distance").unwrap_or(0.0);
    base.remaining_fire_ticks = nbt.get_i64("Fire").map_or(0, |v| v as i16 as i32);
    let air = nbt.get_i64("Air").map_or(MAX_AIR_SUPPLY, |v| v as i32);
    base.set_air_supply(air);
    base.on_ground = nbt.get_bool("OnGround").unwrap_or(false);
    base.invulnerable = nbt.get_bool("Invulnerable").unwrap_or(false);
    base.invulnerable_time = nbt.get_i32("invulnerable_time").unwrap_or(0);
    base.portal_cooldown = nbt.get_i32("PortalCooldown").unwrap_or(0);
    if let Some(uuid) = nbt.get("UUID").and_then(read_uuid) {
        base.uuid = uuid;
    }
    base.set_rotation(base.yaw, base.pitch);
    if let Some(name) = nbt.get("CustomName").and_then(component_from_tag) {
        base.set_custom_name(Some(name));
    }
    base.set_custom_name_visible(nbt.get_bool("CustomNameVisible").unwrap_or(false));
    base.set_silent(nbt.get_bool("Silent").unwrap_or(false));
    base.set_no_gravity(nbt.get_bool("NoGravity").unwrap_or(false));
    base.set_glowing_tag(nbt.get_bool("Glowing").unwrap_or(false));
    base.set_ticks_frozen(nbt.get_i32("TicksFrozen").unwrap_or(0));
    base.has_visual_fire = nbt.get_bool("HasVisualFire").unwrap_or(false);
    base.custom_data = nbt.get_compound("data").cloned();
    base.tags = nbt
        .get_list("Tags")
        .map(|tags| {
            tags.iter()
                .filter_map(|t| t.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    if base.kind.living() {
        base.tick_count = nbt.get_i32("Spigot.ticksLived").unwrap_or(base.tick_count);
    }
    base.persist = nbt.get_bool("Bukkit.persist").unwrap_or(base.persist);
    base.visible_by_default = nbt
        .get_bool("Bukkit.visibleByDefault")
        .unwrap_or(base.visible_by_default);
    base.max_air_ticks = nbt
        .get_i32("Bukkit.MaxAirSupply")
        .unwrap_or(base.max_air_ticks);
    if nbt.get_bool("Bukkit.invisible").unwrap_or(false) {
        base.set_shared_flag(super::base::FLAG_INVISIBLE, true);
        base.persistent_invisibility = true;
    }
    Ok(())
}

fn read_living(nbt: &Compound, base: &mut EntityBase, living: &mut LivingState) {
    living.absorption = nbt
        .get_f64("AbsorptionAmount")
        .map_or(0.0, |v| (v as f32).max(0.0));
    if let Some(list) = nbt.get_list("attributes") {
        for entry in list.iter().filter_map(Tag::as_compound) {
            let Some(name) = entry.get_str("id") else {
                continue;
            };
            let name = if name.contains(':') {
                Cow::Borrowed(name)
            } else {
                Cow::Owned(format!("minecraft:{name}"))
            };
            let Some(instance) = living.attributes.instance(&name) else {
                continue;
            };
            if let Some(base_value) = entry.get_f64("base") {
                instance.base = base_value;
            }
            instance.modifiers = entry
                .get_list("modifiers")
                .map(|mods| {
                    mods.iter()
                        .filter_map(Tag::as_compound)
                        .filter_map(|m| {
                            Some(AttributeModifier {
                                id: Cow::Owned(m.get_str("id")?.to_owned()),
                                amount: m.get_f64("amount")?,
                                operation: operation_from(m.get_str("operation")?)?,
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();
        }
        living.attributes.take_dirty();
    }
    if let Some(effects) = nbt.get_list("active_effects") {
        for effect in effects
            .iter()
            .filter_map(Tag::as_compound)
            .filter_map(effect_from_nbt)
        {
            let _ = living.effects.apply(effect);
        }
    }
    let max = living.max_health() as f32;
    let health_value = nbt.get_f64("Health").map_or(max, |v| v as f32);
    set_health(base, living, health_value);
    living.hurt_time = nbt.get_i64("HurtTime").map_or(0, |v| v as i32);
    living.death_time = nbt.get_i64("DeathTime").map_or(0, |v| v as i32);
    living.impulse_grace = nbt
        .get_i32("current_impulse_context_reset_grace_time")
        .unwrap_or(0);
    living.fall_flying = nbt.get_bool("FallFlying").unwrap_or(false);
    if let Some(equipment) = nbt.get_compound("equipment") {
        for slot in EQUIPMENT_SLOTS {
            living.equipment[living::slot_index(slot)] = equipment
                .get_compound(slot.name())
                .and_then(item_stack_from_nbt);
        }
    }
    if let Some(mob) = &mut living.mob {
        mob.can_pick_up_loot = nbt.get_bool("CanPickUpLoot").unwrap_or(false);
        mob.persistence_required = nbt.get_bool("PersistenceRequired").unwrap_or(false);
        mob.drop_chances = [living::DEFAULT_DROP_CHANCE; 8];
        if let Some(chances) = nbt.get_compound("drop_chances") {
            for slot in EQUIPMENT_SLOTS {
                if let Some(chance) = chances.get_f64(slot.name()) {
                    mob.drop_chances[living::slot_index(slot)] = (chance as f32).max(0.0);
                }
            }
        }
        mob.death_loot_table = nbt.get_str("DeathLootTable").map(str::to_owned);
        mob.death_loot_table_seed = nbt.get_i64("DeathLootTableSeed").unwrap_or(0);
        mob.aware = nbt.get_bool("Bukkit.Aware").unwrap_or(true);
        mob.home_radius = nbt.get_i32("home_radius").unwrap_or(-1);
        if mob.home_radius >= 0 {
            mob.home_pos = nbt
                .get_int_array("home_pos")
                .filter(|pos| pos.len() == 3)
                .map_or(BlockPos::new(0, 0, 0), |pos| {
                    BlockPos::new(pos[0], pos[1], pos[2])
                });
        }
        set_left_handed(base, nbt.get_bool("LeftHanded").unwrap_or(false));
        set_no_ai(base, nbt.get_bool("NoAI").unwrap_or(false));
    }
}

fn extra_keys(nbt: &Compound, known: &[&[&str]]) -> Compound {
    nbt.iter()
        .filter(|(key, _)| !known.iter().any(|keys| keys.contains(key)))
        .map(|(key, value)| (key.to_owned(), value.clone()))
        .collect()
}

pub fn load_entity(nbt: &Compound, id: EntityId, fallback_uuid: Uuid) -> Result<Entity, LoadError> {
    let name = nbt.get_str("id").ok_or(LoadError::MissingId)?;
    let kind =
        EntityKind::from_name(name).ok_or_else(|| LoadError::UnknownType(name.to_owned()))?;
    if !kind.can_serialize() {
        return Err(LoadError::NotSerializable(kind.name()));
    }
    let mut base = EntityBase::new(id, fallback_uuid, kind);
    if !is_simulated(kind) {
        read_base(nbt, &mut base)?;
        return Ok(Entity::new(base, EntityState::Inert(nbt.clone())));
    }
    let mut state = match kind {
        EntityKind::Item => EntityState::Item(ItemState::new(
            &mut base,
            ItemStack::new(ItemId(0), NonZeroU8::MIN),
            0,
        )),
        EntityKind::FallingBlock => EntityState::Falling(FallingState::new(
            &mut base,
            BlockKind::Sand.default_state(),
            BlockPos::new(0, 0, 0),
        )),
        EntityKind::ExperienceOrb => EntityState::Orb(OrbState::new(&mut base, 0)),
        _ => EntityState::Living(Box::new(LivingState::new(&mut base, 0.0))),
    };
    read_base(nbt, &mut base)?;
    let known: &[&[&str]] = match &mut state {
        EntityState::Item(item_state) => {
            item_state.health = nbt.get_i64("Health").map_or(5, |v| v as i16 as i32);
            item_state.age = nbt.get_i64("Age").map_or(0, |v| v as i16 as i32);
            item_state.pickup_delay = nbt.get_i64("PickupDelay").map_or(0, |v| v as i16 as i32);
            item_state.owner = nbt.get("Owner").and_then(read_uuid);
            item_state.thrower = nbt.get("Thrower").and_then(read_uuid);
            let stack = nbt
                .get_compound("Item")
                .and_then(item_stack_from_nbt)
                .ok_or(LoadError::EmptyItem)?;
            item::set_stack(&mut base, Some(stack));
            &[&ENTITY_KEYS, &ITEM_KEYS]
        }
        EntityState::Falling(falling) => {
            falling.state = nbt
                .get_compound("BlockState")
                .map_or(BlockKind::Sand.default_state(), block_state_from_nbt);
            falling.time = nbt.get_i32("Time").unwrap_or(0);
            if let Some(hurt) = nbt.get_bool("HurtEntities") {
                falling.hurt_entities = hurt;
                falling.fall_hurt_amount = nbt.get_f64("FallHurtAmount").map_or(0.0, |v| v as f32);
                falling.fall_hurt_max = nbt.get_i32("FallHurtMax").unwrap_or(40);
            }
            falling.drop_item = nbt.get_bool("DropItem").unwrap_or(true);
            falling.block_data = nbt.get_compound("TileEntityData").cloned();
            falling.cancel_drop = nbt.get_bool("CancelDrop").unwrap_or(false);
            &[&ENTITY_KEYS, &FALLING_KEYS]
        }
        EntityState::Living(living_state) => {
            read_living(nbt, &mut base, living_state);
            &[&ENTITY_KEYS, &LIVING_KEYS]
        }
        EntityState::Orb(orb_state) => {
            orb_state.health = nbt
                .get_i64("Health")
                .map_or(orb::DEFAULT_HEALTH, |v| i32::from(v as i16));
            orb_state.age = nbt.get_i64("Age").map_or(0, |v| i32::from(v as i16));
            orb::set_value(
                &mut base,
                nbt.get_i64("Value").map_or(0, |v| i32::from(v as i16)),
            );
            orb_state.count = nbt.get_i32("Count").filter(|count| *count > 0).unwrap_or(1);
            &[&ENTITY_KEYS, &ORB_KEYS]
        }
        EntityState::Inert(_) => &[],
    };
    let extra = extra_keys(nbt, known);
    Ok(Entity { base, state, extra })
}

fn write_base(entity: &Entity, out: &mut Compound, world: Option<Uuid>) {
    let base = &entity.base;
    out.insert("Pos", doubles(base.position));
    out.insert("Motion", doubles(base.velocity));
    let yaw = if base.yaw.is_nan() { 0.0 } else { base.yaw };
    let pitch = if base.pitch.is_nan() { 0.0 } else { base.pitch };
    out.insert(
        "Rotation",
        Tag::List(vec![Tag::Float(yaw), Tag::Float(pitch)]),
    );
    out.insert("fall_distance", base.fall_distance);
    out.insert("Fire", base.remaining_fire_ticks as i16);
    out.insert("Air", base.air_supply() as i16);
    out.insert("OnGround", bool_tag(base.on_ground));
    out.insert("Invulnerable", bool_tag(base.invulnerable));
    out.insert("PortalCooldown", base.portal_cooldown);
    if base.invulnerable_time > 0 {
        out.insert("invulnerable_time", base.invulnerable_time);
    }
    out.insert("UUID", uuid_tag(base.uuid));
    if let Some(world) = world {
        let (most, least) = world.as_u64_pair();
        out.insert("WorldUUIDLeast", least as i64);
        out.insert("WorldUUIDMost", most as i64);
    }
    out.insert("Bukkit.updateLevel", BUKKIT_UPDATE_LEVEL);
    if !base.persist {
        out.insert("Bukkit.persist", false);
    }
    if !base.visible_by_default {
        out.insert("Bukkit.visibleByDefault", false);
    }
    if base.persistent_invisibility {
        out.insert("Bukkit.invisible", true);
    }
    if base.max_air_ticks != MAX_AIR_SUPPLY {
        out.insert("Bukkit.MaxAirSupply", base.max_air_ticks);
    }
    out.insert("Spigot.ticksLived", base.tick_count);
    if let Some(name) = base.custom_name() {
        out.insert("CustomName", component_to_tag(name));
    }
    if base.custom_name_visible() {
        out.insert("CustomNameVisible", true);
    }
    if base.is_silent() {
        out.insert("Silent", true);
    }
    if base.is_no_gravity() {
        out.insert("NoGravity", true);
    }
    if base.has_glowing_tag {
        out.insert("Glowing", true);
    }
    if base.ticks_frozen() > 0 {
        out.insert("TicksFrozen", base.ticks_frozen());
    }
    if base.has_visual_fire {
        out.insert("HasVisualFire", true);
    }
    if !base.tags.is_empty() {
        out.insert(
            "Tags",
            Tag::List(base.tags.iter().map(|t| Tag::String(t.clone())).collect()),
        );
    }
    if let Some(data) = &base.custom_data
        && !data.is_empty()
    {
        out.insert("data", data.clone());
    }
}

fn write_living(base: &EntityBase, living: &LivingState, out: &mut Compound) {
    out.insert("Health", health(base));
    out.insert("HurtTime", living.hurt_time as i16);
    out.insert("DeathTime", living.death_time as i16);
    out.insert("AbsorptionAmount", living.absorption);
    out.insert(
        "current_impulse_context_reset_grace_time",
        living.impulse_grace,
    );
    let attributes: Vec<Tag> = living
        .attributes
        .instances()
        .iter()
        .map(|instance| {
            let mut entry = Compound::new()
                .with("id", instance.attribute)
                .with("base", instance.base);
            if !instance.modifiers.is_empty() {
                entry.insert(
                    "modifiers",
                    Tag::List(
                        instance
                            .modifiers
                            .iter()
                            .map(|m| {
                                Tag::Compound(
                                    Compound::new()
                                        .with("id", m.id.as_ref())
                                        .with("amount", m.amount)
                                        .with("operation", operation_name(m.operation)),
                                )
                            })
                            .collect(),
                    ),
                );
            }
            Tag::Compound(entry)
        })
        .collect();
    out.insert("attributes", Tag::List(attributes));
    if !living.effects.is_empty() {
        out.insert(
            "active_effects",
            Tag::List(
                living
                    .effects
                    .iter()
                    .map(|e| Tag::Compound(effect_to_nbt(e)))
                    .collect(),
            ),
        );
    }
    out.insert("FallFlying", bool_tag(living.fall_flying));
    let equipment: Compound = EQUIPMENT_SLOTS
        .iter()
        .filter_map(|slot| {
            living.equipment[living::slot_index(*slot)]
                .as_ref()
                .map(|stack| {
                    (
                        slot.name().to_owned(),
                        Tag::Compound(item_stack_to_nbt(stack)),
                    )
                })
        })
        .collect();
    if !equipment.is_empty() {
        out.insert("equipment", equipment);
    }
    if let Some(mob) = &living.mob {
        write_mob(base, mob, out);
    }
}

fn write_mob(base: &EntityBase, mob: &MobState, out: &mut Compound) {
    out.insert("CanPickUpLoot", bool_tag(mob.can_pick_up_loot));
    out.insert("PersistenceRequired", bool_tag(mob.persistence_required));
    let chances: Compound = EQUIPMENT_SLOTS
        .iter()
        .filter(|slot| mob.drop_chances[living::slot_index(**slot)] != living::DEFAULT_DROP_CHANCE)
        .map(|slot| {
            (
                slot.name().to_owned(),
                Tag::Float(mob.drop_chances[living::slot_index(*slot)]),
            )
        })
        .collect();
    if !chances.is_empty() {
        out.insert("drop_chances", chances);
    }
    if mob.home_radius != -1 {
        out.insert("home_radius", mob.home_radius);
        let pos = mob.home_pos;
        out.insert("home_pos", Tag::IntArray(vec![pos.x, pos.y, pos.z]));
    }
    out.insert("LeftHanded", bool_tag(is_left_handed(base)));
    if let Some(table) = &mob.death_loot_table {
        out.insert("DeathLootTable", table.as_str());
    }
    if mob.death_loot_table_seed != 0 {
        out.insert("DeathLootTableSeed", mob.death_loot_table_seed);
    }
    if is_no_ai(base) {
        out.insert("NoAI", true);
    }
    out.insert("Bukkit.Aware", bool_tag(mob.aware));
}

pub fn save_entity(entity: &Entity, world: Option<Uuid>) -> Option<Compound> {
    if let Some(reason) = entity.base.removal
        && !matches!(reason, RemovalReason::UnloadedToChunk)
    {
        return None;
    }
    if !entity.base.persist || !entity.base.kind.can_serialize() {
        return None;
    }
    if let EntityState::Inert(nbt) = &entity.state {
        return Some(nbt.clone());
    }
    let mut out = Compound::new();
    out.insert("id", entity.base.kind.name());
    write_base(entity, &mut out, world);
    match &entity.state {
        EntityState::Item(item_state) => {
            out.insert("Health", item_state.health as i16);
            out.insert("Age", item_state.age as i16);
            out.insert("PickupDelay", item_state.pickup_delay as i16);
            if let Some(thrower) = item_state.thrower {
                out.insert("Thrower", uuid_tag(thrower));
            }
            if let Some(owner) = item_state.owner {
                out.insert("Owner", uuid_tag(owner));
            }
            if let Some(stack) = item::stack(&entity.base) {
                out.insert("Item", item_stack_to_nbt(stack));
            }
        }
        EntityState::Falling(falling) => {
            out.insert("BlockState", block_state_to_nbt(falling.state));
            out.insert("Time", falling.time);
            out.insert("DropItem", bool_tag(falling.drop_item));
            out.insert("HurtEntities", bool_tag(falling.hurt_entities));
            out.insert("FallHurtAmount", falling.fall_hurt_amount);
            out.insert("FallHurtMax", falling.fall_hurt_max);
            if let Some(data) = &falling.block_data {
                out.insert("TileEntityData", data.clone());
            }
            out.insert("CancelDrop", bool_tag(falling.cancel_drop));
        }
        EntityState::Living(living_state) => write_living(&entity.base, living_state, &mut out),
        EntityState::Orb(orb_state) => {
            out.insert("Health", orb_state.health as i16);
            out.insert("Age", orb_state.age as i16);
            out.insert("Value", orb::value(&entity.base) as i16);
            out.insert("Count", orb_state.count);
        }
        EntityState::Inert(_) => {}
    }
    for (key, value) in entity.extra.iter() {
        if !out.contains_key(key) {
            out.insert(key, value.clone());
        }
    }
    Some(out)
}

pub fn entity_position(nbt: &Compound) -> Option<Vec3> {
    read_vec3(nbt, "Pos")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cow_nbt() -> Compound {
        Compound::new()
            .with("id", "minecraft:cow")
            .with("Pos", doubles(Vec3::new(1.5, 64.0, 2.5)))
            .with(
                "Rotation",
                Tag::List(vec![Tag::Float(90.0), Tag::Float(0.0)]),
            )
            .with("UUID", uuid_tag(Uuid::from_u128(42)))
            .with("Health", 7.5f32)
            .with("NoAI", true)
            .with("variant", "minecraft:warm")
    }

    #[test]
    fn mob_round_trip_keeps_known_and_unknown_keys() {
        let entity = load_entity(&cow_nbt(), EntityId(1), Uuid::from_u128(1)).unwrap();
        assert_eq!(entity.base.uuid, Uuid::from_u128(42));
        assert_eq!(entity.base.position, Vec3::new(1.5, 64.0, 2.5));
        assert_eq!(entity.base.yaw, 90.0);
        assert_eq!(health(&entity.base), 7.5);
        assert!(is_no_ai(&entity.base));
        let saved = save_entity(&entity, None).unwrap();
        assert_eq!(saved.get_str("id"), Some("minecraft:cow"));
        assert_eq!(saved.get_str("variant"), Some("minecraft:warm"));
        assert_eq!(saved.get_bool("NoAI"), Some(true));
        assert_eq!(saved.get_f64("Health"), Some(7.5));
        let again = load_entity(&saved, EntityId(2), Uuid::from_u128(2)).unwrap();
        assert_eq!(save_entity(&again, None), Some(saved));
    }

    #[test]
    fn mob_home_restriction_survives_a_round_trip() {
        let nbt = cow_nbt()
            .with("home_radius", 6)
            .with("home_pos", Tag::IntArray(vec![3, 64, -2]));
        let entity = load_entity(&nbt, EntityId(1), Uuid::from_u128(1)).unwrap();
        let mob = entity.living().and_then(|l| l.mob.as_ref()).unwrap();
        assert_eq!(
            (mob.home_pos, mob.home_radius),
            (BlockPos::new(3, 64, -2), 6)
        );
        assert!(!entity.extra.contains_key("home_pos"));
        let saved = save_entity(&entity, None).unwrap();
        assert_eq!(saved.get_i32("home_radius"), Some(6));
        assert_eq!(saved.get_int_array("home_pos"), Some(&[3, 64, -2][..]));
        let free = load_entity(&cow_nbt(), EntityId(2), Uuid::from_u128(2)).unwrap();
        let saved = save_entity(&free, None).unwrap();
        assert!(!saved.contains_key("home_radius"));
        assert!(!saved.contains_key("home_pos"));
    }

    #[test]
    fn unknown_types_are_kept_verbatim() {
        let nbt = Compound::new()
            .with("id", "minecraft:armor_stand")
            .with("Pos", doubles(Vec3::new(0.0, 70.0, 0.0)))
            .with("ShowArms", true)
            .with("Pose", Compound::new().with("Head", Tag::List(Vec::new())));
        let entity = load_entity(&nbt, EntityId(1), Uuid::from_u128(1)).unwrap();
        assert!(!entity.is_simulated());
        assert_eq!(save_entity(&entity, None), Some(nbt));
    }

    #[test]
    fn excessive_motion_is_dropped_like_vanilla() {
        let nbt = cow_nbt().with("Motion", doubles(Vec3::new(11.0, -0.5, 3.0)));
        let entity = load_entity(&nbt, EntityId(1), Uuid::from_u128(1)).unwrap();
        assert_eq!(entity.base.velocity, Vec3::new(0.0, -0.5, 3.0));
    }

    #[test]
    fn experience_orbs_keep_value_count_and_age() {
        let nbt = Compound::new()
            .with("id", "minecraft:experience_orb")
            .with("Pos", doubles(Vec3::new(0.5, 65.0, 0.5)))
            .with("Value", 7i16)
            .with("Count", 3)
            .with("Age", 120i16)
            .with("Health", 4i16);
        let entity = load_entity(&nbt, EntityId(1), Uuid::from_u128(1)).unwrap();
        let EntityState::Orb(state) = &entity.state else {
            panic!("not an orb");
        };
        assert_eq!((state.count, state.age, state.health), (3, 120, 4));
        assert_eq!(orb::value(&entity.base), 7);
        let saved = save_entity(&entity, None).unwrap();
        assert_eq!(saved.get("Value"), Some(&Tag::Short(7)));
        assert_eq!(saved.get("Count"), Some(&Tag::Int(3)));
        assert_eq!(saved.get("Age"), Some(&Tag::Short(120)));
        assert_eq!(saved.get("Health"), Some(&Tag::Short(4)));
        let invalid = load_entity(
            &nbt.clone().with("Count", 0),
            EntityId(2),
            Uuid::from_u128(2),
        )
        .unwrap();
        assert!(matches!(
            invalid.state,
            EntityState::Orb(OrbState { count: 1, .. })
        ));
    }

    #[test]
    fn item_and_falling_block_keys() {
        let item_nbt = Compound::new()
            .with("id", "minecraft:item")
            .with("Pos", doubles(Vec3::new(0.0, 70.0, 0.0)))
            .with(
                "Item",
                Compound::new()
                    .with("id", "minecraft:stone")
                    .with("count", 3),
            )
            .with("PickupDelay", 10i16);
        let entity = load_entity(&item_nbt, EntityId(1), Uuid::from_u128(1)).unwrap();
        assert_eq!(item::stack(&entity.base).map(|s| s.count.get()), Some(3));
        let saved = save_entity(&entity, None).unwrap();
        assert_eq!(saved.get_i64("PickupDelay"), Some(10));
        let falling_nbt = Compound::new()
            .with("id", "minecraft:falling_block")
            .with("Pos", doubles(Vec3::new(0.0, 70.0, 0.0)))
            .with(
                "BlockState",
                Compound::new()
                    .with("Name", "minecraft:anvil")
                    .with("Properties", Compound::new().with("facing", "east")),
            );
        let falling = load_entity(&falling_nbt, EntityId(2), Uuid::from_u128(2)).unwrap();
        let EntityState::Falling(state) = &falling.state else {
            panic!("not a falling block");
        };
        assert_eq!(state.state.property("facing"), Some("east"));
    }
}
