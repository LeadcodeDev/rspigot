use super::{
    BlockBehavior, Linked, LinkedProperty, PlaceContext, Placement, UseContext, UseResult,
    horizontal_facing, toggled, with,
};

pub struct Door<const OPENS_BY_HAND: bool>;

impl<const OPENS_BY_HAND: bool> BlockBehavior for Door<OPENS_BY_HAND> {
    fn place(context: &PlaceContext) -> Option<Placement> {
        let lower = with(
            with(
                context.default_state,
                "facing",
                horizontal_facing(context.yaw),
            ),
            "half",
            "lower",
        );
        let upper = with(lower, "half", "upper");
        Some(Placement {
            state: lower,
            linked: Some(Linked {
                dy: 1,
                state: upper,
            }),
        })
    }

    fn use_block(context: &UseContext) -> Option<UseResult> {
        if !OPENS_BY_HAND {
            return None;
        }
        let (state, value) = toggled(context.state, "open")?;
        let dy = if context.state.property("half")? == "lower" {
            1
        } else {
            -1
        };
        Some(UseResult {
            state,
            linked: Some(LinkedProperty {
                dy,
                property: "open",
                value,
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::family::Face;
    use crate::family::testing::{context, property};

    #[test]
    fn door_places_both_halves_facing_the_player() {
        let placement = Door::<true>::place(&context("oak_door", Face::Up, 90.0, 0.0, 0.0));
        let lower = placement.map(|p| p.state);
        let upper = placement.and_then(|p| p.linked);
        assert_eq!(lower.and_then(|s| property(s, "half")), Some("lower"));
        assert_eq!(lower.and_then(|s| property(s, "facing")), Some("west"));
        assert_eq!(upper.map(|l| l.dy), Some(1));
        assert_eq!(upper.and_then(|l| property(l.state, "half")), Some("upper"));
    }

    #[test]
    fn wooden_door_toggles_and_iron_door_does_not() {
        let lower = Door::<true>::place(&context("oak_door", Face::Up, 0.0, 0.0, 0.0))
            .map(|p| p.state)
            .unwrap_or_default();
        let used = Door::<true>::use_block(&UseContext {
            state: lower,
            sneaking: false,
        });
        assert_eq!(used.and_then(|u| property(u.state, "open")), Some("true"));
        assert_eq!(
            used.and_then(|u| u.linked).map(|l| (l.dy, l.value)),
            Some((1, "true"))
        );
        assert_eq!(
            Door::<false>::use_block(&UseContext {
                state: lower,
                sneaking: false
            }),
            None
        );
    }
}
