use rspigot_blocks::BlockKind;
use rspigot_data::transformers::{Group, Rule, axe};
use rspigot_domain::item::{ItemStack, OFFHAND_SLOT};
use rspigot_domain::player::{GameMode, PlayerId};
use rspigot_domain::{BlockPos, BlockStateId};
use rspigot_entities::SoundSource;
use rspigot_items::ItemKind;

use crate::events::Hand;
use crate::ports::ClientLink;
use crate::server::Server;

const AXES_TAG: &str = "minecraft:axes";

fn is_axe(stack: &ItemStack) -> bool {
    rspigot_data::tags::tag("minecraft:item", AXES_TAG)
        .is_some_and(|ids| ids.binary_search(&u32::from(stack.item.0)).is_ok())
}

fn is_plain(group: &Group) -> bool {
    group.particle.is_none() && group.transform_type.is_none() && group.update_from_neighbors
}

pub(crate) fn transformed_state(old: BlockStateId, rule: &Rule) -> Option<BlockStateId> {
    let mut state = BlockKind::from_name(rule.to)?.default_state();
    for (name, value) in rule.defaults {
        state = state.with_property(name, value).unwrap_or(state);
    }
    for (name, value) in old.properties() {
        state = state.with_property(name, value).unwrap_or(state);
    }
    Some(state)
}

pub(crate) fn axe_transform(old: BlockStateId) -> Option<(BlockStateId, &'static str, u8)> {
    let kind = old.kind()?;
    axe()
        .iter()
        .filter(|group| is_plain(group))
        .find_map(|group| {
            let rule = group.rule_for(kind.name())?;
            Some((transformed_state(old, rule)?, group.sound, group.damage))
        })
}

impl<L: ClientLink> Server<L> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn use_axe_on_block(
        &mut self,
        id: PlayerId,
        hand: Hand,
        pos: BlockPos,
        clicked: BlockStateId,
        held: &ItemStack,
        sneaking: bool,
        game_mode: GameMode,
    ) -> bool {
        if !is_axe(held) || matches!(game_mode, GameMode::Adventure | GameMode::Spectator) {
            return false;
        }
        let shield_raised = hand == Hand::Main
            && !sneaking
            && self
                .players
                .get(&id)
                .and_then(|player| player.inventory.get(OFFHAND_SLOT))
                .is_some_and(|stack| stack.item == ItemKind::Shield.id());
        if shield_raised || !self.within_reach(id, pos) || self.is_protected(id, pos) {
            return false;
        }
        let Some((state, sound, damage)) = axe_transform(clicked) else {
            return false;
        };
        self.set_block(pos, state);
        self.play_sound(Some(id), pos.center(), sound, SoundSource::Blocks, 1.0, 1.0);
        self.hurt_and_break(id, hand, i32::from(damage));
        true
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU8;

    use super::*;

    fn kind(name: &str) -> BlockKind {
        BlockKind::from_name(name).expect("known block")
    }

    fn with_axis(name: &str, axis: &str) -> BlockStateId {
        kind(name)
            .default_state()
            .with_property("axis", axis)
            .expect("axis property")
    }

    #[test]
    fn stripping_keeps_the_axis_of_the_log() {
        for axis in ["x", "y", "z"] {
            let (state, sound, _) = axe_transform(with_axis("oak_log", axis)).expect("strippable");
            assert_eq!(state.kind(), Some(kind("stripped_oak_log")));
            assert_eq!(state.property("axis"), Some(axis));
            assert_eq!(sound, "minecraft:item.axe.strip");
        }
    }

    #[test]
    fn every_strippable_block_becomes_its_stripped_variant() {
        for rule in axe()[0].rules {
            let (state, _, _) = axe_transform(with_axis(rule.from, "z")).expect(rule.from);
            assert_eq!(state.kind(), BlockKind::from_name(rule.to), "{}", rule.from);
            assert_eq!(state.property("axis"), Some("z"), "{}", rule.from);
        }
    }

    #[test]
    fn blocks_without_a_rule_are_left_alone() {
        assert!(axe_transform(kind("stone").default_state()).is_none());
        assert!(axe_transform(kind("stripped_oak_log").default_state()).is_none());
    }

    #[test]
    fn only_axes_strip() {
        let item = |name: &str| {
            let kind = ItemKind::from_name(name).expect("known item");
            ItemStack::new(kind.id(), NonZeroU8::MIN)
        };
        assert!(is_axe(&item("minecraft:iron_axe")));
        assert!(is_axe(&item("minecraft:netherite_axe")));
        assert!(!is_axe(&item("minecraft:iron_pickaxe")));
        assert!(!is_axe(&item("minecraft:stick")));
    }
}
