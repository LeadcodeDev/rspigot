use rspigot_core::plugins::{EffectSpec, ParticleDataSpec, ParticleSpec, SoundSpec, TitleSpec};
use rspigot_domain::{Rotation, Vec3};
use rspigot_plugin_protocol::v1::{self, particle};

use super::Malformed;

pub(super) fn required<T>(field: Option<T>) -> Result<T, Malformed> {
    field.ok_or(Malformed::Empty)
}

pub(super) const fn vec3(vec: v1::Vec3) -> Vec3 {
    Vec3::new(vec.x, vec.y, vec.z)
}

pub(super) const fn wire_vec(vec: Vec3) -> v1::Vec3 {
    v1::Vec3 {
        x: vec.x,
        y: vec.y,
        z: vec.z,
    }
}

pub(super) const fn rotation(rotation: v1::Rotation) -> Rotation {
    Rotation::new(rotation.yaw, rotation.pitch)
}

pub(super) const fn wire_rotation(rotation: Rotation) -> v1::Rotation {
    v1::Rotation {
        yaw: rotation.yaw,
        pitch: rotation.pitch,
    }
}

pub(super) fn effect(effect: v1::Effect) -> EffectSpec {
    EffectSpec {
        key: effect.key,
        duration: effect.duration,
        amplifier: effect.amplifier,
        ambient: effect.ambient,
        particles: effect.particles,
        icon: effect.icon,
    }
}

pub(super) fn wire_effect(effect: EffectSpec) -> v1::Effect {
    v1::Effect {
        key: effect.key,
        duration: effect.duration,
        amplifier: effect.amplifier,
        ambient: effect.ambient,
        particles: effect.particles,
        icon: effect.icon,
    }
}

pub(super) fn particle(particle: v1::Particle) -> Result<ParticleSpec, Malformed> {
    let data = match particle.data {
        None => ParticleDataSpec::None,
        Some(particle::Data::Item(item)) => ParticleDataSpec::Item(item),
        Some(particle::Data::Block(block)) => ParticleDataSpec::Block(block),
        Some(particle::Data::Dust(dust)) => ParticleDataSpec::Dust {
            color: dust.color,
            scale: dust.scale,
        },
    };
    Ok(ParticleSpec {
        key: particle.key,
        data,
        position: vec3(required(particle.position)?),
        count: particle.count,
        spread: vec3(required(particle.spread)?),
        speed: particle.speed,
    })
}

pub(super) fn sound(sound: v1::Sound) -> SoundSpec {
    SoundSpec {
        key: sound.key,
        volume: sound.volume,
        pitch: sound.pitch,
    }
}

pub(super) fn title(title: v1::Title) -> TitleSpec {
    TitleSpec {
        title: title.title,
        subtitle: title.subtitle,
        fade_in: title.fade_in,
        stay: title.stay,
        fade_out: title.fade_out,
    }
}
