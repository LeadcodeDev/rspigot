use super::{
    BlockBehavior, PlaceContext, Placement, UseContext, UseResult, horizontal_facing, opposite,
    placed, toggled, upper_half, with,
};

pub struct Trapdoor<const OPENS_BY_HAND: bool>;

impl<const OPENS_BY_HAND: bool> BlockBehavior for Trapdoor<OPENS_BY_HAND> {
    fn place(context: &PlaceContext) -> Option<Placement> {
        let facing = if context.face.is_horizontal() {
            context.face.name()
        } else {
            opposite(horizontal_facing(context.yaw))
        };
        let half = if upper_half(context) { "top" } else { "bottom" };
        placed(with(
            with(context.default_state, "facing", facing),
            "half",
            half,
        ))
    }

    fn use_block(context: &UseContext) -> Option<UseResult> {
        if !OPENS_BY_HAND {
            return None;
        }
        let (state, _) = toggled(context.state, "open")?;
        Some(UseResult {
            state,
            linked: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::family::Face;
    use crate::family::testing::{context, property};

    #[test]
    fn trapdoor_on_wall_faces_away_from_wall() {
        let state = Trapdoor::<true>::place(&context("oak_trapdoor", Face::East, 0.0, 0.0, 0.8))
            .map(|p| p.state);
        assert_eq!(state.and_then(|s| property(s, "facing")), Some("east"));
        assert_eq!(state.and_then(|s| property(s, "half")), Some("top"));
    }
}
