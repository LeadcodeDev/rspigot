use super::{BlockBehavior, PlaceContext, Placement, placed, with};

pub struct StandingWithWall;

fn rotation(yaw: f32) -> u8 {
    (((f64::from(yaw) + 180.0) * 16.0 / 360.0 + 0.5).floor() as i64).rem_euclid(16) as u8
}

const ROTATIONS: [&str; 16] = [
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15",
];

impl BlockBehavior for StandingWithWall {
    fn place(context: &PlaceContext) -> Option<Placement> {
        if let Some(wall) = context
            .wall_variant
            .filter(|_| context.face.is_horizontal())
        {
            return placed(with(wall, "facing", context.face.name()));
        }
        placed(with(
            context.default_state,
            "rotation",
            ROTATIONS[usize::from(rotation(context.yaw))],
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BlockKind;
    use crate::family::Face;
    use crate::family::testing::{context, property};

    #[test]
    fn torch_on_wall_becomes_wall_torch_facing_out() {
        let state =
            StandingWithWall::place(&context("torch", Face::North, 0.0, 0.0, 0.5)).map(|p| p.state);
        assert_eq!(
            state.and_then(|s| s.kind()),
            BlockKind::from_name("wall_torch")
        );
        assert_eq!(state.and_then(|s| property(s, "facing")), Some("north"));
    }

    #[test]
    fn standing_sign_rotation_follows_yaw() {
        assert_eq!(rotation(0.0), 8);
        assert_eq!(rotation(180.0), 0);
        let state = StandingWithWall::place(&context("oak_sign", Face::Up, 90.0, 0.0, 0.5))
            .map(|p| p.state);
        assert_eq!(state.and_then(|s| property(s, "rotation")), Some("12"));
    }
}
