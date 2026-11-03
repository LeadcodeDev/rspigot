use std::time::{SystemTime, UNIX_EPOCH};

use rspigot_plugin_protocol::v1::{self, particle};

use crate::{
    Effect, EffectType, Enchantment, Error, ItemStack, Material, Particle, Position, Vector,
};

impl Position {
    pub(crate) fn from_wire(vec: Option<v1::Vec3>, field: &str) -> Result<Self, Error> {
        let vec = vec.ok_or_else(|| Error::Malformed(format!("a message without `{field}`")))?;
        Ok(Self {
            x: vec.x,
            y: vec.y,
            z: vec.z,
        })
    }

    pub(crate) const fn to_wire(self) -> v1::Vec3 {
        v1::Vec3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

impl Vector {
    pub(crate) const fn to_wire(self) -> v1::Vec3 {
        v1::Vec3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}

impl ItemStack {
    pub(crate) fn from_wire(item: v1::Item) -> Result<Self, Error> {
        let material = Material::from_key(&item.key)
            .ok_or_else(|| Error::Malformed(format!("an unknown item `{}`", item.key)))?;
        let count = u8::try_from(item.count)
            .map_err(|_| Error::Malformed(format!("an item count {}", item.count)))?;
        let enchantments = item
            .enchantments
            .into_iter()
            .map(|enchant| {
                let enchantment = Enchantment::from_key(&enchant.key).ok_or_else(|| {
                    Error::Malformed(format!("an unknown enchantment `{}`", enchant.key))
                })?;
                let level = u32::try_from(enchant.level).map_err(|_| {
                    Error::Malformed(format!("an enchantment level {}", enchant.level))
                })?;
                Ok((enchantment, level))
            })
            .collect::<Result<_, Error>>()?;
        Ok(Self {
            material,
            count,
            name: item.name,
            lore: item.lore,
            enchantments,
            components: item.components,
        })
    }

    pub(crate) fn into_wire(self) -> v1::Item {
        v1::Item {
            key: self.material.key().to_owned(),
            count: i32::from(self.count),
            name: self.name,
            lore: self.lore,
            enchantments: self
                .enchantments
                .into_iter()
                .map(|(enchantment, level)| v1::Enchant {
                    key: enchantment.key().to_owned(),
                    level: i32::try_from(level).unwrap_or(i32::MAX),
                })
                .collect(),
            components: self.components,
        }
    }
}

impl Effect {
    pub(crate) fn from_wire(effect: v1::Effect) -> Result<Self, Error> {
        let kind = EffectType::from_key(&effect.key)
            .ok_or_else(|| Error::Malformed(format!("an unknown effect `{}`", effect.key)))?;
        let amplifier = u8::try_from(effect.amplifier)
            .map_err(|_| Error::Malformed(format!("an effect amplifier {}", effect.amplifier)))?;
        Ok(Self {
            kind,
            duration: effect.duration,
            amplifier,
            ambient: effect.ambient,
            particles: effect.particles,
            icon: effect.icon,
        })
    }

    pub(crate) fn to_wire(self) -> v1::Effect {
        v1::Effect {
            key: self.kind.key().to_owned(),
            duration: self.duration,
            amplifier: i32::from(self.amplifier),
            ambient: self.ambient,
            particles: self.particles,
            icon: self.icon,
        }
    }
}

impl Particle {
    pub(crate) fn to_wire(
        self,
        position: Position,
        count: i32,
        spread: Vector,
        speed: f64,
    ) -> v1::Particle {
        let data = match self {
            Self::Item(material) => Some(particle::Data::Item(material.key().to_owned())),
            Self::Block(block)
            | Self::BlockMarker(block)
            | Self::FallingDust(block)
            | Self::DustPillar(block)
            | Self::BlockCrumble(block) => Some(particle::Data::Block(block.key().to_owned())),
            Self::Dust { color, scale } => Some(particle::Data::Dust(v1::Dust {
                color: color.to_rgb(),
                scale,
            })),
            _ => None,
        };
        v1::Particle {
            key: self.key().to_owned(),
            data,
            position: Some(position.to_wire()),
            count,
            spread: Some(spread.to_wire()),
            speed,
        }
    }
}

pub(crate) fn millis(time: SystemTime) -> i64 {
    match time.duration_since(UNIX_EPOCH) {
        Ok(after) => i64::try_from(after.as_millis()).unwrap_or(i64::MAX),
        Err(before) => i64::try_from(before.duration().as_millis()).map_or(i64::MIN, |ms| -ms),
    }
}
