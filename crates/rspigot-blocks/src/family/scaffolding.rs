use super::{BlockBehavior, Gravity};
use crate::{BlockKind, StateId};

pub const SCAFFOLDING_MAX_DISTANCE: u8 = 7;

const DISTANCES: [&str; 8] = ["0", "1", "2", "3", "4", "5", "6", "7"];

pub struct Scaffolding;

impl BlockBehavior for Scaffolding {
    const GRAVITY: Gravity = Gravity::Scaffolding;
}

impl StateId {
    pub fn scaffolding_distance(self) -> Option<u8> {
        if self.kind()? != BlockKind::Scaffolding {
            return None;
        }
        self.property("distance")?.parse().ok()
    }
}

pub fn scaffolding_settle(
    state: StateId,
    below: StateId,
    below_sturdy: bool,
    sides: [StateId; 4],
) -> StateId {
    let distance = match below.scaffolding_distance() {
        Some(distance) => distance,
        None if below_sturdy => 0,
        None => SCAFFOLDING_MAX_DISTANCE,
    };
    let distance = if distance == 0 {
        0
    } else {
        sides
            .iter()
            .filter_map(|side| side.scaffolding_distance())
            .fold(distance, |nearest, side| {
                nearest.min(side.saturating_add(1))
            })
    };
    let bottom = distance > 0 && below.scaffolding_distance().is_none();
    let value = DISTANCES[usize::from(distance.min(SCAFFOLDING_MAX_DISTANCE))];
    state
        .with_property("distance", value)
        .and_then(|state| state.with_property("bottom", if bottom { "true" } else { "false" }))
        .unwrap_or(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BlockKind;

    fn scaffolding(distance: &str) -> StateId {
        BlockKind::Scaffolding
            .state_with(&[("distance", distance)])
            .unwrap_or_default()
    }

    #[test]
    fn sturdy_ground_gives_distance_zero() {
        let placed = BlockKind::Scaffolding.default_state();
        let settled = scaffolding_settle(
            placed,
            BlockKind::Stone.default_state(),
            true,
            [StateId::AIR; 4],
        );
        assert_eq!(settled.scaffolding_distance(), Some(0));
        assert_eq!(settled.property("bottom"), Some("false"));
    }

    #[test]
    fn scaffolding_below_passes_its_distance_up() {
        let settled = scaffolding_settle(
            scaffolding("0"),
            scaffolding("3"),
            false,
            [StateId::AIR, scaffolding("5"), StateId::AIR, StateId::AIR],
        );
        assert_eq!(settled.scaffolding_distance(), Some(3));
        assert_eq!(settled.property("bottom"), Some("false"));
    }

    #[test]
    fn hanging_scaffolding_counts_from_its_nearest_side() {
        let settled = scaffolding_settle(
            scaffolding("0"),
            StateId::AIR,
            false,
            [
                scaffolding("4"),
                scaffolding("2"),
                StateId::AIR,
                scaffolding("6"),
            ],
        );
        assert_eq!(settled.scaffolding_distance(), Some(3));
        assert_eq!(settled.property("bottom"), Some("true"));
    }

    #[test]
    fn unsupported_scaffolding_reaches_the_limit() {
        let settled = scaffolding_settle(
            scaffolding("0"),
            StateId::AIR,
            false,
            [scaffolding("7"), StateId::AIR, StateId::AIR, StateId::AIR],
        );
        assert_eq!(
            settled.scaffolding_distance(),
            Some(SCAFFOLDING_MAX_DISTANCE)
        );
        assert_eq!(StateId::AIR.scaffolding_distance(), None);
    }
}
