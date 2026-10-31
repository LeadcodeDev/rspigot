use rspigot_blocks::BlockKind;
use rspigot_domain::{BlockPos, BlockStateId};

use crate::events::Direction;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Dir {
    North,
    East,
    South,
    West,
}

impl Dir {
    pub(crate) const ALL: [Dir; 4] = [Dir::North, Dir::East, Dir::South, Dir::West];

    pub(crate) fn parse(name: &str) -> Option<Self> {
        match name {
            "north" => Some(Self::North),
            "east" => Some(Self::East),
            "south" => Some(Self::South),
            "west" => Some(Self::West),
            _ => None,
        }
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::North => "north",
            Self::East => "east",
            Self::South => "south",
            Self::West => "west",
        }
    }

    pub(crate) fn clockwise(self) -> Self {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
        }
    }

    pub(crate) fn counter_clockwise(self) -> Self {
        match self {
            Self::North => Self::West,
            Self::West => Self::South,
            Self::South => Self::East,
            Self::East => Self::North,
        }
    }

    pub(crate) fn opposite(self) -> Self {
        match self {
            Self::North => Self::South,
            Self::South => Self::North,
            Self::East => Self::West,
            Self::West => Self::East,
        }
    }

    pub(crate) fn step(self) -> (i32, i32) {
        match self {
            Self::North => (0, -1),
            Self::East => (1, 0),
            Self::South => (0, 1),
            Self::West => (-1, 0),
        }
    }

    pub(crate) fn is_along_x(self) -> bool {
        matches!(self, Self::East | Self::West)
    }

    pub(crate) fn from_face(face: Direction) -> Option<Self> {
        match face {
            Direction::North => Some(Self::North),
            Direction::East => Some(Self::East),
            Direction::South => Some(Self::South),
            Direction::West => Some(Self::West),
            Direction::Up | Direction::Down => None,
        }
    }

    pub(crate) fn offset(self, pos: BlockPos) -> BlockPos {
        let (dx, dz) = self.step();
        pos.offset(dx, 0, dz)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChestType {
    Single,
    Left,
    Right,
}

impl ChestType {
    fn parse(name: &str) -> Option<Self> {
        match name {
            "single" => Some(Self::Single),
            "left" => Some(Self::Left),
            "right" => Some(Self::Right),
            _ => None,
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Single => "single",
            Self::Left => "left",
            Self::Right => "right",
        }
    }

    fn opposite(self) -> Self {
        match self {
            Self::Single => Self::Single,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}

pub(crate) fn is_pairable(kind: BlockKind) -> bool {
    matches!(kind, BlockKind::Chest | BlockKind::TrappedChest)
}

pub(crate) fn chest_type(state: BlockStateId) -> Option<ChestType> {
    state.property("type").and_then(ChestType::parse)
}

pub(crate) fn facing(state: BlockStateId) -> Option<Dir> {
    state.property("facing").and_then(Dir::parse)
}

pub(crate) fn connected_direction(state: BlockStateId) -> Option<Dir> {
    let facing = facing(state)?;
    Some(if chest_type(state) == Some(ChestType::Left) {
        facing.clockwise()
    } else {
        facing.counter_clockwise()
    })
}

pub(crate) fn partner_pos(state: BlockStateId, pos: BlockPos) -> Option<BlockPos> {
    match chest_type(state)? {
        ChestType::Single => None,
        ChestType::Left | ChestType::Right => connected_direction(state).map(|dir| dir.offset(pos)),
    }
}

pub(crate) fn pair_of(
    pos: BlockPos,
    state: BlockStateId,
    block_at: impl Fn(BlockPos) -> Option<BlockStateId>,
) -> Option<(BlockPos, BlockPos)> {
    let partner = partner_pos(state, pos)?;
    let other = block_at(partner)?;
    if other.kind() != state.kind() || facing(other) != facing(state) {
        return None;
    }
    let mine = chest_type(state)?;
    if chest_type(other)? != mine.opposite() {
        return None;
    }
    Some(if mine == ChestType::Right {
        (pos, partner)
    } else {
        (partner, pos)
    })
}

fn with(state: BlockStateId, facing: Dir, chest_type: ChestType) -> BlockStateId {
    let faced = state
        .with_property("facing", facing.name())
        .unwrap_or(state);
    faced
        .with_property("type", chest_type.name())
        .unwrap_or(faced)
}

fn candidate_partner_facing(
    kind: BlockKind,
    pos: BlockPos,
    toward: Dir,
    block_at: &impl Fn(BlockPos) -> Option<BlockStateId>,
) -> Option<Dir> {
    let neighbour = block_at(toward.offset(pos))?;
    if neighbour.kind() == Some(kind) && chest_type(neighbour) == Some(ChestType::Single) {
        facing(neighbour)
    } else {
        None
    }
}

pub(crate) fn placed_state(
    state: BlockStateId,
    target: BlockPos,
    face: Direction,
    sneaking: bool,
    block_at: impl Fn(BlockPos) -> Option<BlockStateId>,
) -> BlockStateId {
    let Some(kind) = state.kind().filter(|kind| is_pairable(*kind)) else {
        return state;
    };
    let Some(mut facing) = facing(state) else {
        return state;
    };
    let mut kind_of_chest = ChestType::Single;
    if sneaking
        && let Some(clicked) = Dir::from_face(face)
        && let Some(partner) = candidate_partner_facing(kind, target, clicked.opposite(), &block_at)
        && partner.is_along_x() != clicked.is_along_x()
    {
        facing = partner;
        kind_of_chest = if partner.counter_clockwise() == clicked.opposite() {
            ChestType::Right
        } else {
            ChestType::Left
        };
    }
    if kind_of_chest == ChestType::Single && !sneaking {
        kind_of_chest = if candidate_partner_facing(kind, target, facing.clockwise(), &block_at)
            == Some(facing)
        {
            ChestType::Left
        } else if candidate_partner_facing(kind, target, facing.counter_clockwise(), &block_at)
            == Some(facing)
        {
            ChestType::Right
        } else {
            ChestType::Single
        };
    }
    with(state, facing, kind_of_chest)
}

pub(crate) fn neighbour_fixes(
    pos: BlockPos,
    changed: Option<BlockStateId>,
    block_at: impl Fn(BlockPos) -> Option<BlockStateId>,
) -> Vec<(BlockPos, BlockStateId)> {
    let mut fixes = Vec::new();
    for toward in Dir::ALL {
        let at = toward.offset(pos);
        let Some(neighbour) = block_at(at) else {
            continue;
        };
        let Some(kind) = neighbour.kind().filter(|kind| is_pairable(*kind)) else {
            continue;
        };
        let from_neighbour = toward.opposite();
        let joined = changed.filter(|state| state.kind() == Some(kind));
        match joined {
            Some(state) => {
                let (Some(own), Some(theirs)) = (chest_type(neighbour), chest_type(state)) else {
                    continue;
                };
                if own == ChestType::Single
                    && theirs != ChestType::Single
                    && facing(neighbour) == facing(state)
                    && connected_direction(state) == Some(from_neighbour.opposite())
                    && let Some(face) = facing(neighbour)
                {
                    fixes.push((at, with(neighbour, face, theirs.opposite())));
                }
            }
            None => {
                if chest_type(neighbour) != Some(ChestType::Single)
                    && connected_direction(neighbour) == Some(from_neighbour)
                    && let Some(face) = facing(neighbour)
                {
                    fixes.push((at, with(neighbour, face, ChestType::Single)));
                }
            }
        }
    }
    fixes
}

impl<L: crate::ports::ClientLink> crate::server::Server<L> {
    pub(crate) fn fix_chest_pairs(&mut self, pos: BlockPos, old: BlockStateId, new: BlockStateId) {
        let involves = |state: BlockStateId| state.kind().is_some_and(is_pairable);
        if !involves(old) && !involves(new) {
            return;
        }
        let chunks = &self.world.chunks;
        let fixes = neighbour_fixes(pos, Some(new), |at| chunks.block(at));
        for (at, state) in fixes {
            self.set_block(at, state);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn chest(facing: Dir, kind: ChestType) -> BlockStateId {
        with(BlockKind::Chest.default_state(), facing, kind)
    }

    fn trapped(facing: Dir, kind: ChestType) -> BlockStateId {
        with(BlockKind::TrappedChest.default_state(), facing, kind)
    }

    fn world(blocks: &[(BlockPos, BlockStateId)]) -> impl Fn(BlockPos) -> Option<BlockStateId> {
        let map: HashMap<(i32, i32, i32), BlockStateId> = blocks
            .iter()
            .map(|(pos, state)| ((pos.x, pos.y, pos.z), *state))
            .collect();
        move |pos| map.get(&(pos.x, pos.y, pos.z)).copied()
    }

    const FIRST: BlockPos = BlockPos::new(2, 64, 2);
    const EAST_OF_FIRST: BlockPos = BlockPos::new(3, 64, 2);

    fn placed(
        facing_of_new: Dir,
        target: BlockPos,
        face: Direction,
        sneaking: bool,
        blocks: &[(BlockPos, BlockStateId)],
    ) -> BlockStateId {
        placed_state(
            chest(facing_of_new, ChestType::Single),
            target,
            face,
            sneaking,
            world(blocks),
        )
    }

    #[test]
    fn a_chest_next_to_a_single_chest_of_the_same_facing_pairs_with_it() {
        let blocks = [(FIRST, chest(Dir::North, ChestType::Single))];
        let state = placed(Dir::North, EAST_OF_FIRST, Direction::Up, false, &blocks);
        assert_eq!(chest_type(state), Some(ChestType::Right));
        assert_eq!(facing(state), Some(Dir::North));
        let west_blocks = [(EAST_OF_FIRST, chest(Dir::North, ChestType::Single))];
        let state = placed(Dir::North, FIRST, Direction::Up, false, &west_blocks);
        assert_eq!(chest_type(state), Some(ChestType::Left));
    }

    #[test]
    fn sneaking_places_a_single_chest() {
        let blocks = [(FIRST, chest(Dir::North, ChestType::Single))];
        let state = placed(Dir::North, EAST_OF_FIRST, Direction::Up, true, &blocks);
        assert_eq!(chest_type(state), Some(ChestType::Single));
    }

    #[test]
    fn sneaking_against_the_side_of_a_chest_joins_it_and_takes_its_facing() {
        let blocks = [(FIRST, chest(Dir::North, ChestType::Single))];
        let state = placed(Dir::South, EAST_OF_FIRST, Direction::East, true, &blocks);
        assert_eq!(facing(state), Some(Dir::North));
        assert_eq!(chest_type(state), Some(ChestType::Right));
    }

    #[test]
    fn chests_of_another_kind_or_facing_do_not_pair() {
        let trapped_neighbour = [(FIRST, trapped(Dir::North, ChestType::Single))];
        let state = placed(
            Dir::North,
            EAST_OF_FIRST,
            Direction::Up,
            false,
            &trapped_neighbour,
        );
        assert_eq!(chest_type(state), Some(ChestType::Single));
        let turned = [(FIRST, chest(Dir::East, ChestType::Single))];
        let state = placed(Dir::North, EAST_OF_FIRST, Direction::Up, false, &turned);
        assert_eq!(chest_type(state), Some(ChestType::Single));
    }

    #[test]
    fn a_chest_that_is_already_half_of_a_pair_is_not_a_candidate() {
        let blocks = [(FIRST, chest(Dir::North, ChestType::Left))];
        let state = placed(Dir::North, EAST_OF_FIRST, Direction::Up, false, &blocks);
        assert_eq!(chest_type(state), Some(ChestType::Single));
    }

    #[test]
    fn a_pair_is_first_the_right_half_then_the_left_half() {
        let left = chest(Dir::North, ChestType::Left);
        let right = chest(Dir::North, ChestType::Right);
        let blocks = [(FIRST, left), (EAST_OF_FIRST, right)];
        let expected = Some((EAST_OF_FIRST, FIRST));
        assert_eq!(pair_of(FIRST, left, world(&blocks)), expected);
        assert_eq!(pair_of(EAST_OF_FIRST, right, world(&blocks)), expected);
        let broken = [(FIRST, left)];
        assert_eq!(pair_of(FIRST, left, world(&broken)), None);
        let turned = [
            (FIRST, left),
            (EAST_OF_FIRST, chest(Dir::East, ChestType::Right)),
        ];
        assert_eq!(pair_of(FIRST, left, world(&turned)), None);
    }

    #[test]
    fn the_partner_of_each_half_is_on_the_connected_side() {
        let left = chest(Dir::North, ChestType::Left);
        let right = chest(Dir::North, ChestType::Right);
        assert_eq!(partner_pos(left, FIRST), Some(EAST_OF_FIRST));
        assert_eq!(partner_pos(right, EAST_OF_FIRST), Some(FIRST));
        assert_eq!(
            partner_pos(chest(Dir::North, ChestType::Single), FIRST),
            None
        );
        assert_eq!(connected_direction(left), Some(Dir::East));
        assert_eq!(connected_direction(right), Some(Dir::West));
    }

    #[test]
    fn placing_the_second_chest_turns_the_first_into_the_other_half() {
        let new = chest(Dir::North, ChestType::Right);
        let blocks = [
            (FIRST, chest(Dir::North, ChestType::Single)),
            (EAST_OF_FIRST, new),
        ];
        let fixes = neighbour_fixes(EAST_OF_FIRST, Some(new), world(&blocks));
        assert_eq!(fixes, vec![(FIRST, chest(Dir::North, ChestType::Left))]);
    }

    #[test]
    fn removing_one_half_turns_the_other_back_into_a_single_chest() {
        let blocks = [(FIRST, chest(Dir::North, ChestType::Left))];
        let fixes = neighbour_fixes(EAST_OF_FIRST, None, world(&blocks));
        assert_eq!(fixes, vec![(FIRST, chest(Dir::North, ChestType::Single))]);
        let unrelated = neighbour_fixes(BlockPos::new(2, 64, 5), None, world(&blocks));
        assert!(unrelated.is_empty());
    }

    #[test]
    fn a_single_chest_next_to_a_removed_block_is_left_alone() {
        let blocks = [(FIRST, chest(Dir::North, ChestType::Single))];
        assert!(neighbour_fixes(EAST_OF_FIRST, None, world(&blocks)).is_empty());
    }
}
