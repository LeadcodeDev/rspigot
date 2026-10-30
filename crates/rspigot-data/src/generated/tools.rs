use crate::items::{Tool, ToolRule};
use crate::tags::HolderSet;

pub(crate) static TOOLS: [(u16, Tool); 38] = [
    (
        1025,
        Tool {
            rules: &[
                ToolRule {
                    blocks: HolderSet::Direct(&[139]),
                    speed: Some(15.0),
                    correct_for_drops: Some(true),
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:sword_instantly_mines"),
                    speed: Some(3.4028235e38),
                    correct_for_drops: None,
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:sword_efficient"),
                    speed: Some(1.5),
                    correct_for_drops: None,
                },
            ],
            default_mining_speed: 1.0,
            damage_per_block: 2,
            can_destroy_blocks_in_creative: false,
        },
    ),
    (
        1026,
        Tool {
            rules: &[
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:incorrect_for_wooden_tool"),
                    speed: None,
                    correct_for_drops: Some(false),
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:mineable/shovel"),
                    speed: Some(2.0),
                    correct_for_drops: Some(true),
                },
            ],
            default_mining_speed: 1.0,
            damage_per_block: 1,
            can_destroy_blocks_in_creative: true,
        },
    ),
    (
        1027,
        Tool {
            rules: &[
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:incorrect_for_wooden_tool"),
                    speed: None,
                    correct_for_drops: Some(false),
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:mineable/pickaxe"),
                    speed: Some(2.0),
                    correct_for_drops: Some(true),
                },
            ],
            default_mining_speed: 1.0,
            damage_per_block: 1,
            can_destroy_blocks_in_creative: true,
        },
    ),
    (
        1028,
        Tool {
            rules: &[
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:incorrect_for_wooden_tool"),
                    speed: None,
                    correct_for_drops: Some(false),
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:mineable/axe"),
                    speed: Some(2.0),
                    correct_for_drops: Some(true),
                },
            ],
            default_mining_speed: 1.0,
            damage_per_block: 1,
            can_destroy_blocks_in_creative: true,
        },
    ),
    (
        1029,
        Tool {
            rules: &[
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:incorrect_for_wooden_tool"),
                    speed: None,
                    correct_for_drops: Some(false),
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:mineable/hoe"),
                    speed: Some(2.0),
                    correct_for_drops: Some(true),
                },
            ],
            default_mining_speed: 1.0,
            damage_per_block: 1,
            can_destroy_blocks_in_creative: true,
        },
    ),
    (
        1030,
        Tool {
            rules: &[
                ToolRule {
                    blocks: HolderSet::Direct(&[139]),
                    speed: Some(15.0),
                    correct_for_drops: Some(true),
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:sword_instantly_mines"),
                    speed: Some(3.4028235e38),
                    correct_for_drops: None,
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:sword_efficient"),
                    speed: Some(1.5),
                    correct_for_drops: None,
                },
            ],
            default_mining_speed: 1.0,
            damage_per_block: 2,
            can_destroy_blocks_in_creative: false,
        },
    ),
    (
        1031,
        Tool {
            rules: &[
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:incorrect_for_copper_tool"),
                    speed: None,
                    correct_for_drops: Some(false),
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:mineable/shovel"),
                    speed: Some(5.0),
                    correct_for_drops: Some(true),
                },
            ],
            default_mining_speed: 1.0,
            damage_per_block: 1,
            can_destroy_blocks_in_creative: true,
        },
    ),
    (
        1032,
        Tool {
            rules: &[
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:incorrect_for_copper_tool"),
                    speed: None,
                    correct_for_drops: Some(false),
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:mineable/pickaxe"),
                    speed: Some(5.0),
                    correct_for_drops: Some(true),
                },
            ],
            default_mining_speed: 1.0,
            damage_per_block: 1,
            can_destroy_blocks_in_creative: true,
        },
    ),
    (
        1033,
        Tool {
            rules: &[
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:incorrect_for_copper_tool"),
                    speed: None,
                    correct_for_drops: Some(false),
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:mineable/axe"),
                    speed: Some(5.0),
                    correct_for_drops: Some(true),
                },
            ],
            default_mining_speed: 1.0,
            damage_per_block: 1,
            can_destroy_blocks_in_creative: true,
        },
    ),
    (
        1034,
        Tool {
            rules: &[
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:incorrect_for_copper_tool"),
                    speed: None,
                    correct_for_drops: Some(false),
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:mineable/hoe"),
                    speed: Some(5.0),
                    correct_for_drops: Some(true),
                },
            ],
            default_mining_speed: 1.0,
            damage_per_block: 1,
            can_destroy_blocks_in_creative: true,
        },
    ),
    (
        1035,
        Tool {
            rules: &[
                ToolRule {
                    blocks: HolderSet::Direct(&[139]),
                    speed: Some(15.0),
                    correct_for_drops: Some(true),
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:sword_instantly_mines"),
                    speed: Some(3.4028235e38),
                    correct_for_drops: None,
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:sword_efficient"),
                    speed: Some(1.5),
                    correct_for_drops: None,
                },
            ],
            default_mining_speed: 1.0,
            damage_per_block: 2,
            can_destroy_blocks_in_creative: false,
        },
    ),
    (
        1036,
        Tool {
            rules: &[
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:incorrect_for_stone_tool"),
                    speed: None,
                    correct_for_drops: Some(false),
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:mineable/shovel"),
                    speed: Some(4.0),
                    correct_for_drops: Some(true),
                },
            ],
            default_mining_speed: 1.0,
            damage_per_block: 1,
            can_destroy_blocks_in_creative: true,
        },
    ),
    (
        1037,
        Tool {
            rules: &[
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:incorrect_for_stone_tool"),
                    speed: None,
                    correct_for_drops: Some(false),
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:mineable/pickaxe"),
                    speed: Some(4.0),
                    correct_for_drops: Some(true),
                },
            ],
            default_mining_speed: 1.0,
            damage_per_block: 1,
            can_destroy_blocks_in_creative: true,
        },
    ),
    (
        1038,
        Tool {
            rules: &[
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:incorrect_for_stone_tool"),
                    speed: None,
                    correct_for_drops: Some(false),
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:mineable/axe"),
                    speed: Some(4.0),
                    correct_for_drops: Some(true),
                },
            ],
            default_mining_speed: 1.0,
            damage_per_block: 1,
            can_destroy_blocks_in_creative: true,
        },
    ),
    (
        1039,
        Tool {
            rules: &[
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:incorrect_for_stone_tool"),
                    speed: None,
                    correct_for_drops: Some(false),
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:mineable/hoe"),
                    speed: Some(4.0),
                    correct_for_drops: Some(true),
                },
            ],
            default_mining_speed: 1.0,
            damage_per_block: 1,
            can_destroy_blocks_in_creative: true,
        },
    ),
    (
        1040,
        Tool {
            rules: &[
                ToolRule {
                    blocks: HolderSet::Direct(&[139]),
                    speed: Some(15.0),
                    correct_for_drops: Some(true),
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:sword_instantly_mines"),
                    speed: Some(3.4028235e38),
                    correct_for_drops: None,
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:sword_efficient"),
                    speed: Some(1.5),
                    correct_for_drops: None,
                },
            ],
            default_mining_speed: 1.0,
            damage_per_block: 2,
            can_destroy_blocks_in_creative: false,
        },
    ),
    (
        1041,
        Tool {
            rules: &[
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:incorrect_for_gold_tool"),
                    speed: None,
                    correct_for_drops: Some(false),
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:mineable/shovel"),
                    speed: Some(12.0),
                    correct_for_drops: Some(true),
                },
            ],
            default_mining_speed: 1.0,
            damage_per_block: 1,
            can_destroy_blocks_in_creative: true,
        },
    ),
    (
        1042,
        Tool {
            rules: &[
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:incorrect_for_gold_tool"),
                    speed: None,
                    correct_for_drops: Some(false),
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:mineable/pickaxe"),
                    speed: Some(12.0),
                    correct_for_drops: Some(true),
                },
            ],
            default_mining_speed: 1.0,
            damage_per_block: 1,
            can_destroy_blocks_in_creative: true,
        },
    ),
    (
        1043,
        Tool {
            rules: &[
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:incorrect_for_gold_tool"),
                    speed: None,
                    correct_for_drops: Some(false),
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:mineable/axe"),
                    speed: Some(12.0),
                    correct_for_drops: Some(true),
                },
            ],
            default_mining_speed: 1.0,
            damage_per_block: 1,
            can_destroy_blocks_in_creative: true,
        },
    ),
    (
        1044,
        Tool {
            rules: &[
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:incorrect_for_gold_tool"),
                    speed: None,
                    correct_for_drops: Some(false),
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:mineable/hoe"),
                    speed: Some(12.0),
                    correct_for_drops: Some(true),
                },
            ],
            default_mining_speed: 1.0,
            damage_per_block: 1,
            can_destroy_blocks_in_creative: true,
        },
    ),
    (
        1045,
        Tool {
            rules: &[
                ToolRule {
                    blocks: HolderSet::Direct(&[139]),
                    speed: Some(15.0),
                    correct_for_drops: Some(true),
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:sword_instantly_mines"),
                    speed: Some(3.4028235e38),
                    correct_for_drops: None,
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:sword_efficient"),
                    speed: Some(1.5),
                    correct_for_drops: None,
                },
            ],
            default_mining_speed: 1.0,
            damage_per_block: 2,
            can_destroy_blocks_in_creative: false,
        },
    ),
    (
        1046,
        Tool {
            rules: &[
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:incorrect_for_iron_tool"),
                    speed: None,
                    correct_for_drops: Some(false),
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:mineable/shovel"),
                    speed: Some(6.0),
                    correct_for_drops: Some(true),
                },
            ],
            default_mining_speed: 1.0,
            damage_per_block: 1,
            can_destroy_blocks_in_creative: true,
        },
    ),
    (
        1047,
        Tool {
            rules: &[
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:incorrect_for_iron_tool"),
                    speed: None,
                    correct_for_drops: Some(false),
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:mineable/pickaxe"),
                    speed: Some(6.0),
                    correct_for_drops: Some(true),
                },
            ],
            default_mining_speed: 1.0,
            damage_per_block: 1,
            can_destroy_blocks_in_creative: true,
        },
    ),
    (
        1048,
        Tool {
            rules: &[
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:incorrect_for_iron_tool"),
                    speed: None,
                    correct_for_drops: Some(false),
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:mineable/axe"),
                    speed: Some(6.0),
                    correct_for_drops: Some(true),
                },
            ],
            default_mining_speed: 1.0,
            damage_per_block: 1,
            can_destroy_blocks_in_creative: true,
        },
    ),
    (
        1049,
        Tool {
            rules: &[
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:incorrect_for_iron_tool"),
                    speed: None,
                    correct_for_drops: Some(false),
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:mineable/hoe"),
                    speed: Some(6.0),
                    correct_for_drops: Some(true),
                },
            ],
            default_mining_speed: 1.0,
            damage_per_block: 1,
            can_destroy_blocks_in_creative: true,
        },
    ),
    (
        1050,
        Tool {
            rules: &[
                ToolRule {
                    blocks: HolderSet::Direct(&[139]),
                    speed: Some(15.0),
                    correct_for_drops: Some(true),
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:sword_instantly_mines"),
                    speed: Some(3.4028235e38),
                    correct_for_drops: None,
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:sword_efficient"),
                    speed: Some(1.5),
                    correct_for_drops: None,
                },
            ],
            default_mining_speed: 1.0,
            damage_per_block: 2,
            can_destroy_blocks_in_creative: false,
        },
    ),
    (
        1051,
        Tool {
            rules: &[
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:incorrect_for_diamond_tool"),
                    speed: None,
                    correct_for_drops: Some(false),
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:mineable/shovel"),
                    speed: Some(8.0),
                    correct_for_drops: Some(true),
                },
            ],
            default_mining_speed: 1.0,
            damage_per_block: 1,
            can_destroy_blocks_in_creative: true,
        },
    ),
    (
        1052,
        Tool {
            rules: &[
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:incorrect_for_diamond_tool"),
                    speed: None,
                    correct_for_drops: Some(false),
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:mineable/pickaxe"),
                    speed: Some(8.0),
                    correct_for_drops: Some(true),
                },
            ],
            default_mining_speed: 1.0,
            damage_per_block: 1,
            can_destroy_blocks_in_creative: true,
        },
    ),
    (
        1053,
        Tool {
            rules: &[
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:incorrect_for_diamond_tool"),
                    speed: None,
                    correct_for_drops: Some(false),
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:mineable/axe"),
                    speed: Some(8.0),
                    correct_for_drops: Some(true),
                },
            ],
            default_mining_speed: 1.0,
            damage_per_block: 1,
            can_destroy_blocks_in_creative: true,
        },
    ),
    (
        1054,
        Tool {
            rules: &[
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:incorrect_for_diamond_tool"),
                    speed: None,
                    correct_for_drops: Some(false),
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:mineable/hoe"),
                    speed: Some(8.0),
                    correct_for_drops: Some(true),
                },
            ],
            default_mining_speed: 1.0,
            damage_per_block: 1,
            can_destroy_blocks_in_creative: true,
        },
    ),
    (
        1055,
        Tool {
            rules: &[
                ToolRule {
                    blocks: HolderSet::Direct(&[139]),
                    speed: Some(15.0),
                    correct_for_drops: Some(true),
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:sword_instantly_mines"),
                    speed: Some(3.4028235e38),
                    correct_for_drops: None,
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:sword_efficient"),
                    speed: Some(1.5),
                    correct_for_drops: None,
                },
            ],
            default_mining_speed: 1.0,
            damage_per_block: 2,
            can_destroy_blocks_in_creative: false,
        },
    ),
    (
        1056,
        Tool {
            rules: &[
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:incorrect_for_netherite_tool"),
                    speed: None,
                    correct_for_drops: Some(false),
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:mineable/shovel"),
                    speed: Some(9.0),
                    correct_for_drops: Some(true),
                },
            ],
            default_mining_speed: 1.0,
            damage_per_block: 1,
            can_destroy_blocks_in_creative: true,
        },
    ),
    (
        1057,
        Tool {
            rules: &[
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:incorrect_for_netherite_tool"),
                    speed: None,
                    correct_for_drops: Some(false),
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:mineable/pickaxe"),
                    speed: Some(9.0),
                    correct_for_drops: Some(true),
                },
            ],
            default_mining_speed: 1.0,
            damage_per_block: 1,
            can_destroy_blocks_in_creative: true,
        },
    ),
    (
        1058,
        Tool {
            rules: &[
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:incorrect_for_netherite_tool"),
                    speed: None,
                    correct_for_drops: Some(false),
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:mineable/axe"),
                    speed: Some(9.0),
                    correct_for_drops: Some(true),
                },
            ],
            default_mining_speed: 1.0,
            damage_per_block: 1,
            can_destroy_blocks_in_creative: true,
        },
    ),
    (
        1059,
        Tool {
            rules: &[
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:incorrect_for_netherite_tool"),
                    speed: None,
                    correct_for_drops: Some(false),
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:mineable/hoe"),
                    speed: Some(9.0),
                    correct_for_drops: Some(true),
                },
            ],
            default_mining_speed: 1.0,
            damage_per_block: 1,
            can_destroy_blocks_in_creative: true,
        },
    ),
    (
        1255,
        Tool {
            rules: &[
                ToolRule {
                    blocks: HolderSet::Direct(&[139]),
                    speed: Some(15.0),
                    correct_for_drops: Some(true),
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:shears_extreme_breaking_speed"),
                    speed: Some(15.0),
                    correct_for_drops: None,
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:shears_major_breaking_speed"),
                    speed: Some(5.0),
                    correct_for_drops: None,
                },
                ToolRule {
                    blocks: HolderSet::Tag("minecraft:shears_minor_breaking_speed"),
                    speed: Some(2.0),
                    correct_for_drops: None,
                },
            ],
            default_mining_speed: 1.0,
            damage_per_block: 1,
            can_destroy_blocks_in_creative: true,
        },
    ),
    (
        1374,
        Tool {
            rules: &[],
            default_mining_speed: 1.0,
            damage_per_block: 2,
            can_destroy_blocks_in_creative: false,
        },
    ),
    (
        1483,
        Tool {
            rules: &[],
            default_mining_speed: 1.0,
            damage_per_block: 2,
            can_destroy_blocks_in_creative: false,
        },
    ),
];
