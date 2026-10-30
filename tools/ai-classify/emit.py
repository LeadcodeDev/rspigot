import sys

HEAD = """use rspigot_blocks::{BlockKind, StateFlags, StateId};

use crate::PathType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BlockClass {
    Plain,
    Trapdoor,
    PowderSnow,
    Damaging,
    StickyHoney,
    Cocoa,
    DamageCautious,
    Burning,
    Campfire,
    WoodenDoor,
    IronDoor,
    Rail,
    Leaves,
    Fence,
    FenceGate,
    SnowLayer,
    Liquid,
    Unpathable,
}

pub(crate) fn path_type_of(state: Option<StateId>) -> PathType {
    let Some(state) = state else {
        return PathType::Blocked;
    };
    if state.is_air() {
        return PathType::Open;
    }
    let Some(kind) = state.kind() else {
        return PathType::Blocked;
    };
    let class = class_of(kind);
    match class {
        BlockClass::Trapdoor => return PathType::Trapdoor,
        BlockClass::PowderSnow => return PathType::PowderSnow,
        BlockClass::Damaging => return PathType::Damaging,
        BlockClass::StickyHoney => return PathType::StickyHoney,
        BlockClass::Cocoa => return PathType::Cocoa,
        BlockClass::DamageCautious => return PathType::DamageCautious,
        _ => {}
    }
    let flags = state.flags();
    if flags.contains(StateFlags::LAVA) {
        return PathType::Lava;
    }
    if is_burning(state, class) {
        return PathType::Fire;
    }
    match class {
        BlockClass::WoodenDoor if is_true(state, "open") => return PathType::DoorOpen,
        BlockClass::IronDoor if is_true(state, "open") => return PathType::DoorOpen,
        BlockClass::WoodenDoor => return PathType::DoorWoodClosed,
        BlockClass::IronDoor => return PathType::DoorIronClosed,
        BlockClass::Rail => return PathType::Rail,
        BlockClass::Leaves => return PathType::Leaves,
        BlockClass::Fence => return PathType::Fence,
        BlockClass::FenceGate if !is_true(state, "open") => return PathType::Fence,
        _ => {}
    }
    if !land_pathfindable(state, kind, class) {
        return PathType::Blocked;
    }
    if flags.contains(StateFlags::WATER) {
        return PathType::Water;
    }
    PathType::Open
}

pub(crate) fn is_pathfindable(state: StateId) -> bool {
    state
        .kind()
        .is_some_and(|kind| land_pathfindable(state, kind, class_of(kind)))
}

pub(crate) fn is_cauldron(state: StateId) -> bool {
    matches!(
        state.kind(),
        Some(
            BlockKind::Cauldron
                | BlockKind::WaterCauldron
                | BlockKind::LavaCauldron
                | BlockKind::PowderSnowCauldron
        )
    )
}

fn is_burning(state: StateId, class: BlockClass) -> bool {
    match class {
        BlockClass::Burning => true,
        BlockClass::Campfire => is_true(state, "lit"),
        _ => false,
    }
}

fn is_true(state: StateId, property: &str) -> bool {
    state.property(property) == Some("true")
}

fn land_pathfindable(state: StateId, kind: BlockKind, class: BlockClass) -> bool {
    let open_shape = !state.flags().contains(StateFlags::FULL_COLLISION_CUBE);
    match class {
        BlockClass::Unpathable | BlockClass::Fence | BlockClass::Campfire | BlockClass::Cocoa => {
            false
        }
        BlockClass::WoodenDoor | BlockClass::IronDoor | BlockClass::FenceGate => {
            is_true(state, "open")
        }
        BlockClass::Trapdoor => match state.property("open") {
            Some(open) => open == "true",
            None => open_shape,
        },
        BlockClass::PowderSnow => true,
        BlockClass::SnowLayer => state
            .property("layers")
            .and_then(|layers| layers.parse::<u8>().ok())
            .is_some_and(|layers| layers < 5),
        BlockClass::Liquid => kind != BlockKind::Lava,
        BlockClass::Damaging => kind != BlockKind::Cactus && open_shape,
        BlockClass::DamageCautious => kind == BlockKind::WitherRose && open_shape,
        BlockClass::Burning => kind != BlockKind::LavaCauldron && open_shape,
        BlockClass::Plain | BlockClass::StickyHoney | BlockClass::Rail | BlockClass::Leaves => {
            open_shape
        }
    }
}

"""

TAIL = """#[cfg(test)]
mod tests {
    use super::*;

    fn state(kind: BlockKind, properties: &[(&str, &str)]) -> Option<StateId> {
        kind.state_with(properties)
    }

    #[test]
    fn simple_blocks() {
        assert_eq!(path_type_of(Some(StateId::AIR)), PathType::Open);
        assert_eq!(path_type_of(None), PathType::Blocked);
        assert_eq!(
            path_type_of(state(BlockKind::Stone, &[])),
            PathType::Blocked
        );
        assert_eq!(
            path_type_of(state(BlockKind::ShortGrass, &[])),
            PathType::Open
        );
        assert_eq!(
            path_type_of(state(BlockKind::WhiteCarpet, &[])),
            PathType::Open
        );
        assert_eq!(path_type_of(state(BlockKind::Water, &[])), PathType::Water);
        assert_eq!(path_type_of(state(BlockKind::Lava, &[])), PathType::Lava);
    }

    #[test]
    fn damaging_and_burning_blocks() {
        assert_eq!(
            path_type_of(state(BlockKind::Cactus, &[])),
            PathType::Damaging
        );
        assert_eq!(
            path_type_of(state(BlockKind::SweetBerryBush, &[])),
            PathType::Damaging
        );
        assert_eq!(path_type_of(state(BlockKind::Fire, &[])), PathType::Fire);
        assert_eq!(
            path_type_of(state(BlockKind::SoulFire, &[])),
            PathType::Fire
        );
        assert_eq!(
            path_type_of(state(BlockKind::MagmaBlock, &[])),
            PathType::Fire
        );
        assert_eq!(
            path_type_of(state(BlockKind::LavaCauldron, &[])),
            PathType::Fire
        );
        assert_eq!(
            path_type_of(state(BlockKind::Campfire, &[("lit", "true")])),
            PathType::Fire
        );
        assert_eq!(
            path_type_of(state(BlockKind::Campfire, &[("lit", "false")])),
            PathType::Blocked
        );
        assert_eq!(
            path_type_of(state(BlockKind::WitherRose, &[])),
            PathType::DamageCautious
        );
        assert_eq!(
            path_type_of(state(BlockKind::PointedDripstone, &[])),
            PathType::DamageCautious
        );
        assert_eq!(
            path_type_of(state(BlockKind::HoneyBlock, &[])),
            PathType::StickyHoney
        );
        assert_eq!(path_type_of(state(BlockKind::Cocoa, &[])), PathType::Cocoa);
        assert_eq!(
            path_type_of(state(BlockKind::PowderSnow, &[])),
            PathType::PowderSnow
        );
    }

    #[test]
    fn doors_gates_and_trapdoors() {
        assert_eq!(
            path_type_of(state(BlockKind::OakDoor, &[("open", "false")])),
            PathType::DoorWoodClosed
        );
        assert_eq!(
            path_type_of(state(BlockKind::CopperDoor, &[("open", "false")])),
            PathType::DoorWoodClosed
        );
        assert_eq!(
            path_type_of(state(BlockKind::IronDoor, &[("open", "false")])),
            PathType::DoorIronClosed
        );
        assert_eq!(
            path_type_of(state(BlockKind::IronDoor, &[("open", "true")])),
            PathType::DoorOpen
        );
        assert_eq!(
            path_type_of(state(BlockKind::OakFenceGate, &[("open", "false")])),
            PathType::Fence
        );
        assert_eq!(
            path_type_of(state(BlockKind::OakFenceGate, &[("open", "true")])),
            PathType::Open
        );
        assert_eq!(
            path_type_of(state(BlockKind::IronTrapdoor, &[])),
            PathType::Trapdoor
        );
        assert_eq!(
            path_type_of(state(BlockKind::LilyPad, &[])),
            PathType::Trapdoor
        );
        assert_eq!(
            path_type_of(state(BlockKind::BigDripleaf, &[])),
            PathType::Trapdoor
        );
    }

    #[test]
    fn fences_walls_leaves_rails() {
        assert_eq!(
            path_type_of(state(BlockKind::OakFence, &[])),
            PathType::Fence
        );
        assert_eq!(
            path_type_of(state(BlockKind::NetherBrickFence, &[])),
            PathType::Fence
        );
        assert_eq!(
            path_type_of(state(BlockKind::CobblestoneWall, &[])),
            PathType::Fence
        );
        assert_eq!(
            path_type_of(state(BlockKind::OakLeaves, &[])),
            PathType::Leaves
        );
        assert_eq!(
            path_type_of(state(BlockKind::AzaleaLeaves, &[])),
            PathType::Leaves
        );
        assert_eq!(path_type_of(state(BlockKind::Rail, &[])), PathType::Rail);
        assert_eq!(
            path_type_of(state(BlockKind::PoweredRail, &[])),
            PathType::Rail
        );
    }

    #[test]
    fn unpathable_shapes_block() {
        for kind in [
            BlockKind::OakStairs,
            BlockKind::OakSlab,
            BlockKind::RedBed,
            BlockKind::Chest,
            BlockKind::Cauldron,
            BlockKind::FlowerPot,
            BlockKind::Farmland,
            BlockKind::DirtPath,
            BlockKind::SoulSand,
            BlockKind::Mud,
            BlockKind::GlassPane,
            BlockKind::IronBars,
            BlockKind::Anvil,
            BlockKind::Lantern,
        ] {
            assert_eq!(
                path_type_of(state(kind, &[])),
                PathType::Blocked,
                "{kind:?}"
            );
        }
        assert_eq!(
            path_type_of(state(BlockKind::OakSlab, &[("waterlogged", "true")])),
            PathType::Blocked
        );
        assert_eq!(
            path_type_of(state(BlockKind::Ladder, &[("waterlogged", "true")])),
            PathType::Water
        );
    }

    #[test]
    fn snow_layers_up_to_four_are_open() {
        assert_eq!(
            path_type_of(state(BlockKind::Snow, &[("layers", "4")])),
            PathType::Open
        );
        assert_eq!(
            path_type_of(state(BlockKind::Snow, &[("layers", "5")])),
            PathType::Blocked
        );
    }

    #[test]
    fn pathfindable_matches_vanilla_overrides() {
        let check = |kind: BlockKind, properties: &[(&str, &str)]| {
            state(kind, properties).is_some_and(is_pathfindable)
        };
        assert!(check(BlockKind::Air, &[]));
        assert!(!check(BlockKind::Stone, &[]));
        assert!(check(BlockKind::Water, &[]));
        assert!(!check(BlockKind::Lava, &[]));
        assert!(check(BlockKind::PowderSnow, &[]));
        assert!(!check(BlockKind::Cactus, &[]));
        assert!(check(BlockKind::SweetBerryBush, &[]));
        assert!(check(BlockKind::OakTrapdoor, &[("open", "true")]));
        assert!(!check(BlockKind::OakTrapdoor, &[("open", "false")]));
        assert!(check(BlockKind::LilyPad, &[]));
        assert!(!check(BlockKind::PointedDripstone, &[]));
        assert!(check(BlockKind::Fire, &[]));
        assert!(!check(BlockKind::LavaCauldron, &[]));
        assert!(is_cauldron(BlockKind::WaterCauldron.default_state()));
        assert!(!is_cauldron(BlockKind::Stone.default_state()));
    }
}
"""


def fail(message):
    sys.exit("ai-classify emit: " + message)


def main():
    if len(sys.argv) != 3:
        fail("usage: emit.py TABLE OUT")
    table, out = sys.argv[1:3]
    with open(table, encoding="utf-8") as handle:
        body = handle.read()
    if not body.startswith("pub(crate) const fn class_of("):
        fail("unexpected table " + table)
    with open(out, "w", encoding="utf-8") as handle:
        handle.write(HEAD + body + "\n" + TAIL)


main()
