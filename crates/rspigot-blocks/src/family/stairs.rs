use super::{BlockBehavior, PlaceContext, Placement, horizontal_facing, placed, upper_half, with};

pub struct Stairs;

impl BlockBehavior for Stairs {
    fn place(context: &PlaceContext) -> Option<Placement> {
        let state = with(
            context.default_state,
            "facing",
            horizontal_facing(context.yaw),
        );
        let half = if upper_half(context) { "top" } else { "bottom" };
        placed(with(state, "half", half))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::family::Face;
    use crate::family::testing::{context, property};

    #[test]
    fn stairs_face_the_player_direction_and_flip_on_ceiling() {
        let state =
            Stairs::place(&context("oak_stairs", Face::Down, 180.0, 0.0, 0.5)).map(|p| p.state);
        assert_eq!(state.and_then(|s| property(s, "facing")), Some("north"));
        assert_eq!(state.and_then(|s| property(s, "half")), Some("top"));
    }
}
