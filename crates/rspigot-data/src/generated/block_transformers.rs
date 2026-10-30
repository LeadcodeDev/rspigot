use crate::transformers::{Group, Rule};

pub(crate) static AXE: [Group; 7] = [
    Group {
        sound: "minecraft:item.axe.strip",
        particle: None,
        transform_type: None,
        update_from_neighbors: true,
        damage: 1,
        rules: &[
            Rule {
                from: "minecraft:oak_wood",
                to: "minecraft:stripped_oak_wood",
                defaults: &[("axis", "y")],
            },
            Rule {
                from: "minecraft:oak_log",
                to: "minecraft:stripped_oak_log",
                defaults: &[("axis", "y")],
            },
            Rule {
                from: "minecraft:dark_oak_wood",
                to: "minecraft:stripped_dark_oak_wood",
                defaults: &[("axis", "y")],
            },
            Rule {
                from: "minecraft:dark_oak_log",
                to: "minecraft:stripped_dark_oak_log",
                defaults: &[("axis", "y")],
            },
            Rule {
                from: "minecraft:pale_oak_wood",
                to: "minecraft:stripped_pale_oak_wood",
                defaults: &[("axis", "y")],
            },
            Rule {
                from: "minecraft:pale_oak_log",
                to: "minecraft:stripped_pale_oak_log",
                defaults: &[("axis", "y")],
            },
            Rule {
                from: "minecraft:poplar_wood",
                to: "minecraft:stripped_poplar_wood",
                defaults: &[("axis", "y")],
            },
            Rule {
                from: "minecraft:poplar_log",
                to: "minecraft:stripped_poplar_log",
                defaults: &[("axis", "y")],
            },
            Rule {
                from: "minecraft:acacia_wood",
                to: "minecraft:stripped_acacia_wood",
                defaults: &[("axis", "y")],
            },
            Rule {
                from: "minecraft:acacia_log",
                to: "minecraft:stripped_acacia_log",
                defaults: &[("axis", "y")],
            },
            Rule {
                from: "minecraft:cherry_wood",
                to: "minecraft:stripped_cherry_wood",
                defaults: &[("axis", "y")],
            },
            Rule {
                from: "minecraft:cherry_log",
                to: "minecraft:stripped_cherry_log",
                defaults: &[("axis", "y")],
            },
            Rule {
                from: "minecraft:birch_wood",
                to: "minecraft:stripped_birch_wood",
                defaults: &[("axis", "y")],
            },
            Rule {
                from: "minecraft:birch_log",
                to: "minecraft:stripped_birch_log",
                defaults: &[("axis", "y")],
            },
            Rule {
                from: "minecraft:jungle_wood",
                to: "minecraft:stripped_jungle_wood",
                defaults: &[("axis", "y")],
            },
            Rule {
                from: "minecraft:jungle_log",
                to: "minecraft:stripped_jungle_log",
                defaults: &[("axis", "y")],
            },
            Rule {
                from: "minecraft:spruce_wood",
                to: "minecraft:stripped_spruce_wood",
                defaults: &[("axis", "y")],
            },
            Rule {
                from: "minecraft:spruce_log",
                to: "minecraft:stripped_spruce_log",
                defaults: &[("axis", "y")],
            },
            Rule {
                from: "minecraft:warped_stem",
                to: "minecraft:stripped_warped_stem",
                defaults: &[("axis", "y")],
            },
            Rule {
                from: "minecraft:warped_hyphae",
                to: "minecraft:stripped_warped_hyphae",
                defaults: &[("axis", "y")],
            },
            Rule {
                from: "minecraft:crimson_stem",
                to: "minecraft:stripped_crimson_stem",
                defaults: &[("axis", "y")],
            },
            Rule {
                from: "minecraft:crimson_hyphae",
                to: "minecraft:stripped_crimson_hyphae",
                defaults: &[("axis", "y")],
            },
            Rule {
                from: "minecraft:mangrove_wood",
                to: "minecraft:stripped_mangrove_wood",
                defaults: &[("axis", "y")],
            },
            Rule {
                from: "minecraft:mangrove_log",
                to: "minecraft:stripped_mangrove_log",
                defaults: &[("axis", "y")],
            },
            Rule {
                from: "minecraft:bamboo_block",
                to: "minecraft:stripped_bamboo_block",
                defaults: &[("axis", "y")],
            },
        ],
    },
    Group {
        sound: "minecraft:item.axe.scrape",
        particle: Some("scrape"),
        transform_type: None,
        update_from_neighbors: true,
        damage: 1,
        rules: &[
            Rule { from: "minecraft:exposed_copper", to: "minecraft:copper_block", defaults: &[] },
            Rule {
                from: "minecraft:weathered_copper",
                to: "minecraft:exposed_copper",
                defaults: &[],
            },
            Rule {
                from: "minecraft:oxidized_copper",
                to: "minecraft:weathered_copper",
                defaults: &[],
            },
            Rule {
                from: "minecraft:exposed_cut_copper",
                to: "minecraft:cut_copper",
                defaults: &[],
            },
            Rule {
                from: "minecraft:weathered_cut_copper",
                to: "minecraft:exposed_cut_copper",
                defaults: &[],
            },
            Rule {
                from: "minecraft:oxidized_cut_copper",
                to: "minecraft:weathered_cut_copper",
                defaults: &[],
            },
            Rule {
                from: "minecraft:exposed_chiseled_copper",
                to: "minecraft:chiseled_copper",
                defaults: &[],
            },
            Rule {
                from: "minecraft:weathered_chiseled_copper",
                to: "minecraft:exposed_chiseled_copper",
                defaults: &[],
            },
            Rule {
                from: "minecraft:oxidized_chiseled_copper",
                to: "minecraft:weathered_chiseled_copper",
                defaults: &[],
            },
            Rule {
                from: "minecraft:exposed_cut_copper_slab",
                to: "minecraft:cut_copper_slab",
                defaults: &[("type", "bottom"), ("waterlogged", "false")],
            },
            Rule {
                from: "minecraft:weathered_cut_copper_slab",
                to: "minecraft:exposed_cut_copper_slab",
                defaults: &[("type", "bottom"), ("waterlogged", "false")],
            },
            Rule {
                from: "minecraft:oxidized_cut_copper_slab",
                to: "minecraft:weathered_cut_copper_slab",
                defaults: &[("type", "bottom"), ("waterlogged", "false")],
            },
            Rule {
                from: "minecraft:exposed_cut_copper_stairs",
                to: "minecraft:cut_copper_stairs",
                defaults: &[
                    ("facing", "north"),
                    ("half", "bottom"),
                    ("shape", "straight"),
                    ("waterlogged", "false"),
                ],
            },
            Rule {
                from: "minecraft:weathered_cut_copper_stairs",
                to: "minecraft:exposed_cut_copper_stairs",
                defaults: &[
                    ("facing", "north"),
                    ("half", "bottom"),
                    ("shape", "straight"),
                    ("waterlogged", "false"),
                ],
            },
            Rule {
                from: "minecraft:oxidized_cut_copper_stairs",
                to: "minecraft:weathered_cut_copper_stairs",
                defaults: &[
                    ("facing", "north"),
                    ("half", "bottom"),
                    ("shape", "straight"),
                    ("waterlogged", "false"),
                ],
            },
            Rule {
                from: "minecraft:exposed_copper_trapdoor",
                to: "minecraft:copper_trapdoor",
                defaults: &[
                    ("facing", "north"),
                    ("half", "bottom"),
                    ("open", "false"),
                    ("powered", "false"),
                    ("waterlogged", "false"),
                ],
            },
            Rule {
                from: "minecraft:weathered_copper_trapdoor",
                to: "minecraft:exposed_copper_trapdoor",
                defaults: &[
                    ("facing", "north"),
                    ("half", "bottom"),
                    ("open", "false"),
                    ("powered", "false"),
                    ("waterlogged", "false"),
                ],
            },
            Rule {
                from: "minecraft:oxidized_copper_trapdoor",
                to: "minecraft:weathered_copper_trapdoor",
                defaults: &[
                    ("facing", "north"),
                    ("half", "bottom"),
                    ("open", "false"),
                    ("powered", "false"),
                    ("waterlogged", "false"),
                ],
            },
            Rule {
                from: "minecraft:exposed_copper_bars",
                to: "minecraft:copper_bars",
                defaults: &[
                    ("east", "false"),
                    ("north", "false"),
                    ("south", "false"),
                    ("waterlogged", "false"),
                    ("west", "false"),
                ],
            },
            Rule {
                from: "minecraft:weathered_copper_bars",
                to: "minecraft:exposed_copper_bars",
                defaults: &[
                    ("east", "false"),
                    ("north", "false"),
                    ("south", "false"),
                    ("waterlogged", "false"),
                    ("west", "false"),
                ],
            },
            Rule {
                from: "minecraft:oxidized_copper_bars",
                to: "minecraft:weathered_copper_bars",
                defaults: &[
                    ("east", "false"),
                    ("north", "false"),
                    ("south", "false"),
                    ("waterlogged", "false"),
                    ("west", "false"),
                ],
            },
            Rule {
                from: "minecraft:exposed_copper_grate",
                to: "minecraft:copper_grate",
                defaults: &[("waterlogged", "false")],
            },
            Rule {
                from: "minecraft:weathered_copper_grate",
                to: "minecraft:exposed_copper_grate",
                defaults: &[("waterlogged", "false")],
            },
            Rule {
                from: "minecraft:oxidized_copper_grate",
                to: "minecraft:weathered_copper_grate",
                defaults: &[("waterlogged", "false")],
            },
            Rule {
                from: "minecraft:exposed_copper_bulb",
                to: "minecraft:copper_bulb",
                defaults: &[("lit", "false"), ("powered", "false")],
            },
            Rule {
                from: "minecraft:weathered_copper_bulb",
                to: "minecraft:exposed_copper_bulb",
                defaults: &[("lit", "false"), ("powered", "false")],
            },
            Rule {
                from: "minecraft:oxidized_copper_bulb",
                to: "minecraft:weathered_copper_bulb",
                defaults: &[("lit", "false"), ("powered", "false")],
            },
            Rule {
                from: "minecraft:exposed_copper_lantern",
                to: "minecraft:copper_lantern",
                defaults: &[("hanging", "false"), ("waterlogged", "false")],
            },
            Rule {
                from: "minecraft:weathered_copper_lantern",
                to: "minecraft:exposed_copper_lantern",
                defaults: &[("hanging", "false"), ("waterlogged", "false")],
            },
            Rule {
                from: "minecraft:oxidized_copper_lantern",
                to: "minecraft:weathered_copper_lantern",
                defaults: &[("hanging", "false"), ("waterlogged", "false")],
            },
            Rule {
                from: "minecraft:exposed_copper_golem_statue",
                to: "minecraft:copper_golem_statue",
                defaults: &[
                    ("copper_golem_pose", "standing"),
                    ("facing", "north"),
                    ("waterlogged", "false"),
                ],
            },
            Rule {
                from: "minecraft:weathered_copper_golem_statue",
                to: "minecraft:exposed_copper_golem_statue",
                defaults: &[
                    ("copper_golem_pose", "standing"),
                    ("facing", "north"),
                    ("waterlogged", "false"),
                ],
            },
            Rule {
                from: "minecraft:oxidized_copper_golem_statue",
                to: "minecraft:weathered_copper_golem_statue",
                defaults: &[
                    ("copper_golem_pose", "standing"),
                    ("facing", "north"),
                    ("waterlogged", "false"),
                ],
            },
            Rule {
                from: "minecraft:exposed_lightning_rod",
                to: "minecraft:lightning_rod",
                defaults: &[("facing", "up"), ("powered", "false"), ("waterlogged", "false")],
            },
            Rule {
                from: "minecraft:weathered_lightning_rod",
                to: "minecraft:exposed_lightning_rod",
                defaults: &[("facing", "up"), ("powered", "false"), ("waterlogged", "false")],
            },
            Rule {
                from: "minecraft:oxidized_lightning_rod",
                to: "minecraft:weathered_lightning_rod",
                defaults: &[("facing", "up"), ("powered", "false"), ("waterlogged", "false")],
            },
            Rule {
                from: "minecraft:exposed_copper_chain",
                to: "minecraft:copper_chain",
                defaults: &[("axis", "y"), ("waterlogged", "false")],
            },
            Rule {
                from: "minecraft:weathered_copper_chain",
                to: "minecraft:exposed_copper_chain",
                defaults: &[("axis", "y"), ("waterlogged", "false")],
            },
            Rule {
                from: "minecraft:oxidized_copper_chain",
                to: "minecraft:weathered_copper_chain",
                defaults: &[("axis", "y"), ("waterlogged", "false")],
            },
        ],
    },
    Group {
        sound: "minecraft:item.axe.scrape",
        particle: Some("scrape"),
        transform_type: Some("copper_chest"),
        update_from_neighbors: false,
        damage: 1,
        rules: &[
            Rule {
                from: "minecraft:exposed_copper_chest",
                to: "minecraft:copper_chest",
                defaults: &[("facing", "north"), ("type", "single"), ("waterlogged", "false")],
            },
            Rule {
                from: "minecraft:weathered_copper_chest",
                to: "minecraft:exposed_copper_chest",
                defaults: &[("facing", "north"), ("type", "single"), ("waterlogged", "false")],
            },
            Rule {
                from: "minecraft:oxidized_copper_chest",
                to: "minecraft:weathered_copper_chest",
                defaults: &[("facing", "north"), ("type", "single"), ("waterlogged", "false")],
            },
        ],
    },
    Group {
        sound: "minecraft:item.axe.scrape",
        particle: Some("scrape"),
        transform_type: None,
        update_from_neighbors: false,
        damage: 1,
        rules: &[
            Rule {
                from: "minecraft:exposed_copper_door",
                to: "minecraft:copper_door",
                defaults: &[
                    ("facing", "north"),
                    ("half", "lower"),
                    ("hinge", "left"),
                    ("open", "false"),
                    ("powered", "false"),
                ],
            },
            Rule {
                from: "minecraft:weathered_copper_door",
                to: "minecraft:exposed_copper_door",
                defaults: &[
                    ("facing", "north"),
                    ("half", "lower"),
                    ("hinge", "left"),
                    ("open", "false"),
                    ("powered", "false"),
                ],
            },
            Rule {
                from: "minecraft:oxidized_copper_door",
                to: "minecraft:weathered_copper_door",
                defaults: &[
                    ("facing", "north"),
                    ("half", "lower"),
                    ("hinge", "left"),
                    ("open", "false"),
                    ("powered", "false"),
                ],
            },
        ],
    },
    Group {
        sound: "minecraft:item.axe.wax_off",
        particle: Some("wax_off"),
        transform_type: None,
        update_from_neighbors: true,
        damage: 1,
        rules: &[
            Rule {
                from: "minecraft:waxed_copper_block",
                to: "minecraft:copper_block",
                defaults: &[],
            },
            Rule {
                from: "minecraft:waxed_exposed_copper",
                to: "minecraft:exposed_copper",
                defaults: &[],
            },
            Rule {
                from: "minecraft:waxed_weathered_copper",
                to: "minecraft:weathered_copper",
                defaults: &[],
            },
            Rule {
                from: "minecraft:waxed_oxidized_copper",
                to: "minecraft:oxidized_copper",
                defaults: &[],
            },
            Rule { from: "minecraft:waxed_cut_copper", to: "minecraft:cut_copper", defaults: &[] },
            Rule {
                from: "minecraft:waxed_exposed_cut_copper",
                to: "minecraft:exposed_cut_copper",
                defaults: &[],
            },
            Rule {
                from: "minecraft:waxed_weathered_cut_copper",
                to: "minecraft:weathered_cut_copper",
                defaults: &[],
            },
            Rule {
                from: "minecraft:waxed_oxidized_cut_copper",
                to: "minecraft:oxidized_cut_copper",
                defaults: &[],
            },
            Rule {
                from: "minecraft:waxed_cut_copper_slab",
                to: "minecraft:cut_copper_slab",
                defaults: &[("type", "bottom"), ("waterlogged", "false")],
            },
            Rule {
                from: "minecraft:waxed_exposed_cut_copper_slab",
                to: "minecraft:exposed_cut_copper_slab",
                defaults: &[("type", "bottom"), ("waterlogged", "false")],
            },
            Rule {
                from: "minecraft:waxed_weathered_cut_copper_slab",
                to: "minecraft:weathered_cut_copper_slab",
                defaults: &[("type", "bottom"), ("waterlogged", "false")],
            },
            Rule {
                from: "minecraft:waxed_oxidized_cut_copper_slab",
                to: "minecraft:oxidized_cut_copper_slab",
                defaults: &[("type", "bottom"), ("waterlogged", "false")],
            },
            Rule {
                from: "minecraft:waxed_cut_copper_stairs",
                to: "minecraft:cut_copper_stairs",
                defaults: &[
                    ("facing", "north"),
                    ("half", "bottom"),
                    ("shape", "straight"),
                    ("waterlogged", "false"),
                ],
            },
            Rule {
                from: "minecraft:waxed_exposed_cut_copper_stairs",
                to: "minecraft:exposed_cut_copper_stairs",
                defaults: &[
                    ("facing", "north"),
                    ("half", "bottom"),
                    ("shape", "straight"),
                    ("waterlogged", "false"),
                ],
            },
            Rule {
                from: "minecraft:waxed_weathered_cut_copper_stairs",
                to: "minecraft:weathered_cut_copper_stairs",
                defaults: &[
                    ("facing", "north"),
                    ("half", "bottom"),
                    ("shape", "straight"),
                    ("waterlogged", "false"),
                ],
            },
            Rule {
                from: "minecraft:waxed_oxidized_cut_copper_stairs",
                to: "minecraft:oxidized_cut_copper_stairs",
                defaults: &[
                    ("facing", "north"),
                    ("half", "bottom"),
                    ("shape", "straight"),
                    ("waterlogged", "false"),
                ],
            },
            Rule {
                from: "minecraft:waxed_chiseled_copper",
                to: "minecraft:chiseled_copper",
                defaults: &[],
            },
            Rule {
                from: "minecraft:waxed_exposed_chiseled_copper",
                to: "minecraft:exposed_chiseled_copper",
                defaults: &[],
            },
            Rule {
                from: "minecraft:waxed_weathered_chiseled_copper",
                to: "minecraft:weathered_chiseled_copper",
                defaults: &[],
            },
            Rule {
                from: "minecraft:waxed_oxidized_chiseled_copper",
                to: "minecraft:oxidized_chiseled_copper",
                defaults: &[],
            },
            Rule {
                from: "minecraft:waxed_copper_trapdoor",
                to: "minecraft:copper_trapdoor",
                defaults: &[
                    ("facing", "north"),
                    ("half", "bottom"),
                    ("open", "false"),
                    ("powered", "false"),
                    ("waterlogged", "false"),
                ],
            },
            Rule {
                from: "minecraft:waxed_exposed_copper_trapdoor",
                to: "minecraft:exposed_copper_trapdoor",
                defaults: &[
                    ("facing", "north"),
                    ("half", "bottom"),
                    ("open", "false"),
                    ("powered", "false"),
                    ("waterlogged", "false"),
                ],
            },
            Rule {
                from: "minecraft:waxed_weathered_copper_trapdoor",
                to: "minecraft:weathered_copper_trapdoor",
                defaults: &[
                    ("facing", "north"),
                    ("half", "bottom"),
                    ("open", "false"),
                    ("powered", "false"),
                    ("waterlogged", "false"),
                ],
            },
            Rule {
                from: "minecraft:waxed_oxidized_copper_trapdoor",
                to: "minecraft:oxidized_copper_trapdoor",
                defaults: &[
                    ("facing", "north"),
                    ("half", "bottom"),
                    ("open", "false"),
                    ("powered", "false"),
                    ("waterlogged", "false"),
                ],
            },
            Rule {
                from: "minecraft:waxed_copper_bars",
                to: "minecraft:copper_bars",
                defaults: &[
                    ("east", "false"),
                    ("north", "false"),
                    ("south", "false"),
                    ("waterlogged", "false"),
                    ("west", "false"),
                ],
            },
            Rule {
                from: "minecraft:waxed_exposed_copper_bars",
                to: "minecraft:exposed_copper_bars",
                defaults: &[
                    ("east", "false"),
                    ("north", "false"),
                    ("south", "false"),
                    ("waterlogged", "false"),
                    ("west", "false"),
                ],
            },
            Rule {
                from: "minecraft:waxed_weathered_copper_bars",
                to: "minecraft:weathered_copper_bars",
                defaults: &[
                    ("east", "false"),
                    ("north", "false"),
                    ("south", "false"),
                    ("waterlogged", "false"),
                    ("west", "false"),
                ],
            },
            Rule {
                from: "minecraft:waxed_oxidized_copper_bars",
                to: "minecraft:oxidized_copper_bars",
                defaults: &[
                    ("east", "false"),
                    ("north", "false"),
                    ("south", "false"),
                    ("waterlogged", "false"),
                    ("west", "false"),
                ],
            },
            Rule {
                from: "minecraft:waxed_copper_grate",
                to: "minecraft:copper_grate",
                defaults: &[("waterlogged", "false")],
            },
            Rule {
                from: "minecraft:waxed_exposed_copper_grate",
                to: "minecraft:exposed_copper_grate",
                defaults: &[("waterlogged", "false")],
            },
            Rule {
                from: "minecraft:waxed_weathered_copper_grate",
                to: "minecraft:weathered_copper_grate",
                defaults: &[("waterlogged", "false")],
            },
            Rule {
                from: "minecraft:waxed_oxidized_copper_grate",
                to: "minecraft:oxidized_copper_grate",
                defaults: &[("waterlogged", "false")],
            },
            Rule {
                from: "minecraft:waxed_copper_bulb",
                to: "minecraft:copper_bulb",
                defaults: &[("lit", "false"), ("powered", "false")],
            },
            Rule {
                from: "minecraft:waxed_exposed_copper_bulb",
                to: "minecraft:exposed_copper_bulb",
                defaults: &[("lit", "false"), ("powered", "false")],
            },
            Rule {
                from: "minecraft:waxed_weathered_copper_bulb",
                to: "minecraft:weathered_copper_bulb",
                defaults: &[("lit", "false"), ("powered", "false")],
            },
            Rule {
                from: "minecraft:waxed_oxidized_copper_bulb",
                to: "minecraft:oxidized_copper_bulb",
                defaults: &[("lit", "false"), ("powered", "false")],
            },
            Rule {
                from: "minecraft:waxed_copper_golem_statue",
                to: "minecraft:copper_golem_statue",
                defaults: &[
                    ("copper_golem_pose", "standing"),
                    ("facing", "north"),
                    ("waterlogged", "false"),
                ],
            },
            Rule {
                from: "minecraft:waxed_exposed_copper_golem_statue",
                to: "minecraft:exposed_copper_golem_statue",
                defaults: &[
                    ("copper_golem_pose", "standing"),
                    ("facing", "north"),
                    ("waterlogged", "false"),
                ],
            },
            Rule {
                from: "minecraft:waxed_weathered_copper_golem_statue",
                to: "minecraft:weathered_copper_golem_statue",
                defaults: &[
                    ("copper_golem_pose", "standing"),
                    ("facing", "north"),
                    ("waterlogged", "false"),
                ],
            },
            Rule {
                from: "minecraft:waxed_oxidized_copper_golem_statue",
                to: "minecraft:oxidized_copper_golem_statue",
                defaults: &[
                    ("copper_golem_pose", "standing"),
                    ("facing", "north"),
                    ("waterlogged", "false"),
                ],
            },
            Rule {
                from: "minecraft:waxed_lightning_rod",
                to: "minecraft:lightning_rod",
                defaults: &[("facing", "up"), ("powered", "false"), ("waterlogged", "false")],
            },
            Rule {
                from: "minecraft:waxed_exposed_lightning_rod",
                to: "minecraft:exposed_lightning_rod",
                defaults: &[("facing", "up"), ("powered", "false"), ("waterlogged", "false")],
            },
            Rule {
                from: "minecraft:waxed_weathered_lightning_rod",
                to: "minecraft:weathered_lightning_rod",
                defaults: &[("facing", "up"), ("powered", "false"), ("waterlogged", "false")],
            },
            Rule {
                from: "minecraft:waxed_oxidized_lightning_rod",
                to: "minecraft:oxidized_lightning_rod",
                defaults: &[("facing", "up"), ("powered", "false"), ("waterlogged", "false")],
            },
            Rule {
                from: "minecraft:waxed_copper_lantern",
                to: "minecraft:copper_lantern",
                defaults: &[("hanging", "false"), ("waterlogged", "false")],
            },
            Rule {
                from: "minecraft:waxed_exposed_copper_lantern",
                to: "minecraft:exposed_copper_lantern",
                defaults: &[("hanging", "false"), ("waterlogged", "false")],
            },
            Rule {
                from: "minecraft:waxed_weathered_copper_lantern",
                to: "minecraft:weathered_copper_lantern",
                defaults: &[("hanging", "false"), ("waterlogged", "false")],
            },
            Rule {
                from: "minecraft:waxed_oxidized_copper_lantern",
                to: "minecraft:oxidized_copper_lantern",
                defaults: &[("hanging", "false"), ("waterlogged", "false")],
            },
            Rule {
                from: "minecraft:waxed_copper_chain",
                to: "minecraft:copper_chain",
                defaults: &[("axis", "y"), ("waterlogged", "false")],
            },
            Rule {
                from: "minecraft:waxed_exposed_copper_chain",
                to: "minecraft:exposed_copper_chain",
                defaults: &[("axis", "y"), ("waterlogged", "false")],
            },
            Rule {
                from: "minecraft:waxed_weathered_copper_chain",
                to: "minecraft:weathered_copper_chain",
                defaults: &[("axis", "y"), ("waterlogged", "false")],
            },
            Rule {
                from: "minecraft:waxed_oxidized_copper_chain",
                to: "minecraft:oxidized_copper_chain",
                defaults: &[("axis", "y"), ("waterlogged", "false")],
            },
        ],
    },
    Group {
        sound: "minecraft:item.axe.wax_off",
        particle: Some("wax_off"),
        transform_type: Some("copper_chest"),
        update_from_neighbors: false,
        damage: 1,
        rules: &[
            Rule {
                from: "minecraft:waxed_copper_chest",
                to: "minecraft:copper_chest",
                defaults: &[("facing", "north"), ("type", "single"), ("waterlogged", "false")],
            },
            Rule {
                from: "minecraft:waxed_exposed_copper_chest",
                to: "minecraft:exposed_copper_chest",
                defaults: &[("facing", "north"), ("type", "single"), ("waterlogged", "false")],
            },
            Rule {
                from: "minecraft:waxed_weathered_copper_chest",
                to: "minecraft:weathered_copper_chest",
                defaults: &[("facing", "north"), ("type", "single"), ("waterlogged", "false")],
            },
            Rule {
                from: "minecraft:waxed_oxidized_copper_chest",
                to: "minecraft:oxidized_copper_chest",
                defaults: &[("facing", "north"), ("type", "single"), ("waterlogged", "false")],
            },
        ],
    },
    Group {
        sound: "minecraft:item.axe.wax_off",
        particle: Some("wax_off"),
        transform_type: None,
        update_from_neighbors: false,
        damage: 1,
        rules: &[
            Rule {
                from: "minecraft:waxed_copper_door",
                to: "minecraft:copper_door",
                defaults: &[
                    ("facing", "north"),
                    ("half", "lower"),
                    ("hinge", "left"),
                    ("open", "false"),
                    ("powered", "false"),
                ],
            },
            Rule {
                from: "minecraft:waxed_exposed_copper_door",
                to: "minecraft:exposed_copper_door",
                defaults: &[
                    ("facing", "north"),
                    ("half", "lower"),
                    ("hinge", "left"),
                    ("open", "false"),
                    ("powered", "false"),
                ],
            },
            Rule {
                from: "minecraft:waxed_weathered_copper_door",
                to: "minecraft:weathered_copper_door",
                defaults: &[
                    ("facing", "north"),
                    ("half", "lower"),
                    ("hinge", "left"),
                    ("open", "false"),
                    ("powered", "false"),
                ],
            },
            Rule {
                from: "minecraft:waxed_oxidized_copper_door",
                to: "minecraft:oxidized_copper_door",
                defaults: &[
                    ("facing", "north"),
                    ("half", "lower"),
                    ("hinge", "left"),
                    ("open", "false"),
                    ("powered", "false"),
                ],
            },
        ],
    },
];
