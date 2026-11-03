use rspigot_plugin::Color;
use rspigot_plugin::keys::{
    BlockType, EffectType, Enchantment, EntityType, Material, Particle, Sound,
};

#[test]
fn every_generated_key_is_in_the_server_registries() {
    for (registry, keys) in [
        (
            "minecraft:item",
            Material::ALL.iter().map(|k| k.key()).collect::<Vec<_>>(),
        ),
        (
            "minecraft:block",
            BlockType::ALL.iter().map(|k| k.key()).collect(),
        ),
        (
            "minecraft:mob_effect",
            EffectType::ALL.iter().map(|k| k.key()).collect(),
        ),
        (
            "minecraft:entity_type",
            EntityType::ALL.iter().map(|k| k.key()).collect(),
        ),
        (
            "minecraft:sound_event",
            Sound::ALL.iter().map(|k| k.key()).collect(),
        ),
        (
            "minecraft:particle_type",
            Particle::ALL.iter().map(|k| k.key()).collect(),
        ),
    ] {
        for key in keys {
            assert!(
                rspigot_data::registries::static_index(registry, key).is_some(),
                "{key}"
            );
        }
    }
    for e in Enchantment::ALL {
        assert!(rspigot_data::registries::synced_index("minecraft:enchantment", e.key()).is_some());
    }
    assert_eq!(
        (
            Material::ALL.len(),
            BlockType::ALL.len(),
            EffectType::ALL.len(),
            EntityType::ALL.len(),
            Sound::ALL.len(),
            Enchantment::ALL.len(),
            Particle::ALL.len()
        ),
        (1658, 1286, 40, 161, 1991, 43, 106)
    );
}

#[test]
fn keys_use_spigot_names() {
    assert_eq!(Material::DiamondSword.key(), "minecraft:diamond_sword");
    assert_eq!(Sound::EntityCowHurt.key(), "minecraft:entity.cow.hurt");
    assert_eq!(
        (
            Particle::HappyVillager.key(),
            Enchantment::Sharpness.key(),
            EffectType::Speed.key(),
            EntityType::Zombie.key(),
            BlockType::Stone.key()
        ),
        (
            "minecraft:happy_villager",
            "minecraft:sharpness",
            "minecraft:speed",
            "minecraft:zombie",
            "minecraft:stone"
        )
    );
    assert_eq!(
        Material::from_key("minecraft:diamond"),
        Some(Material::Diamond)
    );
    assert_eq!(Material::from_key("diamond"), Some(Material::Diamond));
    assert_eq!(Material::from_key("minecraft:nope"), None);
    assert_eq!(Particle::from_key("minecraft:dust"), None);
    assert_eq!(
        Particle::Dust {
            color: Color::rgb(255, 0, 0),
            scale: 1.0
        }
        .key(),
        "minecraft:dust"
    );
    assert_eq!(
        Particle::FallingDust(BlockType::Sand).key(),
        "minecraft:falling_dust"
    );
}

fn reads_back<T: Copy + PartialEq + std::fmt::Debug>(
    all: &[T],
    key: fn(T) -> &'static str,
    from_key: fn(&str) -> Option<T>,
) {
    for &k in all {
        assert_eq!(from_key(key(k)), Some(k));
    }
}

#[test]
fn from_key_finds_every_generated_key() {
    reads_back(Material::ALL, Material::key, Material::from_key);
    reads_back(BlockType::ALL, BlockType::key, BlockType::from_key);
    reads_back(EffectType::ALL, EffectType::key, EffectType::from_key);
    reads_back(EntityType::ALL, EntityType::key, EntityType::from_key);
    reads_back(Sound::ALL, Sound::key, Sound::from_key);
    reads_back(Enchantment::ALL, Enchantment::key, Enchantment::from_key);
    reads_back(Particle::ALL, Particle::key, Particle::from_key);
}
