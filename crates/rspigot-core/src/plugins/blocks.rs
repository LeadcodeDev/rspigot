use rspigot_domain::{BlockPos, BlockStateId};

use crate::explosion::BlockInteraction;
use crate::plugins::block::material_of;
use crate::plugins::block::snapshot_of;
use crate::plugins::decisions::{Continuation, Lane};
use crate::plugins::events::{
    BlockCook, BlockExp, BlockExpPatch, BlockExplode, BlockExplodePatch, BlockPhysics,
    ExplosionResult,
};
use crate::plugins::steps::{CookStep, ExpStep, ExplodeStep, PhysicsStep};
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

    pub(crate) fn ask_block_exp(&mut self, step: ExpStep) {
        let ExpStep { pos, amount } = step;
        self.fire(Lane::Block(pos), Continuation::Exp(step), move |_| {
            Some(EventPayload::BlockExp(BlockExp {
                block: pos,
                exp_to_drop: amount,
            }))
        });
    }

    pub(crate) fn finish_block_exp(&mut self, step: ExpStep, patch: BlockExpPatch) {
        let amount = patch.exp_to_drop.unwrap_or(step.amount);
        if self.world.chunks.is_loaded(step.pos.chunk()) {
            self.award_experience(step.pos.center(), amount);
        }
    }

    pub(crate) fn ask_explosion(&mut self, step: ExplodeStep) {
        let block = step.center.block();
        let exploded_block_state = snapshot_of(
            block,
            self.world.chunks.block(block).unwrap_or(BlockStateId::AIR),
        );
        let block_list = step.blocks.clone();
        let explosion_result = match step.interaction {
            BlockInteraction::Keep => ExplosionResult::Keep,
            BlockInteraction::Destroy => ExplosionResult::Destroy,
            BlockInteraction::DestroyWithDecay => ExplosionResult::DestroyWithDecay,
        };
        let explosion_yield = step.explosion_yield;
        self.fire(
            Lane::Block(block),
            Continuation::Explode(Box::new(step)),
            move |_| {
                Some(EventPayload::BlockExplode(BlockExplode {
                    block,
                    block_list,
                    exploded_block_state,
                    explosion_result,
                    explosion_yield,
                }))
            },
        );
    }

    pub(crate) fn finish_explosion(&mut self, step: ExplodeStep, patch: BlockExplodePatch) {
        let chunks = &self.world.chunks;
        let blocks = patch
            .block_list
            .unwrap_or(step.blocks)
            .into_iter()
            .filter(|pos| chunks.block(*pos).is_some_and(|state| !state.is_air()))
            .collect();
        let explosion_yield = patch.explosion_yield.unwrap_or(step.explosion_yield);
        self.blast(ExplodeStep {
            blocks,
            explosion_yield,
            ..step
        });
    }
}
