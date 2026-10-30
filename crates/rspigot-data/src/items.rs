use crate::generated::items::MAX_STACK;
use crate::generated::static_registries::{ITEM, STATIC};
use crate::generated::tools::TOOLS;
use crate::tags::HolderSet;

const BLOCK_REGISTRY: &str = "minecraft:block";

const DEFAULT_STACK_SIZE: u8 = 64;

pub fn id(name: &str) -> Option<u16> {
    STATIC[ITEM].index_of(name)
}

pub fn name(id: u16) -> Option<&'static str> {
    STATIC[ITEM].name_of(id)
}

pub fn count() -> usize {
    STATIC[ITEM].entries.len()
}

pub fn max_stack_size(id: u16) -> u8 {
    MAX_STACK
        .get(usize::from(id))
        .copied()
        .unwrap_or(DEFAULT_STACK_SIZE)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tool {
    pub rules: &'static [ToolRule],
    pub default_mining_speed: f32,
    pub damage_per_block: i32,
    pub can_destroy_blocks_in_creative: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ToolRule {
    pub blocks: HolderSet,
    pub speed: Option<f32>,
    pub correct_for_drops: Option<bool>,
}

impl Tool {
    pub fn is_correct_for_drops(&self, block: u32) -> bool {
        self.rules
            .iter()
            .find_map(|rule| {
                rule.correct_for_drops
                    .filter(|_| rule.blocks.contains(BLOCK_REGISTRY, block))
            })
            .unwrap_or(false)
    }
}

pub fn tools() -> &'static [(u16, Tool)] {
    &TOOLS
}

pub fn tool(id: u16) -> Option<&'static Tool> {
    TOOLS
        .binary_search_by_key(&id, |(item, _)| *item)
        .ok()
        .map(|index| &TOOLS[index].1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registries;
    use crate::tags::HolderSet;

    fn block(name: &str) -> u32 {
        registries::static_index("minecraft:block", name).unwrap()
    }

    fn item(name: &str) -> u16 {
        id(name).unwrap()
    }

    #[test]
    fn tools_are_sorted_and_counted() {
        assert_eq!(tools().len(), 38);
        assert!(tools().windows(2).all(|pair| pair[0].0 < pair[1].0));
        for (item, rules) in tools() {
            assert!(std::ptr::eq(tool(*item).unwrap(), rules));
        }
        assert!(tool(item("minecraft:stick")).is_none());
        assert!(tool(u16::MAX).is_none());
    }

    #[test]
    fn pickaxe_rules() {
        let stone_pickaxe = tool(item("minecraft:stone_pickaxe")).unwrap();
        assert_eq!(
            stone_pickaxe.rules[0],
            ToolRule {
                blocks: HolderSet::Tag("minecraft:incorrect_for_stone_tool"),
                speed: None,
                correct_for_drops: Some(false),
            }
        );
        assert_eq!(stone_pickaxe.rules[1].speed, Some(4.0));
        assert_eq!(stone_pickaxe.default_mining_speed, 1.0);
        assert_eq!(stone_pickaxe.damage_per_block, 1);
        assert!(stone_pickaxe.can_destroy_blocks_in_creative);
        assert!(stone_pickaxe.is_correct_for_drops(block("minecraft:stone")));
        assert!(!stone_pickaxe.is_correct_for_drops(block("minecraft:diamond_ore")));
        assert!(!stone_pickaxe.is_correct_for_drops(block("minecraft:dirt")));
        let iron_pickaxe = tool(item("minecraft:iron_pickaxe")).unwrap();
        assert!(iron_pickaxe.is_correct_for_drops(block("minecraft:diamond_ore")));
    }

    #[test]
    fn shears_and_swords_rules() {
        let shears = tool(item("minecraft:shears")).unwrap();
        assert!(shears.is_correct_for_drops(block("minecraft:cobweb")));
        assert!(!shears.is_correct_for_drops(block("minecraft:oak_leaves")));
        let sword = tool(item("minecraft:iron_sword")).unwrap();
        assert!(sword.is_correct_for_drops(block("minecraft:cobweb")));
        assert_eq!(sword.damage_per_block, 2);
        assert!(!sword.can_destroy_blocks_in_creative);
        let mace = tool(item("minecraft:mace")).unwrap();
        assert!(mace.rules.is_empty());
    }
}
