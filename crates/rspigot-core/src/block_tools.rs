use rspigot_blocks::BlockKind;
use rspigot_data::transformers::{self, Condition, Group, Then};
use rspigot_domain::entity::physics::has_tag;
use rspigot_domain::item::{ItemStack, OFFHAND_SLOT};
use rspigot_domain::player::{GameMode, PlayerId};
use rspigot_domain::{BlockPos, BlockStateId};
use rspigot_entities::EntityKind;
use rspigot_items::ItemKind;

use crate::events::{Direction, Hand};
use crate::loot::{self, LootContext, LootEntity};
use crate::plugins::steps::{Change, PlaceStep, ToolEffects, UseItem};
use crate::ports::ClientLink;
use crate::server::Server;

const CLICKED_FACE: &str = "clicked_face";

pub(crate) struct ToolUse {
    pub hand: Hand,
    pub pos: BlockPos,
    pub face: Direction,
    pub clicked: BlockStateId,
    pub sneaking: bool,
    pub game_mode: GameMode,
}

pub(crate) struct Transformation {
    pub state: BlockStateId,
    pub group: &'static Group,
}

fn tool_groups(stack: &ItemStack) -> Option<&'static [Group]> {
    let item = u32::from(stack.item.0);
    let tagged = |name: &str| {
        rspigot_data::tags::tag("minecraft:item", name)
            .is_some_and(|ids| ids.binary_search(&item).is_ok())
    };
    if tagged("minecraft:axes") {
        Some(transformers::axe())
    } else if tagged("minecraft:hoes") {
        Some(transformers::hoe())
    } else if tagged("minecraft:shovels") {
        Some(transformers::shovel())
    } else {
        None
    }
}

fn particle_event(name: &str) -> Option<i32> {
    match name {
        "scrape" => Some(3005),
        "wax_on" => Some(3003),
        "wax_off" => Some(3004),
        _ => None,
    }
}

fn is_plain(group: &Group) -> bool {
    group.transform_type.is_none()
        && group.update_from_neighbors
        && group
            .particle
            .is_none_or(|name| particle_event(name).is_some())
        && group
            .drop_strategy
            .is_none_or(|strategy| strategy == CLICKED_FACE)
}

fn path(name: &str) -> &str {
    name.strip_prefix("minecraft:").unwrap_or(name)
}

fn face_name(face: Direction) -> &'static str {
    match face {
        Direction::Down => "down",
        Direction::Up => "up",
        Direction::North => "north",
        Direction::South => "south",
        Direction::West => "west",
        Direction::East => "east",
    }
}

fn holds(
    condition: &Condition,
    pos: BlockPos,
    block_at: &impl Fn(BlockPos) -> Option<BlockStateId>,
) -> bool {
    match condition {
        Condition::Block(name) => block_at(pos)
            .and_then(BlockStateId::kind)
            .is_some_and(|kind| path(kind.name()) == path(name)),
        Condition::Tag { tag, offset } => block_at(pos.offset(offset.0, offset.1, offset.2))
            .is_some_and(|state| has_tag(state, tag)),
        Condition::AllOf(all) => all.iter().all(|inner| holds(inner, pos, block_at)),
    }
}

pub(crate) fn built_state(old: BlockStateId, then: &Then) -> Option<BlockStateId> {
    match then {
        Then::Copy { to, defaults } => {
            let mut state = BlockKind::from_name(to)?.default_state();
            for (name, value) in *defaults {
                state = state.with_property(name, value).unwrap_or(state);
            }
            for (name, value) in old.properties() {
                state = state.with_property(name, value).unwrap_or(state);
            }
            Some(state)
        }
        Then::Set { to, properties } => {
            let mut state = BlockKind::from_name(to)?.default_state();
            for (name, value) in *properties {
                state = state.with_property(name, value)?;
            }
            Some(state)
        }
    }
}

pub(crate) fn transform(
    groups: &'static [Group],
    face: Direction,
    pos: BlockPos,
    block_at: impl Fn(BlockPos) -> Option<BlockStateId>,
) -> Option<Transformation> {
    let old = block_at(pos)?;
    groups
        .iter()
        .filter(|group| is_plain(group))
        .filter(|group| !group.disallowed_faces.contains(&face_name(face)))
        .find_map(|group| {
            let rule = group
                .rules
                .iter()
                .find(|rule| holds(&rule.when, pos, &block_at))?;
            Some(Transformation {
                state: built_state(old, &rule.then)?,
                group,
            })
        })
}

impl<L: ClientLink> Server<L> {
    pub(crate) fn use_tool_on_block(
        &mut self,
        id: PlayerId,
        held: &ItemStack,
        usage: &ToolUse,
        slot: usize,
        sequence: i32,
    ) -> bool {
        let Some(groups) = tool_groups(held) else {
            return false;
        };
        if matches!(usage.game_mode, GameMode::Adventure | GameMode::Spectator) {
            return false;
        }
        let shield_raised = usage.hand == Hand::Main
            && !usage.sneaking
            && self
                .players
                .get(&id)
                .and_then(|player| player.inventory.get(OFFHAND_SLOT))
                .is_some_and(|stack| stack.item == ItemKind::Shield.id());
        if shield_raised || !self.within_reach(id, usage.pos) || self.is_protected(id, usage.pos) {
            return false;
        }
        let chunks = &self.world.chunks;
        let Some(found) = transform(groups, usage.face, usage.pos, |at| chunks.block(at)) else {
            return false;
        };
        let group = found.group;
        if let (Some(table), Some(_)) = (group.loot, group.drop_strategy) {
            self.drop_transformation_loot(held, usage, table);
        }
        let build = !self.is_protected(id, usage.pos);
        self.ask_place(PlaceStep {
            using: UseItem {
                player: id,
                hand: usage.hand,
                slot,
                stack: held.clone(),
                against: usage.pos,
                sequence,
                game_mode: usage.game_mode,
            },
            changes: vec![Change {
                pos: usage.pos,
                old: usage.clicked,
                new: found.state,
            }],
            build,
            tool: Some(ToolEffects {
                sound: group.sound,
                particle: group.particle.and_then(particle_event),
                damage: i32::from(group.damage),
            }),
            forced: false,
        });
        true
    }

    fn drop_transformation_loot(&mut self, held: &ItemStack, usage: &ToolUse, table: &str) {
        let Some(table) = rspigot_data::loot::table(table) else {
            return;
        };
        let player = LootEntity::of(EntityKind::Player);
        let context = LootContext::block(usage.clicked, Some(held), Some(&player));
        let step = usage.face.offset();
        for stack in loot::table_drops(table, &context, &mut self.rng) {
            self.spawn_item_from_face(usage.pos, step, stack);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::num::NonZeroU8;

    use super::*;

    fn kind(name: &str) -> BlockKind {
        BlockKind::from_name(name).expect("known block")
    }

    fn state(name: &str) -> BlockStateId {
        kind(name).default_state()
    }

    fn with_axis(name: &str, axis: &str) -> BlockStateId {
        state(name).with_property("axis", axis).expect("axis")
    }

    fn world(blocks: &[(BlockPos, &str)]) -> impl Fn(BlockPos) -> Option<BlockStateId> {
        let map: HashMap<(i32, i32, i32), BlockStateId> = blocks
            .iter()
            .map(|(pos, name)| ((pos.x, pos.y, pos.z), state(name)))
            .collect();
        move |pos| map.get(&(pos.x, pos.y, pos.z)).copied()
    }

    const AT: BlockPos = BlockPos::new(0, 64, 0);
    const ABOVE: BlockPos = BlockPos::new(0, 65, 0);

    fn on(
        groups: &'static [Group],
        face: Direction,
        soil: &str,
        above: &str,
    ) -> Option<Transformation> {
        transform(groups, face, AT, world(&[(AT, soil), (ABOVE, above)]))
    }

    fn result(found: Option<Transformation>) -> Option<BlockStateId> {
        found.map(|found| found.state)
    }

    #[test]
    fn stripping_keeps_the_axis_of_the_log() {
        for axis in ["x", "y", "z"] {
            let found = transform(transformers::axe(), Direction::Up, AT, |_| {
                Some(with_axis("oak_log", axis))
            })
            .expect("strippable");
            assert_eq!(found.state.kind(), Some(kind("stripped_oak_log")));
            assert_eq!(found.state.property("axis"), Some(axis));
            assert_eq!(found.group.sound, "minecraft:item.axe.strip");
        }
    }

    #[test]
    fn every_strippable_block_becomes_its_stripped_variant() {
        for rule in transformers::axe()[0].rules {
            let (Condition::Block(from), Then::Copy { to, .. }) = (rule.when, rule.then) else {
                panic!("strip rules match one block");
            };
            let found = transform(transformers::axe(), Direction::Up, AT, |_| {
                Some(with_axis(from, "z"))
            })
            .expect(from);
            assert_eq!(found.state.kind(), BlockKind::from_name(to), "{from}");
            assert_eq!(found.state.property("axis"), Some("z"), "{from}");
        }
    }

    #[test]
    fn blocks_without_a_rule_are_left_alone() {
        assert!(result(on(transformers::axe(), Direction::Up, "stone", "air")).is_none());
        assert!(
            result(on(
                transformers::axe(),
                Direction::Up,
                "stripped_oak_log",
                "air"
            ))
            .is_none()
        );
    }

    #[test]
    fn the_hoe_tills_soil_into_dry_farmland() {
        for soil in ["dirt", "grass_block", "dirt_path"] {
            let state = result(on(transformers::hoe(), Direction::Up, soil, "air")).expect(soil);
            assert_eq!(state.kind(), Some(kind("farmland")), "{soil}");
            assert_eq!(state.property("moisture"), Some("0"), "{soil}");
        }
    }

    #[test]
    fn the_hoe_needs_air_above_and_refuses_the_underside() {
        assert!(result(on(transformers::hoe(), Direction::Up, "dirt", "stone")).is_none());
        assert!(result(on(transformers::hoe(), Direction::Down, "dirt", "air")).is_none());
        assert!(result(on(transformers::hoe(), Direction::East, "dirt", "air")).is_some());
        assert!(result(on(transformers::hoe(), Direction::Up, "dirt", "cave_air")).is_some());
    }

    #[test]
    fn the_hoe_turns_coarse_dirt_into_dirt_and_rooted_dirt_into_dirt() {
        let coarse = result(on(transformers::hoe(), Direction::Up, "coarse_dirt", "air"));
        assert_eq!(coarse, Some(state("dirt")));
        let rooted =
            on(transformers::hoe(), Direction::Down, "rooted_dirt", "stone").expect("rooted");
        assert_eq!(rooted.state, state("dirt"));
        assert_eq!(rooted.group.loot, Some("minecraft:till/rooted_dirt"));
    }

    #[test]
    fn the_shovel_flattens_soil_with_air_above() {
        let path = result(on(
            transformers::shovel(),
            Direction::Up,
            "grass_block",
            "air",
        ));
        assert_eq!(path, Some(state("dirt_path")));
        assert!(
            result(on(
                transformers::shovel(),
                Direction::Up,
                "grass_block",
                "stone"
            ))
            .is_none()
        );
        assert!(
            result(on(
                transformers::shovel(),
                Direction::Down,
                "grass_block",
                "air"
            ))
            .is_none()
        );
    }

    #[test]
    fn scraping_steps_copper_back_one_stage_and_keeps_its_properties() {
        let slab = state("exposed_cut_copper_slab")
            .with_property("type", "top")
            .and_then(|state| state.with_property("waterlogged", "true"))
            .expect("slab properties");
        let found =
            transform(transformers::axe(), Direction::Up, AT, |_| Some(slab)).expect("scrapes");
        assert_eq!(found.state.kind(), Some(kind("cut_copper_slab")));
        assert_eq!(found.state.property("type"), Some("top"));
        assert_eq!(found.state.property("waterlogged"), Some("true"));
        assert_eq!(found.group.particle, Some("scrape"));
        assert_eq!(found.group.sound, "minecraft:item.axe.scrape");
        let block = result(on(
            transformers::axe(),
            Direction::Up,
            "oxidized_copper",
            "air",
        ));
        assert_eq!(block, Some(state("weathered_copper")));
    }

    #[test]
    fn removing_wax_gives_the_unwaxed_block_with_the_wax_off_effects() {
        let found = on(
            transformers::axe(),
            Direction::Up,
            "waxed_oxidized_copper",
            "air",
        )
        .expect("waxed");
        assert_eq!(found.state, state("oxidized_copper"));
        assert_eq!(found.group.particle, Some("wax_off"));
        assert_eq!(found.group.sound, "minecraft:item.axe.wax_off");
    }

    #[test]
    fn every_scrape_and_wax_rule_keeps_the_properties_of_the_block() {
        for index in [1, 4] {
            for rule in transformers::axe()[index].rules {
                let (Condition::Block(from), Then::Copy { to, .. }) = (rule.when, rule.then) else {
                    panic!("copper rules match one block and copy its properties");
                };
                let old = state(from);
                let found = transform(transformers::axe(), Direction::Up, AT, |_| Some(old))
                    .unwrap_or_else(|| panic!("{from} has no transformation"));
                assert_eq!(found.state.kind(), BlockKind::from_name(to), "{from}");
                for (name, value) in old.properties() {
                    if found.state.property(name).is_some() {
                        assert_eq!(found.state.property(name), Some(value), "{from} {name}");
                    }
                }
            }
        }
    }

    #[test]
    fn copper_doors_and_chests_are_not_handled_yet() {
        for name in [
            "exposed_copper_door",
            "exposed_copper_chest",
            "waxed_copper_door",
            "waxed_copper_chest",
        ] {
            assert!(
                transform(transformers::axe(), Direction::Up, AT, |_| Some(state(
                    name
                )))
                .is_none(),
                "{name}"
            );
        }
    }

    #[test]
    fn tools_are_found_by_their_tag() {
        let item = |name: &str| {
            let kind = ItemKind::from_name(name).expect("known item");
            ItemStack::new(kind.id(), NonZeroU8::MIN)
        };
        assert_eq!(
            tool_groups(&item("minecraft:iron_axe")).map(<[Group]>::len),
            Some(7)
        );
        assert_eq!(
            tool_groups(&item("minecraft:netherite_hoe")).map(<[Group]>::len),
            Some(2)
        );
        assert_eq!(
            tool_groups(&item("minecraft:wooden_shovel")).map(<[Group]>::len),
            Some(1)
        );
        assert!(tool_groups(&item("minecraft:iron_pickaxe")).is_none());
        assert!(tool_groups(&item("minecraft:stick")).is_none());
    }
}
