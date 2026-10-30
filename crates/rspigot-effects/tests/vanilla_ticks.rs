use rspigot_effects::EffectKind;

const DURATIONS: (i32, i32) = (-2, 400);
const AMPLIFIERS: [u8; 12] = [0, 1, 2, 3, 4, 5, 6, 7, 31, 32, 33, 255];
const FINGERPRINTS: [(EffectKind, u64); 40] = [
    (EffectKind::Speed, 0x9b6d67cb1ac900b5),
    (EffectKind::Slowness, 0x9b6d67cb1ac900b5),
    (EffectKind::Haste, 0x9b6d67cb1ac900b5),
    (EffectKind::MiningFatigue, 0x9b6d67cb1ac900b5),
    (EffectKind::Strength, 0x9b6d67cb1ac900b5),
    (EffectKind::InstantHealth, 0x2a6d03de656ee7f5),
    (EffectKind::InstantDamage, 0x2a6d03de656ee7f5),
    (EffectKind::JumpBoost, 0x9b6d67cb1ac900b5),
    (EffectKind::Nausea, 0x9b6d67cb1ac900b5),
    (EffectKind::Regeneration, 0x118cbad781a38731),
    (EffectKind::Resistance, 0x9b6d67cb1ac900b5),
    (EffectKind::FireResistance, 0x9b6d67cb1ac900b5),
    (EffectKind::WaterBreathing, 0x9b6d67cb1ac900b5),
    (EffectKind::Invisibility, 0x9b6d67cb1ac900b5),
    (EffectKind::Blindness, 0x9b6d67cb1ac900b5),
    (EffectKind::NightVision, 0x9b6d67cb1ac900b5),
    (EffectKind::Hunger, 0x2432f031254c8a59),
    (EffectKind::Weakness, 0x9b6d67cb1ac900b5),
    (EffectKind::Poison, 0x6ae5c256a8e0f0ea),
    (EffectKind::Wither, 0x3bd5e2388227b0a6),
    (EffectKind::HealthBoost, 0x9b6d67cb1ac900b5),
    (EffectKind::Absorption, 0x2432f031254c8a59),
    (EffectKind::Saturation, 0x2a6d03de656ee7f5),
    (EffectKind::Glowing, 0x9b6d67cb1ac900b5),
    (EffectKind::Levitation, 0x9b6d67cb1ac900b5),
    (EffectKind::Luck, 0x9b6d67cb1ac900b5),
    (EffectKind::Unluck, 0x9b6d67cb1ac900b5),
    (EffectKind::SlowFalling, 0x9b6d67cb1ac900b5),
    (EffectKind::ConduitPower, 0x9b6d67cb1ac900b5),
    (EffectKind::DolphinsGrace, 0x9b6d67cb1ac900b5),
    (EffectKind::BadOmen, 0x2432f031254c8a59),
    (EffectKind::HeroOfTheVillage, 0x9b6d67cb1ac900b5),
    (EffectKind::Darkness, 0x9b6d67cb1ac900b5),
    (EffectKind::TrialOmen, 0x9b6d67cb1ac900b5),
    (EffectKind::RaidOmen, 0xf529d517daf7e469),
    (EffectKind::WindCharged, 0x9b6d67cb1ac900b5),
    (EffectKind::Weaving, 0x9b6d67cb1ac900b5),
    (EffectKind::Oozing, 0x9b6d67cb1ac900b5),
    (EffectKind::Infested, 0x9b6d67cb1ac900b5),
    (EffectKind::BreathOfTheNautilus, 0x9b6d67cb1ac900b5),
];

fn fingerprint(kind: EffectKind) -> u64 {
    let (first, last) = DURATIONS;
    AMPLIFIERS
        .iter()
        .fold(0xcbf29ce484222325, |hash, &amplifier| {
            (first..=last).fold(hash, |hash, duration| {
                let sample = if kind.should_tick(duration, amplifier) {
                    b'1'
                } else {
                    b'0'
                };
                (hash ^ u64::from(sample)).wrapping_mul(0x00000100000001b3)
            })
        })
}

#[test]
fn should_tick_matches_vanilla_samples() {
    for (kind, expected) in FINGERPRINTS {
        assert_eq!((kind, fingerprint(kind)), (kind, expected));
    }
}
