pub mod blocks;
pub mod entities;
#[rustfmt::skip]
mod generated;
pub mod items;
pub mod registries;
pub mod tags;

pub const PROTOCOL_VERSION: i32 = 777;
pub const VERSION_NAME: &str = "26.3";
pub const WORLD_VERSION: i32 = 5023;
pub const KNOWN_PACK: KnownPack = KnownPack {
    namespace: "minecraft",
    id: "core",
    version: "26.3",
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KnownPack {
    pub namespace: &'static str,
    pub id: &'static str,
    pub version: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    const STONE_ITEM_ID: u16 = 1;
    const STONE_BLOCK_ID: u32 = 1;
    const PIG_ENTITY_ID: u16 = 101;

    #[test]
    fn version_constants() {
        assert_eq!(PROTOCOL_VERSION, 777);
        assert_eq!(VERSION_NAME, "26.3");
        assert_eq!(WORLD_VERSION, 5023);
        assert_eq!(KNOWN_PACK.namespace, "minecraft");
        assert_eq!(KNOWN_PACK.id, "core");
        assert_eq!(KNOWN_PACK.version, "26.3");
    }

    #[test]
    fn air_states() {
        assert!(blocks::is_air(0));
        assert!(blocks::is_air(18649));
        assert!(blocks::is_air(18650));
        assert!(!blocks::is_air(1));
        assert!(!blocks::is_air(18648));
        assert!(!blocks::is_air(18651));
        assert_eq!(blocks::block_name(0), Some("minecraft:air"));
        assert_eq!(blocks::block_name(18649), Some("minecraft:void_air"));
        assert_eq!(blocks::block_name(18650), Some("minecraft:cave_air"));
    }

    #[test]
    fn default_states() {
        assert_eq!(blocks::default_state("minecraft:stone"), Some(1));
        assert_eq!(blocks::default_state("minecraft:grass_block"), Some(9));
        assert_eq!(blocks::default_state("minecraft:oak_log"), Some(140));
        assert_eq!(blocks::default_state("minecraft:nope"), None);
        assert_eq!(blocks::default_state("stone"), None);
    }

    #[test]
    fn state_with_overrides_only_given_properties() {
        assert_eq!(
            blocks::state_with("minecraft:oak_log", &[("axis", "x")]),
            Some(139)
        );
        assert_eq!(blocks::state_with("minecraft:oak_log", &[]), Some(140));
        assert_eq!(
            blocks::state_with("minecraft:oak_log", &[("axis", "w")]),
            None
        );
        assert_eq!(
            blocks::state_with("minecraft:oak_log", &[("facing", "x")]),
            None
        );
        assert_eq!(blocks::state_with("minecraft:nope", &[]), None);
    }

    #[test]
    fn property_access() {
        assert_eq!(blocks::property(140, "axis"), Some("y"));
        assert_eq!(blocks::property(139, "axis"), Some("x"));
        assert_eq!(blocks::property(140, "facing"), None);
        assert_eq!(blocks::properties(140), vec![("axis", "y")]);
        assert_eq!(blocks::properties(1), Vec::<(&str, &str)>::new());
        assert_eq!(blocks::with_property(140, "axis", "z"), Some(141));
        assert_eq!(blocks::with_property(140, "axis", "q"), None);
        assert_eq!(blocks::with_property(140, "nope", "z"), None);
    }

    #[test]
    fn multi_property_blocks_match_reference_ids() {
        assert_eq!(blocks::default_state("minecraft:chest"), Some(5544));
        assert_eq!(
            blocks::state_with(
                "minecraft:chest",
                &[
                    ("waterlogged", "false"),
                    ("facing", "east"),
                    ("type", "left")
                ]
            ),
            Some(5564)
        );
        assert_eq!(
            blocks::state_with(
                "minecraft:oak_stairs",
                &[
                    ("facing", "south"),
                    ("half", "top"),
                    ("shape", "inner_left"),
                    ("waterlogged", "true")
                ]
            ),
            Some(5485)
        );
        assert_eq!(
            blocks::state_with(
                "minecraft:redstone_wire",
                &[
                    ("east", "side"),
                    ("north", "up"),
                    ("power", "7"),
                    ("south", "none"),
                    ("west", "side")
                ]
            ),
            Some(6069)
        );
        assert_eq!(
            blocks::properties(5564),
            vec![
                ("facing", "east"),
                ("type", "left"),
                ("waterlogged", "false")
            ]
        );
        assert_eq!(blocks::property(5564, "type"), Some("left"));
        assert_eq!(blocks::with_property(5544, "facing", "east"), Some(5562));
    }

    #[test]
    fn out_of_range_states() {
        assert_eq!(blocks::block_name(blocks::STATE_COUNT as u16), None);
        assert_eq!(blocks::block_id(u16::MAX), None);
        assert_eq!(blocks::property(u16::MAX, "axis"), None);
        assert_eq!(blocks::with_property(u16::MAX, "axis", "x"), None);
        assert!(blocks::properties(u16::MAX).is_empty());
        assert!(!blocks::is_air(u16::MAX));
    }

    #[test]
    fn counts() {
        assert_eq!(blocks::STATE_COUNT, 35723);
        assert_eq!(blocks::block_count(), 1286);
        assert_eq!(items::count(), 1658);
    }

    #[test]
    fn block_ids() {
        assert_eq!(blocks::block_id(1), Some(STONE_BLOCK_ID as u16));
        assert_eq!(blocks::block_id(0), Some(0));
        assert_eq!(blocks::block_id(139), Some(51));
        assert_eq!(blocks::block_id(141), Some(51));
        assert_eq!(
            registries::static_index("minecraft:block", "minecraft:oak_log"),
            Some(51)
        );
    }

    #[test]
    fn every_state_round_trips() {
        for state in 0..blocks::STATE_COUNT as u16 {
            let name = blocks::block_name(state).unwrap();
            let props = blocks::properties(state);
            assert_eq!(blocks::state_with(name, &props), Some(state));
            for &(key, value) in &props {
                assert_eq!(blocks::property(state, key), Some(value));
                assert_eq!(blocks::with_property(state, key, value), Some(state));
            }
            let id = blocks::block_id(state).unwrap();
            assert_eq!(
                registries::static_entries("minecraft:block").map(|e| e[usize::from(id)]),
                Some(name)
            );
        }
    }

    #[test]
    fn every_block_default_state_is_its_own() {
        let entries = registries::static_entries("minecraft:block").unwrap();
        assert_eq!(entries.len(), blocks::block_count());
        for name in entries {
            let state = blocks::default_state(name).unwrap();
            assert_eq!(blocks::block_name(state), Some(*name));
        }
    }

    #[test]
    fn biomes() {
        let plains = registries::biome_id("minecraft:plains").unwrap();
        assert_eq!(registries::biome_name(plains), Some("minecraft:plains"));
        assert_eq!(registries::biome_id("minecraft:nope"), None);
        assert_eq!(
            registries::synced_index("minecraft:worldgen/biome", "minecraft:plains"),
            Some(u32::from(plains))
        );
    }

    #[test]
    fn dimension_types() {
        let entries = registries::synced_entries("minecraft:dimension_type").unwrap();
        assert_eq!(entries.len(), 4);
        for (index, name) in entries.iter().enumerate() {
            assert_eq!(
                registries::dimension_type_id(name),
                Some(u32::try_from(index).unwrap())
            );
        }
        assert_eq!(registries::dimension_type_id("minecraft:nope"), None);
    }

    #[test]
    fn synced_registries_follow_send_order() {
        let ids: Vec<&str> = registries::SYNCED.iter().map(|r| r.id).collect();
        assert_eq!(ids.len(), 32);
        assert_eq!(ids[0], "minecraft:worldgen/biome");
        assert_eq!(ids[1], "minecraft:chat_type");
        assert_eq!(ids[18], "minecraft:dimension_type");
        assert_eq!(ids[31], "minecraft:worldgen/block_state_provider");
        for registry in registries::SYNCED {
            assert!(registry.entries.windows(2).all(|w| w[0] < w[1]));
            assert_eq!(
                registries::synced_entries(registry.id),
                Some(registry.entries)
            );
            for (index, entry) in registry.entries.iter().enumerate() {
                assert_eq!(
                    registries::synced_index(registry.id, entry),
                    Some(u32::try_from(index).unwrap())
                );
            }
        }
        assert_eq!(registries::synced_entries("minecraft:block"), None);
        assert_eq!(
            registries::synced_index("minecraft:nope", "minecraft:x"),
            None
        );
    }

    #[test]
    fn static_registries_are_ordered_by_protocol_id() {
        assert_eq!(
            registries::static_index("minecraft:item", "minecraft:stone"),
            Some(u32::from(STONE_ITEM_ID))
        );
        assert_eq!(
            registries::static_index("minecraft:entity_type", "minecraft:pig"),
            Some(u32::from(PIG_ENTITY_ID))
        );
        assert_eq!(
            registries::static_index("minecraft:item", "minecraft:nope"),
            None
        );
        assert_eq!(
            registries::static_index("minecraft:nope", "minecraft:stone"),
            None
        );
        assert_eq!(registries::static_entries("minecraft:nope"), None);
        let items = registries::static_entries("minecraft:item").unwrap();
        assert_eq!(items.len(), items::count());
        assert_eq!(items[1], "minecraft:stone");
        for (index, name) in items.iter().enumerate() {
            assert_eq!(
                registries::static_index("minecraft:item", name),
                Some(u32::try_from(index).unwrap())
            );
        }
    }

    #[test]
    fn items_lookup() {
        assert_eq!(items::id("minecraft:stone"), Some(STONE_ITEM_ID));
        assert_eq!(items::name(STONE_ITEM_ID), Some("minecraft:stone"));
        assert_eq!(items::id("minecraft:nope"), None);
        assert_eq!(items::name(u16::MAX), None);
        assert_eq!(items::id("minecraft:air"), Some(0));
        for id in 0..items::count() as u16 {
            let name = items::name(id).unwrap();
            assert_eq!(items::id(name), Some(id));
        }
    }

    #[test]
    fn stack_sizes() {
        let id = |name: &str| items::id(name).unwrap();
        assert_eq!(items::max_stack_size(id("minecraft:stone")), 64);
        assert_eq!(items::max_stack_size(id("minecraft:diamond_sword")), 1);
        assert_eq!(items::max_stack_size(id("minecraft:ender_pearl")), 16);
        assert_eq!(items::max_stack_size(id("minecraft:snowball")), 16);
    }

    #[test]
    fn block_state_for_items() {
        let id = |name: &str| items::id(name).unwrap();
        assert_eq!(items::block_state_for(id("minecraft:stone")), Some(1));
        assert_eq!(
            items::block_state_for(id("minecraft:oak_log")),
            blocks::default_state("minecraft:oak_log")
        );
        assert_eq!(
            items::block_state_for(id("minecraft:redstone")),
            blocks::default_state("minecraft:redstone_wire")
        );
        assert_eq!(
            items::block_state_for(id("minecraft:string")),
            blocks::default_state("minecraft:tripwire")
        );
        assert_eq!(
            items::block_state_for(id("minecraft:wheat_seeds")),
            blocks::default_state("minecraft:wheat")
        );
        assert_eq!(
            items::block_state_for(id("minecraft:carrot")),
            blocks::default_state("minecraft:carrots")
        );
        assert_eq!(items::block_state_for(id("minecraft:diamond_sword")), None);
        assert_eq!(items::block_state_for(id("minecraft:wheat")), None);
        assert_eq!(items::block_state_for(u16::MAX), None);
    }

    #[test]
    fn entities_lookup() {
        assert_eq!(entities::id("minecraft:pig"), Some(PIG_ENTITY_ID));
        assert_eq!(entities::name(PIG_ENTITY_ID), Some("minecraft:pig"));
        assert_eq!(entities::id("minecraft:nope"), None);
        assert_eq!(entities::name(u16::MAX), None);
    }

    fn tag_ids(registry: &str, tag: &str) -> &'static [u32] {
        let registry = tags::TAGS.iter().find(|r| r.registry == registry).unwrap();
        registry
            .tags
            .iter()
            .find(|(name, _)| *name == tag)
            .map(|(_, ids)| *ids)
            .unwrap()
    }

    #[test]
    fn block_tags_contain_pickaxe_with_stone() {
        let ids = tag_ids("minecraft:block", "minecraft:mineable/pickaxe");
        assert!(ids.contains(&STONE_BLOCK_ID));
        assert!(ids.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn tags_expand_nested_references() {
        let air = tag_ids("minecraft:block", "minecraft:air");
        let id = |name: &str| registries::static_index("minecraft:block", name).unwrap();
        assert_eq!(
            air,
            &[
                id("minecraft:air"),
                id("minecraft:void_air"),
                id("minecraft:cave_air")
            ]
        );
        let logs = tag_ids("minecraft:block", "minecraft:logs");
        assert!(logs.contains(&id("minecraft:oak_log")));
        assert!(logs.contains(&id("minecraft:crimson_stem")));
        assert!(!logs.contains(&id("minecraft:stone")));
    }

    #[test]
    fn tags_cover_static_and_synced_registries_only() {
        let registries: Vec<&str> = tags::TAGS.iter().map(|r| r.registry).collect();
        assert!(registries.contains(&"minecraft:block"));
        assert!(registries.contains(&"minecraft:item"));
        assert!(registries.contains(&"minecraft:worldgen/biome"));
        assert!(registries.contains(&"minecraft:damage_type"));
        assert!(!registries.contains(&"minecraft:villager_trade"));
        assert!(!registries.contains(&"minecraft:worldgen/structure"));
        assert!(registries.windows(2).all(|w| w[0] < w[1]));
        for registry in tags::TAGS {
            let size = registries::static_entries(registry.registry)
                .map(<[&str]>::len)
                .or_else(|| registries::synced_entries(registry.registry).map(<[&str]>::len))
                .unwrap();
            assert!(registry.tags.windows(2).all(|w| w[0].0 < w[1].0));
            for (_, ids) in registry.tags {
                assert!(ids.windows(2).all(|w| w[0] < w[1]));
                assert!(ids.iter().all(|id| (*id as usize) < size));
            }
        }
    }
}
