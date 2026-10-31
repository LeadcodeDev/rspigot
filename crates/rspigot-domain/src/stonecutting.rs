use std::sync::OnceLock;

use rspigot_items::{Craft, ItemId, ItemKind, ItemSet};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cut {
    pub id: &'static str,
    pub ingredient: ItemSet,
    pub result: ItemKind,
    pub count: u8,
}

pub fn recipes() -> &'static [Cut] {
    static CUTS: OnceLock<Vec<Cut>> = OnceLock::new();
    CUTS.get_or_init(|| {
        let mut cuts: Vec<Cut> = ItemKind::ALL
            .into_iter()
            .flat_map(|owner| {
                owner.crafts().iter().filter_map(move |craft| match *craft {
                    Craft::Stonecutting {
                        id,
                        ingredient,
                        count,
                    } => Some(Cut {
                        id,
                        ingredient,
                        result: owner,
                        count,
                    }),
                    _ => None,
                })
            })
            .collect();
        cuts.sort_by(|left, right| left.id.cmp(right.id));
        cuts
    })
}

pub fn recipes_for(item: ItemId) -> Vec<&'static Cut> {
    recipes()
        .iter()
        .filter(|cut| cut.ingredient.contains(item))
        .collect()
}

pub fn accepts(item: ItemId) -> bool {
    recipes().iter().any(|cut| cut.ingredient.contains(item))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(item: ItemKind) -> Vec<&'static str> {
        recipes_for(item.id()).iter().map(|cut| cut.id).collect()
    }

    #[test]
    fn every_recipe_of_the_datapack_is_listed_once() {
        assert_eq!(recipes().len(), 351);
        let mut ids: Vec<_> = recipes().iter().map(|cut| cut.id).collect();
        ids.dedup();
        assert_eq!(ids.len(), 351);
    }

    #[test]
    fn the_list_follows_the_recipe_name_order_of_the_game() {
        assert_eq!(
            names(ItemKind::Stone),
            [
                "minecraft:chiseled_stone_bricks_from_stone_stonecutting",
                "minecraft:cobblestone_from_stone_stonecutting",
                "minecraft:cobblestone_slab_from_stone_stonecutting",
                "minecraft:cobblestone_stairs_from_stone_stonecutting",
                "minecraft:cobblestone_wall_from_stone_stonecutting",
                "minecraft:stone_brick_slab_from_stone_stonecutting",
                "minecraft:stone_brick_stairs_from_stone_stonecutting",
                "minecraft:stone_brick_wall_from_stone_stonecutting",
                "minecraft:stone_bricks_from_stone_stonecutting",
                "minecraft:stone_slab_from_stone_stonecutting",
                "minecraft:stone_stairs_from_stone_stonecutting",
            ]
        );
    }

    #[test]
    fn a_recipe_makes_the_count_of_its_result() {
        let slab = recipes_for(ItemKind::Stone.id())
            .into_iter()
            .find(|cut| cut.result == ItemKind::StoneSlab)
            .unwrap();
        assert_eq!(slab.count, 2);
        assert!(accepts(ItemKind::Stone.id()));
        assert!(!accepts(ItemKind::Dirt.id()));
        assert!(recipes_for(ItemKind::Dirt.id()).is_empty());
    }
}
