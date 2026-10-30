use super::{
    BlockBehavior, PlaceContext, Placement, UseContext, UseResult, horizontal_facing, placed,
    toggled, with,
};

pub struct FenceGate;

impl BlockBehavior for FenceGate {
    fn place(context: &PlaceContext) -> Option<Placement> {
        placed(with(
            context.default_state,
            "facing",
            horizontal_facing(context.yaw),
        ))
    }

    fn use_block(context: &UseContext) -> Option<UseResult> {
        let (state, _) = toggled(context.state, "open")?;
        Some(UseResult {
            state,
            linked: None,
        })
    }
}
