use rspigot_effects::{
    AttributeOperation, AttributeTemplate, EffectCategory, EffectId, EffectKind, MobEffect,
    Regeneration, Speed,
};

#[test]
fn forty_effects_follow_the_registry() {
    assert_eq!(EffectKind::ALL.len(), 40);
    let registry = rspigot_data::registries::static_entries("minecraft:mob_effect").unwrap();
    let names: Vec<_> = EffectKind::ALL.iter().map(|kind| kind.name()).collect();
    assert_eq!(names, registry);
    for (index, kind) in EffectKind::ALL.into_iter().enumerate() {
        assert_eq!(usize::from(kind.id().0), index);
        assert_eq!(EffectKind::from_id(kind.id()), Some(kind));
        assert_eq!(kind.id().kind(), Some(kind));
        assert_eq!(kind.id().name(), Some(kind.name()));
        assert_eq!(EffectKind::from_name(kind.name()), Some(kind));
        assert_eq!(EffectId::from_name(kind.name()), Some(kind.id()));
        let path = kind.name().strip_prefix("minecraft:").unwrap();
        assert_eq!(EffectKind::from_name(path), Some(kind));
        assert_eq!(kind.translation_key(), format!("effect.minecraft.{path}"));
    }
    assert_eq!(EffectKind::from_id(EffectId(40)), None);
    assert_eq!(EffectId(40).kind(), None);
    assert_eq!(EffectId(40).name(), None);
    assert_eq!(EffectId(40).translation_key(), None);
}

#[test]
fn speed_is_effect_zero() {
    assert_eq!(Speed::ID, EffectId(0));
    assert_eq!(Speed::NAME, "minecraft:speed");
    assert_eq!(Speed::COLOR, 0x33EBFF);
    assert_eq!(EffectKind::from_name("speed"), Some(EffectKind::Speed));
    assert_eq!(
        EffectKind::from_name("minecraft:speed"),
        Some(EffectKind::Speed)
    );
    assert_eq!(EffectKind::from_name("other:speed"), None);
    assert_eq!(EffectKind::from_name("not_an_effect"), None);
    assert_eq!(
        EffectKind::Speed.translation_key(),
        "effect.minecraft.speed"
    );
    assert_eq!(EffectId::from_name("speed"), Some(EffectId(0)));
    assert_eq!(EffectId(0).name(), Some("minecraft:speed"));
    assert_eq!(
        EffectId(0).translation_key().as_deref(),
        Some("effect.minecraft.speed")
    );
}

const fn template(
    attribute: &'static str,
    modifier: &'static str,
    amount: f64,
    operation: AttributeOperation,
) -> AttributeTemplate {
    AttributeTemplate {
        attribute,
        modifier,
        amount,
        operation,
    }
}

#[test]
fn modifiers_match_vanilla() {
    use AttributeOperation::{AddMultipliedTotal, AddValue};
    let expected = [
        (
            EffectKind::Speed,
            template(
                "minecraft:movement_speed",
                "minecraft:effect.speed",
                0.20000000298023224,
                AddMultipliedTotal,
            ),
        ),
        (
            EffectKind::Slowness,
            template(
                "minecraft:movement_speed",
                "minecraft:effect.slowness",
                -0.15000000596046448,
                AddMultipliedTotal,
            ),
        ),
        (
            EffectKind::Haste,
            template(
                "minecraft:attack_speed",
                "minecraft:effect.haste",
                0.10000000149011612,
                AddMultipliedTotal,
            ),
        ),
        (
            EffectKind::MiningFatigue,
            template(
                "minecraft:attack_speed",
                "minecraft:effect.mining_fatigue",
                -0.10000000149011612,
                AddMultipliedTotal,
            ),
        ),
        (
            EffectKind::Strength,
            template(
                "minecraft:attack_damage",
                "minecraft:effect.strength",
                3.0,
                AddValue,
            ),
        ),
        (
            EffectKind::JumpBoost,
            template(
                "minecraft:safe_fall_distance",
                "minecraft:effect.jump_boost",
                1.0,
                AddValue,
            ),
        ),
        (
            EffectKind::Invisibility,
            template(
                "minecraft:waypoint_transmit_range",
                "minecraft:effect.waypoint_transmit_range_hide",
                -1.0,
                AddMultipliedTotal,
            ),
        ),
        (
            EffectKind::Weakness,
            template(
                "minecraft:attack_damage",
                "minecraft:effect.weakness",
                -4.0,
                AddValue,
            ),
        ),
        (
            EffectKind::HealthBoost,
            template(
                "minecraft:max_health",
                "minecraft:effect.health_boost",
                4.0,
                AddValue,
            ),
        ),
        (
            EffectKind::Absorption,
            template(
                "minecraft:max_absorption",
                "minecraft:effect.absorption",
                4.0,
                AddValue,
            ),
        ),
        (
            EffectKind::Luck,
            template("minecraft:luck", "minecraft:effect.luck", 1.0, AddValue),
        ),
        (
            EffectKind::Unluck,
            template("minecraft:luck", "minecraft:effect.unluck", -1.0, AddValue),
        ),
    ];
    for (kind, modifier) in expected {
        assert_eq!((kind, kind.modifiers()), (kind, &[modifier][..]));
    }
    assert_eq!(Speed::MODIFIERS, EffectKind::Speed.modifiers());
}

#[test]
fn amount_scales_with_amplifier() {
    let speed = EffectKind::Speed.modifiers()[0];
    assert_eq!(speed.amount_for(0), speed.amount);
    assert!((speed.amount_for(250) - 50.2).abs() < 1e-5);
}

#[test]
fn categories_match_vanilla() {
    assert_eq!(EffectKind::Speed.category(), EffectCategory::Beneficial);
    assert_eq!(EffectKind::Poison.category(), EffectCategory::Harmful);
    assert_eq!(EffectKind::Glowing.category(), EffectCategory::Neutral);
    assert!(EffectCategory::Beneficial.is_beneficial());
    assert!(!EffectCategory::Neutral.is_beneficial());
}

#[test]
fn only_instant_health_damage_and_saturation_are_instantaneous() {
    let instantaneous: Vec<_> = EffectKind::ALL
        .into_iter()
        .filter(|kind| kind.instantaneous())
        .collect();
    assert_eq!(
        instantaneous,
        [
            EffectKind::InstantHealth,
            EffectKind::InstantDamage,
            EffectKind::Saturation
        ]
    );
    for kind in instantaneous {
        assert!(kind.should_tick(1, 0));
        assert!(kind.should_tick(600, 3));
        assert!(!kind.should_tick(0, 0));
    }
}

#[test]
fn regeneration_ticks_every_fifty_at_amplifier_zero() {
    for duration in -200..=2000 {
        assert_eq!(
            (duration, EffectKind::Regeneration.should_tick(duration, 0)),
            (duration, duration % 50 == 0)
        );
    }
    assert_eq!(Regeneration::ID, EffectId(9));
}

#[test]
fn tick_intervals_shift_by_amplifier() {
    let regeneration = EffectKind::Regeneration;
    assert!(regeneration.should_tick(25, 1) && !regeneration.should_tick(26, 1));
    assert!(regeneration.should_tick(7, 6));
    assert!(regeneration.should_tick(100, 32) && !regeneration.should_tick(25, 32));
    assert!(EffectKind::Poison.should_tick(25, 0) && !EffectKind::Poison.should_tick(24, 0));
    assert!(EffectKind::Wither.should_tick(40, 0) && !EffectKind::Wither.should_tick(25, 0));
    assert!((0..100).all(|duration| EffectKind::Hunger.should_tick(duration, 0)));
    assert!(EffectKind::RaidOmen.should_tick(1, 0) && !EffectKind::RaidOmen.should_tick(2, 0));
    assert!(!EffectKind::Speed.should_tick(50, 0));
}
