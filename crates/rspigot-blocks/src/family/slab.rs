use super::{BlockBehavior, PlaceContext, Placement, placed, upper_half, with};

pub struct Slab;

impl BlockBehavior for Slab {
    fn place(context: &PlaceContext) -> Option<Placement> {
        let half = if upper_half(context) { "top" } else { "bottom" };
        placed(with(context.default_state, "type", half))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::family::Face;
    use crate::family::testing::{context, property};

    #[test]
    fn upper_half_click_places_top_slab() {
        let top = Slab::place(&context("oak_slab", Face::North, 0.0, 0.0, 0.75)).map(|p| p.state);
        assert_eq!(top.and_then(|s| property(s, "type")), Some("top"));
        let bottom = Slab::place(&context("oak_slab", Face::Up, 0.0, 0.0, 0.0)).map(|p| p.state);
        assert_eq!(bottom.and_then(|s| property(s, "type")), Some("bottom"));
    }
}
