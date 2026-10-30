use crate::transformers::{Condition, Group, Rule, Then};

pub(crate) static AXE: [Group; 7] = [
    Group {
        sound: "minecraft:item.axe.strip",
        particle: None,
        transform_type: None,
        update_from_neighbors: true,
        damage: 1,
        disallowed_faces: &[],
        drop_strategy: None,
        loot: None,
        rules: &[
            Rule {
                when: Condition::Block("minecraft:oak_wood"),
                then: Then::Copy { to: "minecraft:stripped_oak_wood", defaults: &[("axis", "y")] },
            },
            Rule {
                when: Condition::Block("minecraft:oak_log"),
                then: Then::Copy { to: "minecraft:stripped_oak_log", defaults: &[("axis", "y")] },
            },
            Rule {
                when: Condition::Block("minecraft:dark_oak_wood"),
                then: Then::Copy {
                    to: "minecraft:stripped_dark_oak_wood",
                    defaults: &[("axis", "y")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:dark_oak_log"),
                then: Then::Copy {
                    to: "minecraft:stripped_dark_oak_log",
                    defaults: &[("axis", "y")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:pale_oak_wood"),
                then: Then::Copy {
                    to: "minecraft:stripped_pale_oak_wood",
                    defaults: &[("axis", "y")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:pale_oak_log"),
                then: Then::Copy {
                    to: "minecraft:stripped_pale_oak_log",
                    defaults: &[("axis", "y")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:poplar_wood"),
                then: Then::Copy {
                    to: "minecraft:stripped_poplar_wood",
                    defaults: &[("axis", "y")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:poplar_log"),
                then: Then::Copy {
                    to: "minecraft:stripped_poplar_log",
                    defaults: &[("axis", "y")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:acacia_wood"),
                then: Then::Copy {
                    to: "minecraft:stripped_acacia_wood",
                    defaults: &[("axis", "y")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:acacia_log"),
                then: Then::Copy {
                    to: "minecraft:stripped_acacia_log",
                    defaults: &[("axis", "y")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:cherry_wood"),
                then: Then::Copy {
                    to: "minecraft:stripped_cherry_wood",
                    defaults: &[("axis", "y")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:cherry_log"),
                then: Then::Copy {
                    to: "minecraft:stripped_cherry_log",
                    defaults: &[("axis", "y")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:birch_wood"),
                then: Then::Copy {
                    to: "minecraft:stripped_birch_wood",
                    defaults: &[("axis", "y")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:birch_log"),
                then: Then::Copy { to: "minecraft:stripped_birch_log", defaults: &[("axis", "y")] },
            },
            Rule {
                when: Condition::Block("minecraft:jungle_wood"),
                then: Then::Copy {
                    to: "minecraft:stripped_jungle_wood",
                    defaults: &[("axis", "y")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:jungle_log"),
                then: Then::Copy {
                    to: "minecraft:stripped_jungle_log",
                    defaults: &[("axis", "y")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:spruce_wood"),
                then: Then::Copy {
                    to: "minecraft:stripped_spruce_wood",
                    defaults: &[("axis", "y")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:spruce_log"),
                then: Then::Copy {
                    to: "minecraft:stripped_spruce_log",
                    defaults: &[("axis", "y")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:warped_stem"),
                then: Then::Copy {
                    to: "minecraft:stripped_warped_stem",
                    defaults: &[("axis", "y")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:warped_hyphae"),
                then: Then::Copy {
                    to: "minecraft:stripped_warped_hyphae",
                    defaults: &[("axis", "y")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:crimson_stem"),
                then: Then::Copy {
                    to: "minecraft:stripped_crimson_stem",
                    defaults: &[("axis", "y")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:crimson_hyphae"),
                then: Then::Copy {
                    to: "minecraft:stripped_crimson_hyphae",
                    defaults: &[("axis", "y")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:mangrove_wood"),
                then: Then::Copy {
                    to: "minecraft:stripped_mangrove_wood",
                    defaults: &[("axis", "y")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:mangrove_log"),
                then: Then::Copy {
                    to: "minecraft:stripped_mangrove_log",
                    defaults: &[("axis", "y")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:bamboo_block"),
                then: Then::Copy {
                    to: "minecraft:stripped_bamboo_block",
                    defaults: &[("axis", "y")],
                },
            },
        ],
    },
    Group {
        sound: "minecraft:item.axe.scrape",
        particle: Some("scrape"),
        transform_type: None,
        update_from_neighbors: true,
        damage: 1,
        disallowed_faces: &[],
        drop_strategy: None,
        loot: None,
        rules: &[
            Rule {
                when: Condition::Block("minecraft:exposed_copper"),
                then: Then::Copy { to: "minecraft:copper_block", defaults: &[] },
            },
            Rule {
                when: Condition::Block("minecraft:weathered_copper"),
                then: Then::Copy { to: "minecraft:exposed_copper", defaults: &[] },
            },
            Rule {
                when: Condition::Block("minecraft:oxidized_copper"),
                then: Then::Copy { to: "minecraft:weathered_copper", defaults: &[] },
            },
            Rule {
                when: Condition::Block("minecraft:exposed_cut_copper"),
                then: Then::Copy { to: "minecraft:cut_copper", defaults: &[] },
            },
            Rule {
                when: Condition::Block("minecraft:weathered_cut_copper"),
                then: Then::Copy { to: "minecraft:exposed_cut_copper", defaults: &[] },
            },
            Rule {
                when: Condition::Block("minecraft:oxidized_cut_copper"),
                then: Then::Copy { to: "minecraft:weathered_cut_copper", defaults: &[] },
            },
            Rule {
                when: Condition::Block("minecraft:exposed_chiseled_copper"),
                then: Then::Copy { to: "minecraft:chiseled_copper", defaults: &[] },
            },
            Rule {
                when: Condition::Block("minecraft:weathered_chiseled_copper"),
                then: Then::Copy { to: "minecraft:exposed_chiseled_copper", defaults: &[] },
            },
            Rule {
                when: Condition::Block("minecraft:oxidized_chiseled_copper"),
                then: Then::Copy { to: "minecraft:weathered_chiseled_copper", defaults: &[] },
            },
            Rule {
                when: Condition::Block("minecraft:exposed_cut_copper_slab"),
                then: Then::Copy {
                    to: "minecraft:cut_copper_slab",
                    defaults: &[("type", "bottom"), ("waterlogged", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:weathered_cut_copper_slab"),
                then: Then::Copy {
                    to: "minecraft:exposed_cut_copper_slab",
                    defaults: &[("type", "bottom"), ("waterlogged", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:oxidized_cut_copper_slab"),
                then: Then::Copy {
                    to: "minecraft:weathered_cut_copper_slab",
                    defaults: &[("type", "bottom"), ("waterlogged", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:exposed_cut_copper_stairs"),
                then: Then::Copy {
                    to: "minecraft:cut_copper_stairs",
                    defaults: &[
                        ("facing", "north"),
                        ("half", "bottom"),
                        ("shape", "straight"),
                        ("waterlogged", "false"),
                    ],
                },
            },
            Rule {
                when: Condition::Block("minecraft:weathered_cut_copper_stairs"),
                then: Then::Copy {
                    to: "minecraft:exposed_cut_copper_stairs",
                    defaults: &[
                        ("facing", "north"),
                        ("half", "bottom"),
                        ("shape", "straight"),
                        ("waterlogged", "false"),
                    ],
                },
            },
            Rule {
                when: Condition::Block("minecraft:oxidized_cut_copper_stairs"),
                then: Then::Copy {
                    to: "minecraft:weathered_cut_copper_stairs",
                    defaults: &[
                        ("facing", "north"),
                        ("half", "bottom"),
                        ("shape", "straight"),
                        ("waterlogged", "false"),
                    ],
                },
            },
            Rule {
                when: Condition::Block("minecraft:exposed_copper_trapdoor"),
                then: Then::Copy {
                    to: "minecraft:copper_trapdoor",
                    defaults: &[
                        ("facing", "north"),
                        ("half", "bottom"),
                        ("open", "false"),
                        ("powered", "false"),
                        ("waterlogged", "false"),
                    ],
                },
            },
            Rule {
                when: Condition::Block("minecraft:weathered_copper_trapdoor"),
                then: Then::Copy {
                    to: "minecraft:exposed_copper_trapdoor",
                    defaults: &[
                        ("facing", "north"),
                        ("half", "bottom"),
                        ("open", "false"),
                        ("powered", "false"),
                        ("waterlogged", "false"),
                    ],
                },
            },
            Rule {
                when: Condition::Block("minecraft:oxidized_copper_trapdoor"),
                then: Then::Copy {
                    to: "minecraft:weathered_copper_trapdoor",
                    defaults: &[
                        ("facing", "north"),
                        ("half", "bottom"),
                        ("open", "false"),
                        ("powered", "false"),
                        ("waterlogged", "false"),
                    ],
                },
            },
            Rule {
                when: Condition::Block("minecraft:exposed_copper_bars"),
                then: Then::Copy {
                    to: "minecraft:copper_bars",
                    defaults: &[
                        ("east", "false"),
                        ("north", "false"),
                        ("south", "false"),
                        ("waterlogged", "false"),
                        ("west", "false"),
                    ],
                },
            },
            Rule {
                when: Condition::Block("minecraft:weathered_copper_bars"),
                then: Then::Copy {
                    to: "minecraft:exposed_copper_bars",
                    defaults: &[
                        ("east", "false"),
                        ("north", "false"),
                        ("south", "false"),
                        ("waterlogged", "false"),
                        ("west", "false"),
                    ],
                },
            },
            Rule {
                when: Condition::Block("minecraft:oxidized_copper_bars"),
                then: Then::Copy {
                    to: "minecraft:weathered_copper_bars",
                    defaults: &[
                        ("east", "false"),
                        ("north", "false"),
                        ("south", "false"),
                        ("waterlogged", "false"),
                        ("west", "false"),
                    ],
                },
            },
            Rule {
                when: Condition::Block("minecraft:exposed_copper_grate"),
                then: Then::Copy {
                    to: "minecraft:copper_grate",
                    defaults: &[("waterlogged", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:weathered_copper_grate"),
                then: Then::Copy {
                    to: "minecraft:exposed_copper_grate",
                    defaults: &[("waterlogged", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:oxidized_copper_grate"),
                then: Then::Copy {
                    to: "minecraft:weathered_copper_grate",
                    defaults: &[("waterlogged", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:exposed_copper_bulb"),
                then: Then::Copy {
                    to: "minecraft:copper_bulb",
                    defaults: &[("lit", "false"), ("powered", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:weathered_copper_bulb"),
                then: Then::Copy {
                    to: "minecraft:exposed_copper_bulb",
                    defaults: &[("lit", "false"), ("powered", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:oxidized_copper_bulb"),
                then: Then::Copy {
                    to: "minecraft:weathered_copper_bulb",
                    defaults: &[("lit", "false"), ("powered", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:exposed_copper_lantern"),
                then: Then::Copy {
                    to: "minecraft:copper_lantern",
                    defaults: &[("hanging", "false"), ("waterlogged", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:weathered_copper_lantern"),
                then: Then::Copy {
                    to: "minecraft:exposed_copper_lantern",
                    defaults: &[("hanging", "false"), ("waterlogged", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:oxidized_copper_lantern"),
                then: Then::Copy {
                    to: "minecraft:weathered_copper_lantern",
                    defaults: &[("hanging", "false"), ("waterlogged", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:exposed_copper_golem_statue"),
                then: Then::Copy {
                    to: "minecraft:copper_golem_statue",
                    defaults: &[
                        ("copper_golem_pose", "standing"),
                        ("facing", "north"),
                        ("waterlogged", "false"),
                    ],
                },
            },
            Rule {
                when: Condition::Block("minecraft:weathered_copper_golem_statue"),
                then: Then::Copy {
                    to: "minecraft:exposed_copper_golem_statue",
                    defaults: &[
                        ("copper_golem_pose", "standing"),
                        ("facing", "north"),
                        ("waterlogged", "false"),
                    ],
                },
            },
            Rule {
                when: Condition::Block("minecraft:oxidized_copper_golem_statue"),
                then: Then::Copy {
                    to: "minecraft:weathered_copper_golem_statue",
                    defaults: &[
                        ("copper_golem_pose", "standing"),
                        ("facing", "north"),
                        ("waterlogged", "false"),
                    ],
                },
            },
            Rule {
                when: Condition::Block("minecraft:exposed_lightning_rod"),
                then: Then::Copy {
                    to: "minecraft:lightning_rod",
                    defaults: &[("facing", "up"), ("powered", "false"), ("waterlogged", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:weathered_lightning_rod"),
                then: Then::Copy {
                    to: "minecraft:exposed_lightning_rod",
                    defaults: &[("facing", "up"), ("powered", "false"), ("waterlogged", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:oxidized_lightning_rod"),
                then: Then::Copy {
                    to: "minecraft:weathered_lightning_rod",
                    defaults: &[("facing", "up"), ("powered", "false"), ("waterlogged", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:exposed_copper_chain"),
                then: Then::Copy {
                    to: "minecraft:copper_chain",
                    defaults: &[("axis", "y"), ("waterlogged", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:weathered_copper_chain"),
                then: Then::Copy {
                    to: "minecraft:exposed_copper_chain",
                    defaults: &[("axis", "y"), ("waterlogged", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:oxidized_copper_chain"),
                then: Then::Copy {
                    to: "minecraft:weathered_copper_chain",
                    defaults: &[("axis", "y"), ("waterlogged", "false")],
                },
            },
        ],
    },
    Group {
        sound: "minecraft:item.axe.scrape",
        particle: Some("scrape"),
        transform_type: Some("copper_chest"),
        update_from_neighbors: false,
        damage: 1,
        disallowed_faces: &[],
        drop_strategy: None,
        loot: None,
        rules: &[
            Rule {
                when: Condition::Block("minecraft:exposed_copper_chest"),
                then: Then::Copy {
                    to: "minecraft:copper_chest",
                    defaults: &[("facing", "north"), ("type", "single"), ("waterlogged", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:weathered_copper_chest"),
                then: Then::Copy {
                    to: "minecraft:exposed_copper_chest",
                    defaults: &[("facing", "north"), ("type", "single"), ("waterlogged", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:oxidized_copper_chest"),
                then: Then::Copy {
                    to: "minecraft:weathered_copper_chest",
                    defaults: &[("facing", "north"), ("type", "single"), ("waterlogged", "false")],
                },
            },
        ],
    },
    Group {
        sound: "minecraft:item.axe.scrape",
        particle: Some("scrape"),
        transform_type: None,
        update_from_neighbors: false,
        damage: 1,
        disallowed_faces: &[],
        drop_strategy: None,
        loot: None,
        rules: &[
            Rule {
                when: Condition::Block("minecraft:exposed_copper_door"),
                then: Then::Copy {
                    to: "minecraft:copper_door",
                    defaults: &[
                        ("facing", "north"),
                        ("half", "lower"),
                        ("hinge", "left"),
                        ("open", "false"),
                        ("powered", "false"),
                    ],
                },
            },
            Rule {
                when: Condition::Block("minecraft:weathered_copper_door"),
                then: Then::Copy {
                    to: "minecraft:exposed_copper_door",
                    defaults: &[
                        ("facing", "north"),
                        ("half", "lower"),
                        ("hinge", "left"),
                        ("open", "false"),
                        ("powered", "false"),
                    ],
                },
            },
            Rule {
                when: Condition::Block("minecraft:oxidized_copper_door"),
                then: Then::Copy {
                    to: "minecraft:weathered_copper_door",
                    defaults: &[
                        ("facing", "north"),
                        ("half", "lower"),
                        ("hinge", "left"),
                        ("open", "false"),
                        ("powered", "false"),
                    ],
                },
            },
        ],
    },
    Group {
        sound: "minecraft:item.axe.wax_off",
        particle: Some("wax_off"),
        transform_type: None,
        update_from_neighbors: true,
        damage: 1,
        disallowed_faces: &[],
        drop_strategy: None,
        loot: None,
        rules: &[
            Rule {
                when: Condition::Block("minecraft:waxed_copper_block"),
                then: Then::Copy { to: "minecraft:copper_block", defaults: &[] },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_exposed_copper"),
                then: Then::Copy { to: "minecraft:exposed_copper", defaults: &[] },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_weathered_copper"),
                then: Then::Copy { to: "minecraft:weathered_copper", defaults: &[] },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_oxidized_copper"),
                then: Then::Copy { to: "minecraft:oxidized_copper", defaults: &[] },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_cut_copper"),
                then: Then::Copy { to: "minecraft:cut_copper", defaults: &[] },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_exposed_cut_copper"),
                then: Then::Copy { to: "minecraft:exposed_cut_copper", defaults: &[] },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_weathered_cut_copper"),
                then: Then::Copy { to: "minecraft:weathered_cut_copper", defaults: &[] },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_oxidized_cut_copper"),
                then: Then::Copy { to: "minecraft:oxidized_cut_copper", defaults: &[] },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_cut_copper_slab"),
                then: Then::Copy {
                    to: "minecraft:cut_copper_slab",
                    defaults: &[("type", "bottom"), ("waterlogged", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_exposed_cut_copper_slab"),
                then: Then::Copy {
                    to: "minecraft:exposed_cut_copper_slab",
                    defaults: &[("type", "bottom"), ("waterlogged", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_weathered_cut_copper_slab"),
                then: Then::Copy {
                    to: "minecraft:weathered_cut_copper_slab",
                    defaults: &[("type", "bottom"), ("waterlogged", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_oxidized_cut_copper_slab"),
                then: Then::Copy {
                    to: "minecraft:oxidized_cut_copper_slab",
                    defaults: &[("type", "bottom"), ("waterlogged", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_cut_copper_stairs"),
                then: Then::Copy {
                    to: "minecraft:cut_copper_stairs",
                    defaults: &[
                        ("facing", "north"),
                        ("half", "bottom"),
                        ("shape", "straight"),
                        ("waterlogged", "false"),
                    ],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_exposed_cut_copper_stairs"),
                then: Then::Copy {
                    to: "minecraft:exposed_cut_copper_stairs",
                    defaults: &[
                        ("facing", "north"),
                        ("half", "bottom"),
                        ("shape", "straight"),
                        ("waterlogged", "false"),
                    ],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_weathered_cut_copper_stairs"),
                then: Then::Copy {
                    to: "minecraft:weathered_cut_copper_stairs",
                    defaults: &[
                        ("facing", "north"),
                        ("half", "bottom"),
                        ("shape", "straight"),
                        ("waterlogged", "false"),
                    ],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_oxidized_cut_copper_stairs"),
                then: Then::Copy {
                    to: "minecraft:oxidized_cut_copper_stairs",
                    defaults: &[
                        ("facing", "north"),
                        ("half", "bottom"),
                        ("shape", "straight"),
                        ("waterlogged", "false"),
                    ],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_chiseled_copper"),
                then: Then::Copy { to: "minecraft:chiseled_copper", defaults: &[] },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_exposed_chiseled_copper"),
                then: Then::Copy { to: "minecraft:exposed_chiseled_copper", defaults: &[] },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_weathered_chiseled_copper"),
                then: Then::Copy { to: "minecraft:weathered_chiseled_copper", defaults: &[] },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_oxidized_chiseled_copper"),
                then: Then::Copy { to: "minecraft:oxidized_chiseled_copper", defaults: &[] },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_copper_trapdoor"),
                then: Then::Copy {
                    to: "minecraft:copper_trapdoor",
                    defaults: &[
                        ("facing", "north"),
                        ("half", "bottom"),
                        ("open", "false"),
                        ("powered", "false"),
                        ("waterlogged", "false"),
                    ],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_exposed_copper_trapdoor"),
                then: Then::Copy {
                    to: "minecraft:exposed_copper_trapdoor",
                    defaults: &[
                        ("facing", "north"),
                        ("half", "bottom"),
                        ("open", "false"),
                        ("powered", "false"),
                        ("waterlogged", "false"),
                    ],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_weathered_copper_trapdoor"),
                then: Then::Copy {
                    to: "minecraft:weathered_copper_trapdoor",
                    defaults: &[
                        ("facing", "north"),
                        ("half", "bottom"),
                        ("open", "false"),
                        ("powered", "false"),
                        ("waterlogged", "false"),
                    ],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_oxidized_copper_trapdoor"),
                then: Then::Copy {
                    to: "minecraft:oxidized_copper_trapdoor",
                    defaults: &[
                        ("facing", "north"),
                        ("half", "bottom"),
                        ("open", "false"),
                        ("powered", "false"),
                        ("waterlogged", "false"),
                    ],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_copper_bars"),
                then: Then::Copy {
                    to: "minecraft:copper_bars",
                    defaults: &[
                        ("east", "false"),
                        ("north", "false"),
                        ("south", "false"),
                        ("waterlogged", "false"),
                        ("west", "false"),
                    ],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_exposed_copper_bars"),
                then: Then::Copy {
                    to: "minecraft:exposed_copper_bars",
                    defaults: &[
                        ("east", "false"),
                        ("north", "false"),
                        ("south", "false"),
                        ("waterlogged", "false"),
                        ("west", "false"),
                    ],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_weathered_copper_bars"),
                then: Then::Copy {
                    to: "minecraft:weathered_copper_bars",
                    defaults: &[
                        ("east", "false"),
                        ("north", "false"),
                        ("south", "false"),
                        ("waterlogged", "false"),
                        ("west", "false"),
                    ],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_oxidized_copper_bars"),
                then: Then::Copy {
                    to: "minecraft:oxidized_copper_bars",
                    defaults: &[
                        ("east", "false"),
                        ("north", "false"),
                        ("south", "false"),
                        ("waterlogged", "false"),
                        ("west", "false"),
                    ],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_copper_grate"),
                then: Then::Copy {
                    to: "minecraft:copper_grate",
                    defaults: &[("waterlogged", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_exposed_copper_grate"),
                then: Then::Copy {
                    to: "minecraft:exposed_copper_grate",
                    defaults: &[("waterlogged", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_weathered_copper_grate"),
                then: Then::Copy {
                    to: "minecraft:weathered_copper_grate",
                    defaults: &[("waterlogged", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_oxidized_copper_grate"),
                then: Then::Copy {
                    to: "minecraft:oxidized_copper_grate",
                    defaults: &[("waterlogged", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_copper_bulb"),
                then: Then::Copy {
                    to: "minecraft:copper_bulb",
                    defaults: &[("lit", "false"), ("powered", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_exposed_copper_bulb"),
                then: Then::Copy {
                    to: "minecraft:exposed_copper_bulb",
                    defaults: &[("lit", "false"), ("powered", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_weathered_copper_bulb"),
                then: Then::Copy {
                    to: "minecraft:weathered_copper_bulb",
                    defaults: &[("lit", "false"), ("powered", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_oxidized_copper_bulb"),
                then: Then::Copy {
                    to: "minecraft:oxidized_copper_bulb",
                    defaults: &[("lit", "false"), ("powered", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_copper_golem_statue"),
                then: Then::Copy {
                    to: "minecraft:copper_golem_statue",
                    defaults: &[
                        ("copper_golem_pose", "standing"),
                        ("facing", "north"),
                        ("waterlogged", "false"),
                    ],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_exposed_copper_golem_statue"),
                then: Then::Copy {
                    to: "minecraft:exposed_copper_golem_statue",
                    defaults: &[
                        ("copper_golem_pose", "standing"),
                        ("facing", "north"),
                        ("waterlogged", "false"),
                    ],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_weathered_copper_golem_statue"),
                then: Then::Copy {
                    to: "minecraft:weathered_copper_golem_statue",
                    defaults: &[
                        ("copper_golem_pose", "standing"),
                        ("facing", "north"),
                        ("waterlogged", "false"),
                    ],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_oxidized_copper_golem_statue"),
                then: Then::Copy {
                    to: "minecraft:oxidized_copper_golem_statue",
                    defaults: &[
                        ("copper_golem_pose", "standing"),
                        ("facing", "north"),
                        ("waterlogged", "false"),
                    ],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_lightning_rod"),
                then: Then::Copy {
                    to: "minecraft:lightning_rod",
                    defaults: &[("facing", "up"), ("powered", "false"), ("waterlogged", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_exposed_lightning_rod"),
                then: Then::Copy {
                    to: "minecraft:exposed_lightning_rod",
                    defaults: &[("facing", "up"), ("powered", "false"), ("waterlogged", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_weathered_lightning_rod"),
                then: Then::Copy {
                    to: "minecraft:weathered_lightning_rod",
                    defaults: &[("facing", "up"), ("powered", "false"), ("waterlogged", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_oxidized_lightning_rod"),
                then: Then::Copy {
                    to: "minecraft:oxidized_lightning_rod",
                    defaults: &[("facing", "up"), ("powered", "false"), ("waterlogged", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_copper_lantern"),
                then: Then::Copy {
                    to: "minecraft:copper_lantern",
                    defaults: &[("hanging", "false"), ("waterlogged", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_exposed_copper_lantern"),
                then: Then::Copy {
                    to: "minecraft:exposed_copper_lantern",
                    defaults: &[("hanging", "false"), ("waterlogged", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_weathered_copper_lantern"),
                then: Then::Copy {
                    to: "minecraft:weathered_copper_lantern",
                    defaults: &[("hanging", "false"), ("waterlogged", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_oxidized_copper_lantern"),
                then: Then::Copy {
                    to: "minecraft:oxidized_copper_lantern",
                    defaults: &[("hanging", "false"), ("waterlogged", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_copper_chain"),
                then: Then::Copy {
                    to: "minecraft:copper_chain",
                    defaults: &[("axis", "y"), ("waterlogged", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_exposed_copper_chain"),
                then: Then::Copy {
                    to: "minecraft:exposed_copper_chain",
                    defaults: &[("axis", "y"), ("waterlogged", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_weathered_copper_chain"),
                then: Then::Copy {
                    to: "minecraft:weathered_copper_chain",
                    defaults: &[("axis", "y"), ("waterlogged", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_oxidized_copper_chain"),
                then: Then::Copy {
                    to: "minecraft:oxidized_copper_chain",
                    defaults: &[("axis", "y"), ("waterlogged", "false")],
                },
            },
        ],
    },
    Group {
        sound: "minecraft:item.axe.wax_off",
        particle: Some("wax_off"),
        transform_type: Some("copper_chest"),
        update_from_neighbors: false,
        damage: 1,
        disallowed_faces: &[],
        drop_strategy: None,
        loot: None,
        rules: &[
            Rule {
                when: Condition::Block("minecraft:waxed_copper_chest"),
                then: Then::Copy {
                    to: "minecraft:copper_chest",
                    defaults: &[("facing", "north"), ("type", "single"), ("waterlogged", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_exposed_copper_chest"),
                then: Then::Copy {
                    to: "minecraft:exposed_copper_chest",
                    defaults: &[("facing", "north"), ("type", "single"), ("waterlogged", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_weathered_copper_chest"),
                then: Then::Copy {
                    to: "minecraft:weathered_copper_chest",
                    defaults: &[("facing", "north"), ("type", "single"), ("waterlogged", "false")],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_oxidized_copper_chest"),
                then: Then::Copy {
                    to: "minecraft:oxidized_copper_chest",
                    defaults: &[("facing", "north"), ("type", "single"), ("waterlogged", "false")],
                },
            },
        ],
    },
    Group {
        sound: "minecraft:item.axe.wax_off",
        particle: Some("wax_off"),
        transform_type: None,
        update_from_neighbors: false,
        damage: 1,
        disallowed_faces: &[],
        drop_strategy: None,
        loot: None,
        rules: &[
            Rule {
                when: Condition::Block("minecraft:waxed_copper_door"),
                then: Then::Copy {
                    to: "minecraft:copper_door",
                    defaults: &[
                        ("facing", "north"),
                        ("half", "lower"),
                        ("hinge", "left"),
                        ("open", "false"),
                        ("powered", "false"),
                    ],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_exposed_copper_door"),
                then: Then::Copy {
                    to: "minecraft:exposed_copper_door",
                    defaults: &[
                        ("facing", "north"),
                        ("half", "lower"),
                        ("hinge", "left"),
                        ("open", "false"),
                        ("powered", "false"),
                    ],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_weathered_copper_door"),
                then: Then::Copy {
                    to: "minecraft:weathered_copper_door",
                    defaults: &[
                        ("facing", "north"),
                        ("half", "lower"),
                        ("hinge", "left"),
                        ("open", "false"),
                        ("powered", "false"),
                    ],
                },
            },
            Rule {
                when: Condition::Block("minecraft:waxed_oxidized_copper_door"),
                then: Then::Copy {
                    to: "minecraft:oxidized_copper_door",
                    defaults: &[
                        ("facing", "north"),
                        ("half", "lower"),
                        ("hinge", "left"),
                        ("open", "false"),
                        ("powered", "false"),
                    ],
                },
            },
        ],
    },
];

pub(crate) static HOE: [Group; 2] = [
    Group {
        sound: "minecraft:item.hoe.till",
        particle: None,
        transform_type: None,
        update_from_neighbors: true,
        damage: 1,
        disallowed_faces: &["down"],
        drop_strategy: None,
        loot: None,
        rules: &[
            Rule {
                when: Condition::AllOf(&[
                    Condition::Tag { tag: "minecraft:turns_into_farmland", offset: (0, 0, 0) },
                    Condition::Tag { tag: "minecraft:air", offset: (0, 1, 0) },
                ]),
                then: Then::Set { to: "minecraft:farmland", properties: &[("moisture", "0")] },
            },
            Rule {
                when: Condition::AllOf(&[
                    Condition::Block("minecraft:coarse_dirt"),
                    Condition::Tag { tag: "minecraft:air", offset: (0, 1, 0) },
                ]),
                then: Then::Set { to: "minecraft:dirt", properties: &[] },
            },
        ],
    },
    Group {
        sound: "minecraft:item.hoe.till",
        particle: None,
        transform_type: None,
        update_from_neighbors: true,
        damage: 1,
        disallowed_faces: &[],
        drop_strategy: Some("clicked_face"),
        loot: Some("minecraft:till/rooted_dirt"),
        rules: &[Rule {
            when: Condition::Block("minecraft:rooted_dirt"),
            then: Then::Set { to: "minecraft:dirt", properties: &[] },
        }],
    },
];

pub(crate) static SHOVEL: [Group; 1] = [
    Group {
        sound: "minecraft:item.shovel.flatten",
        particle: None,
        transform_type: None,
        update_from_neighbors: true,
        damage: 1,
        disallowed_faces: &["down"],
        drop_strategy: None,
        loot: None,
        rules: &[Rule {
            when: Condition::AllOf(&[
                Condition::Tag { tag: "minecraft:turns_into_dirt_path", offset: (0, 0, 0) },
                Condition::Tag { tag: "minecraft:air", offset: (0, 1, 0) },
            ]),
            then: Then::Set { to: "minecraft:dirt_path", properties: &[] },
        }],
    },
];
