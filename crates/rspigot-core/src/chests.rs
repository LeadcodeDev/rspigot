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

pub(crate) const COPPER_CHESTS: [BlockKind; 8] = [
    BlockKind::CopperChest,
    BlockKind::ExposedCopperChest,
    BlockKind::WeatheredCopperChest,
    BlockKind::OxidizedCopperChest,
    BlockKind::WaxedCopperChest,
    BlockKind::WaxedExposedCopperChest,
    BlockKind::WaxedWeatheredCopperChest,
    BlockKind::WaxedOxidizedCopperChest,
];

pub(crate) fn is_copper_chest(kind: BlockKind) -> bool {
    COPPER_CHESTS.contains(&kind)
}

pub(crate) fn is_pairable(kind: BlockKind) -> bool {
    matches!(kind, BlockKind::Chest | BlockKind::TrappedChest) || is_copper_chest(kind)
}

fn connects(kind: BlockKind, other: BlockKind) -> bool {
    kind == other || (is_copper_chest(kind) && is_copper_chest(other))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Oxidation {
    Unaffected,
    Exposed,
    Weathered,
    Oxidized,
}

impl Oxidation {
    const ORDER: [Oxidation; 4] = [
        Oxidation::Unaffected,
        Oxidation::Exposed,
        Oxidation::Weathered,
        Oxidation::Oxidized,
    ];

    fn rank(self) -> usize {
        Self::ORDER
            .iter()
            .position(|candidate| *candidate == self)
            .unwrap_or_default()
    }
}

pub(crate) fn oxidation_of(kind: BlockKind) -> Option<Oxidation> {
    match kind {
        BlockKind::CopperChest | BlockKind::WaxedCopperChest => Some(Oxidation::Unaffected),
        BlockKind::ExposedCopperChest | BlockKind::WaxedExposedCopperChest => {
            Some(Oxidation::Exposed)
        }
        BlockKind::WeatheredCopperChest | BlockKind::WaxedWeatheredCopperChest => {
            Some(Oxidation::Weathered)
        }
        BlockKind::OxidizedCopperChest | BlockKind::WaxedOxidizedCopperChest => {
            Some(Oxidation::Oxidized)
        }
        _ => None,
    }
}

fn is_waxed(kind: BlockKind) -> bool {
    matches!(
        kind,
        BlockKind::WaxedCopperChest
            | BlockKind::WaxedExposedCopperChest
            | BlockKind::WaxedWeatheredCopperChest
            | BlockKind::WaxedOxidizedCopperChest
    )
}

fn copper_chest_of(oxidation: Oxidation, waxed: bool) -> BlockKind {
    match (oxidation, waxed) {
        (Oxidation::Unaffected, false) => BlockKind::CopperChest,
        (Oxidation::Exposed, false) => BlockKind::ExposedCopperChest,
        (Oxidation::Weathered, false) => BlockKind::WeatheredCopperChest,
        (Oxidation::Oxidized, false) => BlockKind::OxidizedCopperChest,
        (Oxidation::Unaffected, true) => BlockKind::WaxedCopperChest,
        (Oxidation::Exposed, true) => BlockKind::WaxedExposedCopperChest,
        (Oxidation::Weathered, true) => BlockKind::WaxedWeatheredCopperChest,
        (Oxidation::Oxidized, true) => BlockKind::WaxedOxidizedCopperChest,
    }
}

fn with_properties_of(kind: BlockKind, source: BlockStateId) -> BlockStateId {
    ["facing", "type", "waterlogged"]
        .into_iter()
        .fold(kind.default_state(), |state, name| {
            source
                .property(name)
                .and_then(|value| state.with_property(name, value))
                .unwrap_or(state)
        })
}

fn least_oxidized(state: BlockStateId, partner: BlockStateId) -> BlockStateId {
    let (Some(own), Some(theirs)) = (state.kind(), partner.kind()) else {
        return state;
    };
    let (Some(own_oxidation), Some(their_oxidation)) = (oxidation_of(own), oxidation_of(theirs))
    else {
        return state;
    };
    if chest_type(state) == Some(ChestType::Single) {
        return state;
    }
    let mixed_wax = is_waxed(own) != is_waxed(theirs);
    let (own_waxed, their_waxed) = if mixed_wax {
        (false, false)
    } else {
        (is_waxed(own), is_waxed(theirs))
    };
    let chosen = if own_oxidation.rank() <= their_oxidation.rank() {
        copper_chest_of(own_oxidation, own_waxed)
    } else {
        copper_chest_of(their_oxidation, their_waxed)
    };
    with_properties_of(chosen, state)
}

fn unified_with_partner(
    state: BlockStateId,
    pos: BlockPos,
    block_at: &impl Fn(BlockPos) -> Option<BlockStateId>,
) -> BlockStateId {
    let Some(partner) = partner_pos(state, pos).and_then(block_at) else {
        return state;
    };
    least_oxidized(state, partner)
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
    if neighbour.kind().is_some_and(|other| connects(kind, other))
        && chest_type(neighbour) == Some(ChestType::Single)
    {
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
    let placed = with(state, facing, kind_of_chest);
    if is_copper_chest(kind) {
        unified_with_partner(placed, target, &block_at)
    } else {
        placed
    }
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
        let joined =
            changed.filter(|state| state.kind().is_some_and(|other| connects(kind, other)));
        match joined {
            Some(state) => {
                let (Some(own), Some(theirs)) = (chest_type(neighbour), chest_type(state)) else {
                    continue;
                };
                let mut next = neighbour;
                if own == ChestType::Single
                    && theirs != ChestType::Single
                    && facing(neighbour) == facing(state)
                    && connected_direction(state) == Some(from_neighbour.opposite())
                    && let Some(face) = facing(neighbour)
                {
                    next = with(neighbour, face, theirs.opposite());
                }
                if is_copper_chest(kind)
                    && chest_type(next) != Some(ChestType::Single)
                    && connected_direction(next) == Some(from_neighbour)
                    && let Some(joined_kind) = state.kind()
                {
                    next = with_properties_of(joined_kind, next);
                }
                if next != neighbour {
                    fixes.push((at, next));
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

    use rspigot_domain::entity::physics::has_tag;

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

    fn copper(kind: BlockKind, facing: Dir, chest_type: ChestType) -> BlockStateId {
        with(kind.default_state(), facing, chest_type)
    }

    fn placed_copper(
        kind: BlockKind,
        neighbour: BlockKind,
        sneaking: bool,
    ) -> Option<(BlockKind, ChestType)> {
        let blocks = [(FIRST, copper(neighbour, Dir::North, ChestType::Single))];
        let state = placed_state(
            copper(kind, Dir::North, ChestType::Single),
            EAST_OF_FIRST,
            Direction::Up,
            sneaking,
            world(&blocks),
        );
        Some((state.kind()?, chest_type(state)?))
    }

    #[test]
    fn every_copper_chest_pairs_with_every_other_copper_chest() {
        for kind in COPPER_CHESTS {
            for neighbour in COPPER_CHESTS {
                let (_, chest_type) =
                    placed_copper(kind, neighbour, false).expect("a copper chest state");
                assert_eq!(
                    chest_type,
                    ChestType::Right,
                    "{kind:?} next to {neighbour:?}"
                );
            }
        }
    }

    fn single(kind: BlockKind) -> BlockStateId {
        with(kind.default_state(), Dir::North, ChestType::Single)
    }

    fn placed_next_to(kind: BlockKind, neighbour: BlockKind) -> Option<ChestType> {
        let blocks = [(FIRST, single(neighbour))];
        let state = placed_state(
            single(kind),
            EAST_OF_FIRST,
            Direction::Up,
            false,
            world(&blocks),
        );
        chest_type(state)
    }

    #[test]
    fn a_copper_chest_does_not_pair_with_a_plain_or_a_trapped_chest() {
        for other in [BlockKind::Chest, BlockKind::TrappedChest] {
            assert_eq!(
                placed_next_to(BlockKind::CopperChest, other),
                Some(ChestType::Single),
                "copper next to {other:?}"
            );
            assert_eq!(
                placed_next_to(other, BlockKind::CopperChest),
                Some(ChestType::Single),
                "{other:?} next to copper"
            );
        }
    }

    #[test]
    fn a_pair_takes_the_block_of_the_less_oxidized_half() {
        let cases = [
            (
                BlockKind::ExposedCopperChest,
                BlockKind::WeatheredCopperChest,
                BlockKind::ExposedCopperChest,
            ),
            (
                BlockKind::WeatheredCopperChest,
                BlockKind::ExposedCopperChest,
                BlockKind::ExposedCopperChest,
            ),
            (
                BlockKind::OxidizedCopperChest,
                BlockKind::CopperChest,
                BlockKind::CopperChest,
            ),
            (
                BlockKind::CopperChest,
                BlockKind::OxidizedCopperChest,
                BlockKind::CopperChest,
            ),
            (
                BlockKind::WeatheredCopperChest,
                BlockKind::WeatheredCopperChest,
                BlockKind::WeatheredCopperChest,
            ),
        ];
        for (placed_kind, neighbour, expected) in cases {
            let (kind, chest_type) = placed_copper(placed_kind, neighbour, false).expect("state");
            assert_eq!(kind, expected, "{placed_kind:?} next to {neighbour:?}");
            assert_eq!(chest_type, ChestType::Right);
        }
    }

    #[test]
    fn two_waxed_chests_stay_waxed_and_take_the_less_oxidized_block() {
        let (kind, _) = placed_copper(
            BlockKind::WaxedWeatheredCopperChest,
            BlockKind::WaxedOxidizedCopperChest,
            false,
        )
        .expect("state");
        assert_eq!(kind, BlockKind::WaxedWeatheredCopperChest);
        let (kind, _) = placed_copper(
            BlockKind::WaxedOxidizedCopperChest,
            BlockKind::WaxedExposedCopperChest,
            false,
        )
        .expect("state");
        assert_eq!(kind, BlockKind::WaxedExposedCopperChest);
    }

    #[test]
    fn a_waxed_chest_next_to_an_unwaxed_one_loses_its_wax_in_the_pair() {
        let cases = [
            (
                BlockKind::WaxedCopperChest,
                BlockKind::CopperChest,
                BlockKind::CopperChest,
            ),
            (
                BlockKind::ExposedCopperChest,
                BlockKind::WaxedCopperChest,
                BlockKind::CopperChest,
            ),
            (
                BlockKind::WaxedExposedCopperChest,
                BlockKind::OxidizedCopperChest,
                BlockKind::ExposedCopperChest,
            ),
            (
                BlockKind::OxidizedCopperChest,
                BlockKind::WaxedWeatheredCopperChest,
                BlockKind::WeatheredCopperChest,
            ),
        ];
        for (placed_kind, neighbour, expected) in cases {
            let (kind, _) = placed_copper(placed_kind, neighbour, false).expect("state");
            assert_eq!(kind, expected, "{placed_kind:?} next to {neighbour:?}");
        }
    }

    #[test]
    fn a_single_copper_chest_keeps_its_block() {
        let (kind, chest_type) =
            placed_copper(BlockKind::OxidizedCopperChest, BlockKind::CopperChest, true)
                .expect("state");
        assert_eq!(kind, BlockKind::OxidizedCopperChest);
        assert_eq!(chest_type, ChestType::Single);
    }

    #[test]
    fn the_neighbour_of_a_new_copper_half_takes_its_block() {
        let new = copper(BlockKind::ExposedCopperChest, Dir::North, ChestType::Right);
        let blocks = [
            (
                FIRST,
                copper(
                    BlockKind::WeatheredCopperChest,
                    Dir::North,
                    ChestType::Single,
                ),
            ),
            (EAST_OF_FIRST, new),
        ];
        let fixes = neighbour_fixes(EAST_OF_FIRST, Some(new), world(&blocks));
        assert_eq!(
            fixes,
            vec![(
                FIRST,
                copper(BlockKind::ExposedCopperChest, Dir::North, ChestType::Left)
            )]
        );
    }

    #[test]
    fn a_half_follows_the_block_its_partner_changes_into() {
        let left = copper(BlockKind::CopperChest, Dir::North, ChestType::Left);
        let changed = copper(BlockKind::WaxedCopperChest, Dir::North, ChestType::Right);
        let blocks = [(FIRST, left), (EAST_OF_FIRST, changed)];
        let fixes = neighbour_fixes(EAST_OF_FIRST, Some(changed), world(&blocks));
        assert_eq!(
            fixes,
            vec![(
                FIRST,
                copper(BlockKind::WaxedCopperChest, Dir::North, ChestType::Left)
            )]
        );
    }

    #[test]
    fn a_pair_that_is_already_one_block_needs_no_fix() {
        let left = copper(BlockKind::CopperChest, Dir::North, ChestType::Left);
        let right = copper(BlockKind::CopperChest, Dir::North, ChestType::Right);
        let blocks = [(FIRST, left), (EAST_OF_FIRST, right)];
        assert!(neighbour_fixes(EAST_OF_FIRST, Some(right), world(&blocks)).is_empty());
    }

    #[test]
    fn a_copper_half_whose_partner_is_removed_becomes_single() {
        let blocks = [(
            FIRST,
            copper(BlockKind::WeatheredCopperChest, Dir::North, ChestType::Left),
        )];
        let fixes = neighbour_fixes(EAST_OF_FIRST, None, world(&blocks));
        assert_eq!(
            fixes,
            vec![(
                FIRST,
                copper(
                    BlockKind::WeatheredCopperChest,
                    Dir::North,
                    ChestType::Single
                )
            )]
        );
    }

    #[test]
    fn halves_of_two_different_copper_blocks_do_not_make_a_pair() {
        let left = copper(BlockKind::CopperChest, Dir::North, ChestType::Left);
        let right = copper(BlockKind::ExposedCopperChest, Dir::North, ChestType::Right);
        let blocks = [(FIRST, left), (EAST_OF_FIRST, right)];
        assert_eq!(pair_of(FIRST, left, world(&blocks)), None);
    }

    #[test]
    fn the_copper_chest_kinds_are_the_eight_of_the_vanilla_tag() {
        let named: Vec<BlockKind> = BlockKind::ALL
            .into_iter()
            .filter(|kind| kind.name().ends_with("copper_chest"))
            .collect();
        let tagged: Vec<BlockKind> = BlockKind::ALL
            .into_iter()
            .filter(|kind| has_tag(kind.default_state(), "minecraft:copper_chests"))
            .collect();
        assert_eq!(named.len(), COPPER_CHESTS.len());
        assert_eq!(tagged, named);
        assert!(named.iter().all(|kind| is_copper_chest(*kind)));
    }
}
