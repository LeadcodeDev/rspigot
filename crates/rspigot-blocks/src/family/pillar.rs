use super::{BlockBehavior, Face, PlaceContext, Placement, placed, with};

pub struct Pillar;

impl BlockBehavior for Pillar {
    fn place(context: &PlaceContext) -> Option<Placement> {
        let axis = match context.face {
            Face::Up | Face::Down => "y",
            Face::North | Face::South => "z",
            Face::East | Face::West => "x",
        };
        placed(with(context.default_state, "axis", axis))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::family::testing::{context, property};

    #[test]
    fn axis_follows_clicked_face() {
        let placed = Pillar::place(&context("oak_log", Face::East, 0.0, 0.0, 0.5)).map(|p| p.state);
        assert_eq!(placed.and_then(|s| property(s, "axis")), Some("x"));
    }
}
