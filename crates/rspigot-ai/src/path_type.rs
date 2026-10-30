#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum PathType {
    Blocked,
    Open,
    Walkable,
    WalkableDoor,
    Trapdoor,
    PowderSnow,
    OnTopOfPowderSnow,
    Fence,
    Lava,
    Water,
    WaterBorder,
    Rail,
    UnpassableRail,
    FireInNeighbor,
    Fire,
    DamagingInNeighbor,
    Damaging,
    DoorOpen,
    DoorWoodClosed,
    DoorIronClosed,
    Breach,
    Leaves,
    StickyHoney,
    Cocoa,
    DamageCautious,
    OnTopOfTrapdoor,
    BigMobsCloseToDanger,
}

impl PathType {
    pub const COUNT: usize = 27;

    pub const ALL: [Self; Self::COUNT] = [
        Self::Blocked,
        Self::Open,
        Self::Walkable,
        Self::WalkableDoor,
        Self::Trapdoor,
        Self::PowderSnow,
        Self::OnTopOfPowderSnow,
        Self::Fence,
        Self::Lava,
        Self::Water,
        Self::WaterBorder,
        Self::Rail,
        Self::UnpassableRail,
        Self::FireInNeighbor,
        Self::Fire,
        Self::DamagingInNeighbor,
        Self::Damaging,
        Self::DoorOpen,
        Self::DoorWoodClosed,
        Self::DoorIronClosed,
        Self::Breach,
        Self::Leaves,
        Self::StickyHoney,
        Self::Cocoa,
        Self::DamageCautious,
        Self::OnTopOfTrapdoor,
        Self::BigMobsCloseToDanger,
    ];

    pub const fn id(self) -> u8 {
        self as u8
    }

    pub fn from_id(id: u8) -> Option<Self> {
        Self::ALL.get(usize::from(id)).copied()
    }

    pub const fn malus(self) -> f32 {
        match self {
            Self::Blocked
            | Self::PowderSnow
            | Self::Fence
            | Self::Lava
            | Self::UnpassableRail
            | Self::Damaging
            | Self::DoorWoodClosed
            | Self::DoorIronClosed
            | Self::Leaves => -1.0,
            Self::Water
            | Self::WaterBorder
            | Self::FireInNeighbor
            | Self::DamagingInNeighbor
            | Self::StickyHoney => 8.0,
            Self::Fire => 16.0,
            Self::Breach | Self::BigMobsCloseToDanger => 4.0,
            Self::Open
            | Self::Walkable
            | Self::WalkableDoor
            | Self::Trapdoor
            | Self::OnTopOfPowderSnow
            | Self::Rail
            | Self::DoorOpen
            | Self::Cocoa
            | Self::DamageCautious
            | Self::OnTopOfTrapdoor => 0.0,
        }
    }

    pub(crate) const fn bit(self) -> u32 {
        1 << self as u32
    }

    pub(crate) const fn has_partial_collision(self) -> bool {
        matches!(
            self,
            Self::Fence | Self::DoorWoodClosed | Self::DoorIronClosed
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_follow_vanilla_ordinals() {
        for (index, kind) in PathType::ALL.iter().enumerate() {
            assert_eq!(usize::from(kind.id()), index);
            assert_eq!(PathType::from_id(kind.id()), Some(*kind));
        }
        assert_eq!(PathType::from_id(27), None);
        assert_eq!(PathType::BigMobsCloseToDanger.id(), 26);
    }

    #[test]
    fn malus_matches_vanilla_table() {
        let expected: [f32; PathType::COUNT] = [
            -1.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, -1.0, -1.0, 8.0, 8.0, 0.0, -1.0, 8.0, 16.0, 8.0,
            -1.0, 0.0, -1.0, -1.0, 4.0, -1.0, 8.0, 0.0, 0.0, 0.0, 4.0,
        ];
        let actual: Vec<f32> = PathType::ALL.iter().map(|kind| kind.malus()).collect();
        assert_eq!(actual, expected);
    }
}
