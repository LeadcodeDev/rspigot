const REGISTRY: &str = "minecraft:damage_type";
const FALL_POWER_EPSILON: f64 = 1.0e-6;
const ARMOR_TOUGHNESS_BASE: f32 = 2.0;
const ARMOR_TOUGHNESS_DIVIDER: f32 = 4.0;
const MIN_ARMOR_RATIO: f32 = 0.2;
const MAX_ARMOR: f32 = 20.0;
const ARMOR_PROTECTION_DIVIDER: f32 = 25.0;
const RESISTANCE_PER_LEVEL: i32 = 5;
const RESISTANCE_DIVIDER: i32 = 25;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DamageType {
    Fall,
    OutOfWorld,
    FallingAnvil,
    FallingBlock,
    FallingStalactite,
    Stalagmite,
    PlayerAttack,
    GenericKill,
    Generic,
    Starve,
    Magic,
    Wither,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DamageTag {
    AlwaysMostSignificantFall,
    BypassesArmor,
    BypassesCooldown,
    BypassesEffects,
    BypassesInvulnerability,
    BypassesResistance,
    DamagesHelmet,
    IsFall,
    IsFire,
    NoImpact,
    NoKnockback,
}

impl DamageTag {
    pub const fn name(self) -> &'static str {
        match self {
            Self::AlwaysMostSignificantFall => "minecraft:always_most_significant_fall",
            Self::BypassesArmor => "minecraft:bypasses_armor",
            Self::BypassesCooldown => "minecraft:bypasses_cooldown",
            Self::BypassesEffects => "minecraft:bypasses_effects",
            Self::BypassesInvulnerability => "minecraft:bypasses_invulnerability",
            Self::BypassesResistance => "minecraft:bypasses_resistance",
            Self::DamagesHelmet => "minecraft:damages_helmet",
            Self::IsFall => "minecraft:is_fall",
            Self::IsFire => "minecraft:is_fire",
            Self::NoImpact => "minecraft:no_impact",
            Self::NoKnockback => "minecraft:no_knockback",
        }
    }
}

impl DamageType {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Fall => "minecraft:fall",
            Self::OutOfWorld => "minecraft:out_of_world",
            Self::FallingAnvil => "minecraft:falling_anvil",
            Self::FallingBlock => "minecraft:falling_block",
            Self::FallingStalactite => "minecraft:falling_stalactite",
            Self::Stalagmite => "minecraft:stalagmite",
            Self::PlayerAttack => "minecraft:player_attack",
            Self::GenericKill => "minecraft:generic_kill",
            Self::Generic => "minecraft:generic",
            Self::Starve => "minecraft:starve",
            Self::Magic => "minecraft:magic",
            Self::Wither => "minecraft:wither",
        }
    }

    pub const fn message_id(self) -> &'static str {
        match self {
            Self::Fall => "fall",
            Self::OutOfWorld => "outOfWorld",
            Self::FallingAnvil => "anvil",
            Self::FallingBlock => "fallingBlock",
            Self::FallingStalactite => "fallingStalactite",
            Self::Stalagmite => "stalagmite",
            Self::PlayerAttack => "player",
            Self::GenericKill => "genericKill",
            Self::Generic => "generic",
            Self::Starve => "starve",
            Self::Magic => "magic",
            Self::Wither => "wither",
        }
    }

    pub const fn exhaustion(self) -> f32 {
        match self {
            Self::FallingAnvil | Self::FallingBlock | Self::FallingStalactite => 0.1,
            Self::PlayerAttack => 0.1,
            _ => 0.0,
        }
    }

    pub const fn fall_variants(self) -> bool {
        matches!(self, Self::Fall)
    }

    pub fn registry_id(self) -> Option<i32> {
        rspigot_data::registries::synced_index(REGISTRY, self.name())
            .and_then(|id| i32::try_from(id).ok())
    }

    pub fn is(self, tag: DamageTag) -> bool {
        let Some(id) = rspigot_data::registries::synced_index(REGISTRY, self.name()) else {
            return false;
        };
        rspigot_data::tags::tag(REGISTRY, tag.name())
            .is_some_and(|members| members.binary_search(&id).is_ok())
    }
}

pub fn java_clamp(value: f32, min: f32, max: f32) -> f32 {
    if value < min { min } else { value.min(max) }
}

pub fn fall_damage(distance: f64, multiplier: f32, safe_fall: f64, damage_multiplier: f64) -> i32 {
    let power = distance + FALL_POWER_EPSILON - safe_fall;
    (power * f64::from(multiplier) * damage_multiplier).floor() as i32
}

pub fn after_armor(damage: f32, armor: f32, toughness: f32) -> f32 {
    let toughness = ARMOR_TOUGHNESS_BASE + toughness / ARMOR_TOUGHNESS_DIVIDER;
    let effective = java_clamp(
        armor - damage / toughness,
        armor * MIN_ARMOR_RATIO,
        MAX_ARMOR,
    );
    damage * (1.0 - effective / ARMOR_PROTECTION_DIVIDER)
}

pub fn after_resistance(damage: f32, amplifier: u8) -> f32 {
    let reduction = (i32::from(amplifier) + 1) * RESISTANCE_PER_LEVEL;
    let kept = (RESISTANCE_DIVIDER - reduction) as f32;
    (damage * kept / RESISTANCE_DIVIDER as f32).max(0.0)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Absorbed {
    pub damage: f32,
    pub absorption: f32,
}

pub fn absorb(damage: f32, absorption: f32) -> Absorbed {
    let remaining = (damage - absorption).max(0.0);
    Absorbed {
        damage: remaining,
        absorption: absorption - (damage - remaining),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_ids_follow_the_synced_damage_type_order() {
        assert_eq!(DamageType::Fall.registry_id(), Some(10));
        assert_eq!(DamageType::FallingAnvil.registry_id(), Some(11));
        assert_eq!(DamageType::GenericKill.registry_id(), Some(19));
        assert_eq!(DamageType::OutOfWorld.registry_id(), Some(32));
        assert_eq!(DamageType::PlayerAttack.registry_id(), Some(34));
        assert_eq!(DamageType::Wither.registry_id(), Some(49));
    }

    #[test]
    fn tags_come_from_the_datapack() {
        assert!(DamageType::OutOfWorld.is(DamageTag::BypassesInvulnerability));
        assert!(DamageType::GenericKill.is(DamageTag::BypassesInvulnerability));
        assert!(!DamageType::Fall.is(DamageTag::BypassesInvulnerability));
        assert!(DamageType::Fall.is(DamageTag::IsFall));
        assert!(DamageType::Stalagmite.is(DamageTag::IsFall));
        assert!(DamageType::Fall.is(DamageTag::NoKnockback));
        assert!(DamageType::Magic.is(DamageTag::NoKnockback));
        assert!(!DamageType::PlayerAttack.is(DamageTag::NoKnockback));
        assert!(!DamageType::FallingAnvil.is(DamageTag::NoKnockback));
        assert!(DamageType::FallingAnvil.is(DamageTag::DamagesHelmet));
        assert!(DamageType::Starve.is(DamageTag::BypassesEffects));
        assert!(DamageType::OutOfWorld.is(DamageTag::AlwaysMostSignificantFall));
        assert!(!DamageType::Magic.is(DamageTag::BypassesCooldown));
    }

    #[test]
    fn fall_damage_subtracts_the_safe_distance() {
        assert_eq!(fall_damage(20.0, 1.0, 3.0, 1.0), 17);
        assert_eq!(fall_damage(3.0, 1.0, 3.0, 1.0), 0);
        assert_eq!(fall_damage(3.5, 1.0, 3.0, 1.0), 0);
        assert_eq!(fall_damage(4.0, 1.0, 3.0, 1.0), 1);
        assert_eq!(fall_damage(1.0, 1.0, 3.0, 1.0), -2);
    }

    #[test]
    fn jump_boost_raises_the_safe_distance_and_hay_cuts_damage() {
        assert_eq!(fall_damage(20.0, 1.0, 5.0, 1.0), 15);
        assert_eq!(fall_damage(20.0, 0.2, 3.0, 1.0), 3);
        assert_eq!(fall_damage(30.0, 0.0, 3.0, 1.0), 0);
        assert_eq!(fall_damage(10.0 * 0.5, 1.0, 3.0, 1.0), 2);
        assert_eq!(fall_damage(4.0 + 2.5, 2.0, 3.0, 1.0), 7);
    }

    #[test]
    fn resistance_removes_a_fifth_per_level() {
        assert_eq!(after_resistance(10.0, 0), 8.0);
        assert_eq!(after_resistance(10.0, 1), 6.0);
        assert_eq!(after_resistance(10.0, 3), 2.0);
        assert_eq!(after_resistance(10.0, 4), 0.0);
        assert_eq!(after_resistance(10.0, 200), 0.0);
    }

    #[test]
    fn absorption_soaks_damage_first() {
        assert_eq!(
            absorb(3.0, 4.0),
            Absorbed {
                damage: 0.0,
                absorption: 1.0
            }
        );
        assert_eq!(
            absorb(6.0, 4.0),
            Absorbed {
                damage: 2.0,
                absorption: 0.0
            }
        );
        assert_eq!(
            absorb(5.0, 0.0),
            Absorbed {
                damage: 5.0,
                absorption: 0.0
            }
        );
    }

    #[test]
    fn armor_follows_combat_rules() {
        assert_eq!(after_armor(10.0, 0.0, 0.0), 10.0);
        assert!((after_armor(10.0, 20.0, 0.0) - 10.0 * (1.0 - 15.0 / 25.0)).abs() < 1e-6);
        assert!((after_armor(100.0, 20.0, 0.0) - 100.0 * (1.0 - 4.0 / 25.0)).abs() < 1e-4);
        assert!((after_armor(4.0, 2.0, 0.0) - 4.0 * (1.0 - 0.4 / 25.0)).abs() < 1e-6);
    }
}
