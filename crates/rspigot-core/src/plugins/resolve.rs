use std::fmt::Display;
use std::ops::RangeInclusive;

use rspigot_blocks::BlockKind;
use rspigot_data::registries::{static_entries, static_index};
use rspigot_domain::effect::{EffectDuration, EffectId, EffectInstance};
use rspigot_domain::particle::{BLOCK_PARTICLES, PARTICLES_WITH_DATA, Particle, ParticleData};
use rspigot_domain::text::Component;
use rspigot_domain::{ItemId, Rotation, Vec3};
use rspigot_items::ItemKind;

use crate::plugins::{EffectSpec, ParticleDataSpec, ParticleSpec, SoundSpec};

const WORLD_BORDER: f64 = 30_000_000.0;
const WORLD_HEIGHT: f64 = 20_000_000.0;
const PITCH: RangeInclusive<f32> = -90.0..=90.0;
const SOUND_PITCH: RangeInclusive<f32> = 0.0..=2.0;
const DUST_SCALE: RangeInclusive<f32> = 0.01..=4.0;
const INFINITE_DURATION: i32 = -1;
const ITEM_PARTICLE: &str = "minecraft:item";
const DUST_PARTICLE: &str = "minecraft:dust";

pub(super) fn text(legacy: &str) -> Component {
    Component::from_section_codes(legacy)
}

pub(super) fn within<T: PartialOrd + Display>(
    what: &str,
    value: T,
    range: RangeInclusive<T>,
) -> Result<(), String> {
    if range.contains(&value) {
        Ok(())
    } else {
        Err(format!(
            "{what} {value} is outside {}..={}",
            range.start(),
            range.end()
        ))
    }
}

pub(super) fn not_negative(what: &str, value: i32) -> Result<(), String> {
    if value >= 0 {
        Ok(())
    } else {
        Err(format!("{what} {value} is below 0"))
    }
}

pub(super) fn finite_not_negative(what: &str, value: f64) -> Result<(), String> {
    if value.is_finite() && value >= 0.0 {
        Ok(())
    } else {
        Err(format!(
            "{what} {value} is not a finite number of at least 0"
        ))
    }
}

pub(super) fn name_of(name: &str) -> Result<&str, String> {
    if name.trim().is_empty() {
        Err("name cannot be empty".to_owned())
    } else {
        Ok(name)
    }
}

pub(super) fn position_of(position: Vec3) -> Result<Vec3, String> {
    let inside = position.is_finite()
        && position.x.abs() < WORLD_BORDER
        && position.z.abs() < WORLD_BORDER
        && (-WORLD_HEIGHT..WORLD_HEIGHT).contains(&position.y);
    if inside {
        Ok(position)
    } else {
        Err(format!(
            "position {} {} {} is outside the world",
            position.x, position.y, position.z
        ))
    }
}

pub(super) fn rotation_of(rotation: Rotation) -> Result<Rotation, String> {
    if !rotation.yaw.is_finite() {
        return Err(format!("yaw {} is not finite", rotation.yaw));
    }
    within("pitch", rotation.pitch, PITCH)?;
    Ok(rotation)
}

pub(super) fn velocity_of(velocity: Vec3) -> Result<Vec3, String> {
    if velocity.is_finite() {
        Ok(velocity)
    } else {
        Err(format!(
            "velocity {} {} {} is not finite",
            velocity.x, velocity.y, velocity.z
        ))
    }
}

pub(super) fn damage_of(amount: f64) -> Result<f32, String> {
    finite_not_negative("damage", amount)?;
    Ok(amount as f32)
}

pub(super) fn effect_id(key: &str) -> Result<EffectId, String> {
    EffectId::from_name(key).ok_or_else(|| format!("unknown effect {key}"))
}

pub(super) fn effect_of(spec: &EffectSpec) -> Result<EffectInstance, String> {
    let effect = effect_id(&spec.key)?;
    let amplifier = u8::try_from(spec.amplifier)
        .map_err(|_| format!("amplifier {} is outside 0..=255", spec.amplifier))?;
    let duration = match spec.duration {
        INFINITE_DURATION => EffectDuration::Infinite,
        ticks => EffectDuration::Ticks(
            u32::try_from(ticks).map_err(|_| format!("duration {ticks} is below -1"))?,
        ),
    };
    Ok(EffectInstance {
        effect,
        amplifier,
        duration,
        ambient: spec.ambient,
        visible: spec.particles,
        show_icon: spec.icon,
    })
}

pub(super) fn sound_of(spec: &SoundSpec) -> Result<&'static str, String> {
    let sound = registered("minecraft:sound_event", &spec.key)
        .ok_or_else(|| format!("unknown sound {}", spec.key))?;
    finite_not_negative("volume", f64::from(spec.volume))?;
    within("pitch", spec.pitch, SOUND_PITCH)?;
    Ok(sound)
}

pub(super) fn particle_of(spec: &ParticleSpec) -> Result<Particle, String> {
    let kind = registered("minecraft:particle_type", &spec.key)
        .ok_or_else(|| format!("unknown particle {}", spec.key))?;
    let data = particle_data(kind, &spec.data)?;
    not_negative("count", spec.count)?;
    finite_not_negative("speed", spec.speed)?;
    Ok(Particle {
        kind,
        data,
        position: spec.position,
        count: spec.count,
        spread: spec.spread,
        speed: spec.speed,
    })
}

fn particle_data(kind: &'static str, data: &ParticleDataSpec) -> Result<ParticleData, String> {
    match data {
        ParticleDataSpec::None if !PARTICLES_WITH_DATA.contains(&kind) => Ok(ParticleData::None),
        ParticleDataSpec::Item(key) if kind == ITEM_PARTICLE => {
            match ItemKind::from_name(key).map(ItemKind::id) {
                Some(ItemId::AIR) => Err(format!("particle {kind} needs an item other than air")),
                Some(item) => Ok(ParticleData::Item(item)),
                None => Err(format!("unknown item {key}")),
            }
        }
        ParticleDataSpec::Block(key) if BLOCK_PARTICLES.contains(&kind) => {
            block_kind(key).map(|block| ParticleData::Block(block.default_state()))
        }
        ParticleDataSpec::Dust { color, scale } if kind == DUST_PARTICLE => {
            within("dust scale", *scale, DUST_SCALE)?;
            Ok(ParticleData::Dust {
                color: *color,
                scale: *scale,
            })
        }
        _ => Err(format!("particle {kind} does not take this data")),
    }
}

pub(super) fn block_kind(key: &str) -> Result<BlockKind, String> {
    BlockKind::from_name(key).ok_or_else(|| format!("unknown block {key}"))
}

fn registered(registry: &str, key: &str) -> Option<&'static str> {
    let index = usize::try_from(static_index(registry, key)?).ok()?;
    static_entries(registry)?.get(index).copied()
}
