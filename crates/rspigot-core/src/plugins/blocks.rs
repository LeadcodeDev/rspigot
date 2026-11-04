use rspigot_domain::{BlockPos, BlockStateId};

use crate::plugins::block::material_of;
use crate::plugins::decisions::{Continuation, Lane};
use crate::plugins::events::{BlockCook, BlockPhysics};
use crate::plugins::steps::{CookStep, PhysicsStep};
use crate::plugins::{EventPayload, spec_of};
use crate::ports::ClientLink;
use crate::server::Server;

impl<L: ClientLink> Server<L> {
    pub(crate) fn ask_physics(&mut self, pos: BlockPos, state: BlockStateId) {
        self.fire(
            Lane::Block(pos),
            Continuation::Physics(PhysicsStep { pos, state }),
            move |_| {
                Some(EventPayload::BlockPhysics(BlockPhysics {
                    block: pos,
                    changed_type: material_of(state).to_owned(),
                    source_block: pos,
                }))
            },
        );
    }

    pub(crate) fn finish_physics(&mut self, step: PhysicsStep) {
        if self.world.chunks.block(step.pos) == Some(step.state) {
            self.update_neighbour_shapes(step.pos);
        }
    }

    pub(crate) fn ask_cook(&mut self, step: CookStep) {
        let pos = step.pos;
        let source = spec_of(&step.source);
        let result = spec_of(&step.result);
        self.fire(
            Lane::Block(pos),
            Continuation::Cook(Box::new(step)),
            move |_| {
                Some(EventPayload::BlockCook(BlockCook {
                    block: pos,
                    result,
                    source,
                }))
            },
        );
    }
}
