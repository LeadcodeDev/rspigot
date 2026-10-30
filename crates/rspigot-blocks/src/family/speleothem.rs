use super::{BlockBehavior, Face, Gravity};
use crate::{BlockKind, StateId};

pub struct Speleothem;

impl BlockBehavior for Speleothem {
    const GRAVITY: Gravity = Gravity::Speleothem;
}

impl StateId {
    pub fn tip_direction(self) -> Option<Face> {
        if self.kind()?.gravity() != Gravity::Speleothem {
            return None;
        }
        match self.property("vertical_direction")? {
            "up" => Some(Face::Up),
            "down" => Some(Face::Down),
            _ => None,
        }
    }

    pub fn is_speleothem_toward(self, tip: Face) -> bool {
        self.tip_direction() == Some(tip)
    }

    pub fn is_speleothem_tip(self, merge: bool) -> bool {
        self.tip_direction().is_some()
            && match self.property("thickness") {
                Some("tip") => true,
                Some("tip_merge") => merge,
                _ => false,
            }
    }
}

pub fn speleothem_thickness(
    kind: BlockKind,
    tip: Face,
    toward_tip: StateId,
    away_from_tip: StateId,
    merge: bool,
) -> &'static str {
    if toward_tip.is_speleothem_toward(tip.opposite()) && toward_tip.kind() == Some(kind) {
        return if merge || toward_tip.property("thickness") == Some("tip_merge") {
            "tip_merge"
        } else {
            "tip"
        };
    }
    if !toward_tip.is_speleothem_toward(tip) {
        return "tip";
    }
    match toward_tip.property("thickness") {
        Some("tip" | "tip_merge") => "frustum",
        _ if away_from_tip.is_speleothem_toward(tip) => "middle",
        _ => "base",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BlockKind;
    use crate::family::Face;

    fn dripstone(thickness: &str, direction: &str) -> StateId {
        BlockKind::PointedDripstone
            .state_with(&[("thickness", thickness), ("vertical_direction", direction)])
            .unwrap_or_default()
    }

    #[test]
    fn tip_direction_follows_vertical_direction() {
        assert_eq!(dripstone("tip", "down").tip_direction(), Some(Face::Down));
        assert_eq!(dripstone("base", "up").tip_direction(), Some(Face::Up));
        assert_eq!(StateId::AIR.tip_direction(), None);
        assert_eq!(
            BlockKind::SulfurSpike.default_state().tip_direction(),
            Some(Face::Up)
        );
    }

    #[test]
    fn tips_include_merged_tips_only_when_asked() {
        assert!(dripstone("tip", "down").is_speleothem_tip(false));
        assert!(!dripstone("tip_merge", "down").is_speleothem_tip(false));
        assert!(dripstone("tip_merge", "down").is_speleothem_tip(true));
        assert!(!dripstone("frustum", "down").is_speleothem_tip(true));
    }

    #[test]
    fn stalactite_column_reads_base_middle_frustum_tip() {
        let kind = BlockKind::PointedDripstone;
        let stone = BlockKind::Stone.default_state();
        let tip = dripstone("tip", "down");
        let frustum = dripstone("frustum", "down");
        let middle = dripstone("middle", "down");
        assert_eq!(
            speleothem_thickness(kind, Face::Down, StateId::AIR, frustum, false),
            "tip"
        );
        assert_eq!(
            speleothem_thickness(kind, Face::Down, tip, middle, false),
            "frustum"
        );
        assert_eq!(
            speleothem_thickness(kind, Face::Down, frustum, middle, false),
            "middle"
        );
        assert_eq!(
            speleothem_thickness(kind, Face::Down, middle, stone, false),
            "base"
        );
    }

    #[test]
    fn facing_tips_merge() {
        let kind = BlockKind::PointedDripstone;
        let below = dripstone("tip", "up");
        let merged = dripstone("tip_merge", "up");
        assert_eq!(
            speleothem_thickness(kind, Face::Down, below, StateId::AIR, false),
            "tip"
        );
        assert_eq!(
            speleothem_thickness(kind, Face::Down, below, StateId::AIR, true),
            "tip_merge"
        );
        assert_eq!(
            speleothem_thickness(kind, Face::Down, merged, StateId::AIR, false),
            "tip_merge"
        );
        assert_eq!(
            speleothem_thickness(
                BlockKind::SulfurSpike,
                Face::Down,
                below,
                StateId::AIR,
                true
            ),
            "tip"
        );
    }
}
