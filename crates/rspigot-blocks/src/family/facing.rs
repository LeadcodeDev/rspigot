use super::{
    BlockBehavior, PlaceContext, Placement, horizontal_facing, looking, opposite, placed, with,
};

pub struct HorizontalAway;

impl BlockBehavior for HorizontalAway {
    fn place(context: &PlaceContext) -> Option<Placement> {
        placed(with(
            context.default_state,
            "facing",
            opposite(horizontal_facing(context.yaw)),
        ))
    }
}

pub struct HorizontalToward;

impl BlockBehavior for HorizontalToward {
    fn place(context: &PlaceContext) -> Option<Placement> {
        placed(with(
            context.default_state,
            "facing",
            horizontal_facing(context.yaw),
        ))
    }
}

pub struct Directional<const AWAY: bool>;

impl<const AWAY: bool> BlockBehavior for Directional<AWAY> {
    fn place(context: &PlaceContext) -> Option<Placement> {
        let toward = looking(context.yaw, context.pitch);
        let facing = if AWAY { opposite(toward) } else { toward };
        placed(with(context.default_state, "facing", facing))
    }
}

pub struct FaceAttached;

impl BlockBehavior for FaceAttached {
    fn place(context: &PlaceContext) -> Option<Placement> {
        placed(with(context.default_state, "facing", context.face.name()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::family::Face;
    use crate::family::testing::{context, property};

    #[test]
    fn furnace_faces_the_player() {
        let state =
            HorizontalAway::place(&context("furnace", Face::Up, 180.0, 0.0, 0.0)).map(|p| p.state);
        assert_eq!(state.and_then(|s| property(s, "facing")), Some("south"));
    }

    #[test]
    fn dispenser_looking_down_faces_up_and_observer_faces_down() {
        let dispenser = Directional::<true>::place(&context("dispenser", Face::Up, 0.0, 80.0, 0.0))
            .map(|p| p.state);
        assert_eq!(dispenser.and_then(|s| property(s, "facing")), Some("up"));
        let observer = Directional::<false>::place(&context("observer", Face::Up, 0.0, 80.0, 0.0))
            .map(|p| p.state);
        assert_eq!(observer.and_then(|s| property(s, "facing")), Some("down"));
    }

    #[test]
    fn end_rod_points_out_of_the_clicked_face() {
        let state =
            FaceAttached::place(&context("end_rod", Face::West, 0.0, 0.0, 0.0)).map(|p| p.state);
        assert_eq!(state.and_then(|s| property(s, "facing")), Some("west"));
    }
}
