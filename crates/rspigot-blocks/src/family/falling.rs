use super::{
    BlockBehavior, Gravity, PlaceContext, Placement, clockwise, horizontal_facing, placed, with,
};
use crate::{BlockKind, StateId};

pub struct Falling;

impl BlockBehavior for Falling {
    const GRAVITY: Gravity = Gravity::Falling;
}

pub struct ConcretePowder;

impl BlockBehavior for ConcretePowder {
    const GRAVITY: Gravity = Gravity::ConcretePowder;
}

pub struct Anvil;

impl BlockBehavior for Anvil {
    const GRAVITY: Gravity = Gravity::Anvil;

    fn place(context: &PlaceContext) -> Option<Placement> {
        placed(with(
            context.default_state,
            "facing",
            clockwise(horizontal_facing(context.yaw)),
        ))
    }
}

impl BlockKind {
    pub const fn concrete(self) -> Option<Self> {
        Some(match self {
            Self::WhiteConcretePowder => Self::WhiteConcrete,
            Self::OrangeConcretePowder => Self::OrangeConcrete,
            Self::MagentaConcretePowder => Self::MagentaConcrete,
            Self::LightBlueConcretePowder => Self::LightBlueConcrete,
            Self::YellowConcretePowder => Self::YellowConcrete,
            Self::LimeConcretePowder => Self::LimeConcrete,
            Self::PinkConcretePowder => Self::PinkConcrete,
            Self::GrayConcretePowder => Self::GrayConcrete,
            Self::LightGrayConcretePowder => Self::LightGrayConcrete,
            Self::CyanConcretePowder => Self::CyanConcrete,
            Self::PurpleConcretePowder => Self::PurpleConcrete,
            Self::BlueConcretePowder => Self::BlueConcrete,
            Self::BrownConcretePowder => Self::BrownConcrete,
            Self::GreenConcretePowder => Self::GreenConcrete,
            Self::RedConcretePowder => Self::RedConcrete,
            Self::BlackConcretePowder => Self::BlackConcrete,
            _ => return None,
        })
    }
}

impl StateId {
    pub fn damaged_anvil(self) -> Option<Self> {
        let next = match self.kind()? {
            BlockKind::Anvil => BlockKind::ChippedAnvil,
            BlockKind::ChippedAnvil => BlockKind::DamagedAnvil,
            _ => return None,
        };
        next.state_with(&[("facing", self.property("facing")?)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BlockKind;
    use crate::family::Face;
    use crate::family::testing::{context, property};

    #[test]
    fn anvil_turns_clockwise_from_the_player() {
        let state = Anvil::place(&context("anvil", Face::Up, 0.0, 0.0, 0.0)).map(|p| p.state);
        assert_eq!(state.and_then(|s| property(s, "facing")), Some("west"));
    }

    #[test]
    fn sand_gravel_powder_anvil_and_egg_fall_but_stone_does_not() {
        for name in [
            "sand",
            "red_sand",
            "gravel",
            "white_concrete_powder",
            "anvil",
            "dragon_egg",
            "suspicious_sand",
        ] {
            assert!(
                BlockKind::from_name(name).is_some_and(BlockKind::falls),
                "{name}"
            );
        }
        assert!(!BlockKind::from_name("stone").is_some_and(BlockKind::falls));
        assert!(!BlockKind::from_name("soul_sand").is_some_and(BlockKind::falls));
        assert!(!BlockKind::PointedDripstone.falls());
        assert!(!BlockKind::Scaffolding.falls());
    }

    #[test]
    fn gravity_follows_the_vanilla_class() {
        assert_eq!(BlockKind::Sand.gravity(), Gravity::Falling);
        assert_eq!(
            BlockKind::LimeConcretePowder.gravity(),
            Gravity::ConcretePowder
        );
        assert_eq!(BlockKind::DamagedAnvil.gravity(), Gravity::Anvil);
        assert_eq!(BlockKind::PointedDripstone.gravity(), Gravity::Speleothem);
        assert_eq!(BlockKind::SulfurSpike.gravity(), Gravity::Speleothem);
        assert_eq!(BlockKind::Scaffolding.gravity(), Gravity::Scaffolding);
        assert_eq!(BlockKind::Stone.gravity(), Gravity::None);
    }

    #[test]
    fn every_concrete_powder_hardens_into_its_concrete() {
        let powders: Vec<BlockKind> = BlockKind::ALL
            .into_iter()
            .filter(|kind| kind.gravity() == Gravity::ConcretePowder)
            .collect();
        assert_eq!(powders.len(), 16);
        for powder in powders {
            let name = powder.name().strip_suffix("_powder");
            assert_eq!(powder.concrete(), name.and_then(BlockKind::from_name));
        }
        assert_eq!(BlockKind::Sand.concrete(), None);
    }

    #[test]
    fn anvils_degrade_keeping_facing_until_destroyed() {
        let anvil = BlockKind::Anvil.state_with(&[("facing", "east")]);
        let chipped = anvil.and_then(StateId::damaged_anvil);
        let damaged = chipped.and_then(StateId::damaged_anvil);
        assert_eq!(
            chipped.and_then(StateId::kind),
            Some(BlockKind::ChippedAnvil)
        );
        assert_eq!(
            damaged.and_then(StateId::kind),
            Some(BlockKind::DamagedAnvil)
        );
        assert_eq!(damaged.and_then(|s| property(s, "facing")), Some("east"));
        assert_eq!(damaged.and_then(StateId::damaged_anvil), None);
        assert_eq!(BlockKind::Sand.default_state().damaged_anvil(), None);
    }
}
