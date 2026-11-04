use rspigot_domain::effect::EffectId;
use rspigot_domain::item::ItemStack;
use rspigot_domain::player::{GameMode, PlayerId};
use rspigot_domain::{BlockPos, BlockStateId};
use rspigot_entities::EntityKind as Kind;
use rspigot_items::ItemKind;
use rustc_hash::FxHashSet;

use crate::events::{ClientEvent, Direction};
use crate::loot::{self, LootContext, LootEntity};
use crate::player::Player;
use crate::plugins::block::snapshot_of;
use crate::plugins::decisions::Continuation;
use crate::plugins::events::{
    BlockBreak, BlockBreakPatch, BlockDamage, BlockDamageAbort, BlockDamagePatch, BlockDropItem,
    BlockDropItemPatch,
};
use crate::plugins::steps::{
    AbortStep, Announced, BreakStep, DamageStep, DropItemsStep, FinishStep,
};
use crate::plugins::{EntityView, EventPayload, ItemSpec, spec_of};
use crate::ports::ClientLink;
use crate::server::{Audience, Server};

const MINING_EXHAUSTION: f32 = 0.005;
const CORRECT_TOOL_DIVISOR: f32 = 30.0;
const WRONG_TOOL_DIVISOR: f32 = 100.0;
const HASTE_PER_LEVEL: f32 = 0.2;
const SUBMERGED_MINING_SPEED: f32 = 0.2;
const AIRBORNE_DIVISOR: f32 = 5.0;
const NEIGHBOURS: [Direction; 6] = [
    Direction::Down,
    Direction::Up,
    Direction::North,
    Direction::South,
    Direction::West,
    Direction::East,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Mining {
    pub(crate) pos: BlockPos,
    pub(crate) since: u64,
}

pub(crate) fn hand_spec(held: Option<&ItemStack>) -> ItemSpec {
    held.map_or_else(|| ItemSpec::of("minecraft:air", 0), spec_of)
}

fn hardness_of(state: BlockStateId) -> f32 {
    state.kind().map_or(0.0, |kind| kind.settings().hardness)
}

fn fatigue_factor(amplifier: u8) -> f32 {
    match amplifier {
        0 => 0.3,
        1 => 0.09,
        2 => 0.0027,
        _ => 8.1e-4,
    }
}

fn amplifier_of<L>(player: &Player<L>, name: &str) -> Option<u8> {
    let effect = EffectId::from_name(name)?;
    player
        .effects
        .get(effect)
        .map(|instance| instance.amplifier)
}

fn efficiency_of(stack: &ItemStack) -> f32 {
    stack
        .enchantments()
        .into_iter()
        .find(|(key, _)| key == "minecraft:efficiency")
        .map_or(0.0, |(_, level)| {
            let level = f32::from(level);
            level * level + 1.0
        })
}

pub(crate) fn mines_in_creative(held: Option<&ItemStack>) -> bool {
    held.and_then(|stack| stack.item.kind())
        .and_then(ItemKind::tool)
        .is_none_or(|tool| tool.can_destroy_blocks_in_creative)
}

impl<L: ClientLink> Server<L> {
    fn destroy_speed(&self, player: &Player<L>, state: BlockStateId) -> f32 {
        let held = player.inventory.held();
        let tool = held
            .and_then(|stack| stack.item.kind())
            .and_then(ItemKind::tool);
        let mut speed = match (tool, state.kind()) {
            (Some(tool), Some(kind)) => tool.mining_speed(kind.id()),
            _ => 1.0,
        };
        if speed > 1.0
            && let Some(stack) = held
        {
            speed += efficiency_of(stack);
        }
        let haste = [
            amplifier_of(player, "minecraft:haste"),
            amplifier_of(player, "minecraft:conduit_power"),
        ]
        .into_iter()
        .flatten()
        .max();
        if let Some(amplifier) = haste {
            speed *= 1.0 + (f32::from(amplifier) + 1.0) * HASTE_PER_LEVEL;
        }
        if let Some(amplifier) = amplifier_of(player, "minecraft:mining_fatigue") {
            speed *= fatigue_factor(amplifier);
        }
        let eyes = player.eye_position();
        let submerged = self
            .world
            .chunks
            .block(eyes.block())
            .is_some_and(|state| state.flags().contains(rspigot_blocks::StateFlags::WATER));
        if submerged {
            speed *= SUBMERGED_MINING_SPEED;
        }
        if !player.on_ground {
            speed /= AIRBORNE_DIVISOR;
        }
        speed
    }

    pub(crate) fn destroy_progress(&self, id: PlayerId, state: BlockStateId) -> f32 {
        let Some(player) = self.players.get(&id) else {
            return 0.0;
        };
        let hardness = hardness_of(state);
        if hardness < 0.0 {
            return 0.0;
        }
        let divisor = if loot::tool_is_correct(player.inventory.held(), state) {
            CORRECT_TOOL_DIVISOR
        } else {
            WRONG_TOOL_DIVISOR
        };
        self.destroy_speed(player, state) / hardness / divisor
    }

    pub(crate) fn ack_block(&mut self, id: PlayerId, sequence: i32) {
        self.push(Audience::One(id), ClientEvent::BlockChangedAck(sequence));
    }

    pub(crate) fn start_destroying(&mut self, id: PlayerId, pos: BlockPos, sequence: i32) {
        let Some(player) = self.players.get(&id) else {
            return;
        };
        let game_mode = player.game_mode;
        let held = player.inventory.held().cloned();
        let Some(state) = self.world.chunks.block(pos) else {
            self.ack_block(id, sequence);
            return;
        };
        match game_mode {
            GameMode::Creative => {
                let refused = !mines_in_creative(held.as_ref());
                self.ask_break(BreakStep {
                    player: id,
                    pos,
                    sequence,
                    state,
                    game_mode,
                    held,
                    refused,
                    exp: 0,
                });
            }
            GameMode::Survival if !state.is_air() => {
                let insta = self.destroy_progress(id, state) >= 1.0;
                let item_in_hand = hand_spec(held.as_ref());
                let step = DamageStep {
                    player: id,
                    pos,
                    sequence,
                    state,
                    held,
                    insta,
                };
                self.fire(id, Continuation::Damage(Box::new(step)), move |server| {
                    Some(EventPayload::BlockDamage(BlockDamage {
                        block: pos,
                        insta_break: insta,
                        item_in_hand,
                        player: server.snapshot_of(id)?,
                    }))
                });
            }
            _ => self.ack_block(id, sequence),
        }
    }

    pub(crate) fn finish_damage(
        &mut self,
        step: DamageStep,
        cancelled: bool,
        patch: BlockDamagePatch,
    ) {
        let (id, pos) = (step.player, step.pos);
        let Some(player) = self.players.get(&id) else {
            return;
        };
        let valid = player.game_mode == GameMode::Survival
            && player.inventory.held() == step.held.as_ref()
            && self.world.chunks.block(pos) == Some(step.state);
        if cancelled || !valid {
            self.refuse_block(id, pos, step.sequence);
            return;
        }
        if step.insta || patch.insta_break == Some(true) {
            self.ask_break(BreakStep {
                player: id,
                pos,
                sequence: step.sequence,
                state: step.state,
                game_mode: GameMode::Survival,
                held: step.held,
                refused: false,
                exp: 0,
            });
            return;
        }
        let since = self.current_tick();
        if let Some(player) = self.players.get_mut(&id) {
            player.mining = Some(Mining { pos, since });
        }
        self.ack_block(id, step.sequence);
    }

    pub(crate) fn finish_digging(&mut self, step: FinishStep) {
        let (id, pos, sequence) = (step.player, step.pos, step.sequence);
        let Some(player) = self.players.get_mut(&id) else {
            return;
        };
        let started = player.mining.take_if(|mining| mining.pos == pos).is_some();
        let game_mode = player.game_mode;
        let held = player.inventory.held().cloned();
        let state = self.world.chunks.block(pos);
        let Some(state) = state.filter(|_| started && game_mode == GameMode::Survival) else {
            self.refuse_block(id, pos, sequence);
            return;
        };
        if state.is_air() {
            self.ack_block(id, sequence);
            return;
        }
        if hardness_of(state) < 0.0 || !self.within_reach(id, pos) || self.is_protected(id, pos) {
            self.refuse_block(id, pos, sequence);
            return;
        }
        self.ask_break(BreakStep {
            player: id,
            pos,
            sequence,
            state,
            game_mode,
            held,
            refused: false,
            exp: 0,
        });
    }

    fn ask_break(&mut self, step: BreakStep) {
        let (id, pos, exp) = (step.player, step.pos, step.exp);
        let describe = move |server: &Self| {
            Some(EventPayload::BlockBreak(BlockBreak {
                block: pos,
                drop_items: true,
                exp_to_drop: exp,
                player: server.snapshot_of(id)?,
            }))
        };
        let refused = step.refused;
        let then = Continuation::Break(Box::new(step));
        if refused {
            self.fire_refused(id, then, describe);
            return;
        }
        if self.asks(&then) && !self.has_block_entity(pos) {
            self.push(
                Audience::One(id),
                ClientEvent::BlockUpdate {
                    pos,
                    state: BlockStateId::AIR,
                },
            );
        }
        self.fire(id, then, describe);
    }

    pub(crate) fn finish_break(
        &mut self,
        step: BreakStep,
        cancelled: bool,
        patch: BlockBreakPatch,
    ) {
        let (id, pos, sequence) = (step.player, step.pos, step.sequence);
        if step.refused {
            self.ack_block(id, sequence);
            return;
        }
        let Some(player) = self.players.get(&id) else {
            return;
        };
        let valid = player.game_mode == step.game_mode
            && player.inventory.held() == step.held.as_ref()
            && self.world.chunks.block(pos) == Some(step.state);
        if cancelled || !valid {
            self.refuse_break(id, pos, sequence);
            return;
        }
        let drop_items = patch.drop_items.unwrap_or(true);
        let exp = patch.exp_to_drop.unwrap_or(step.exp);
        self.set_block(pos, BlockStateId::AIR);
        self.ack_block(id, sequence);
        let survival = step.game_mode == GameMode::Survival;
        if survival {
            self.wear_tool_for_mining(id, hardness_of(step.state));
        }
        if !drop_items {
            self.pay_block_exp(pos, exp);
            return;
        }
        let mut stacks = Vec::new();
        if survival && loot::tool_is_correct(step.held.as_ref(), step.state) {
            self.exhaust(id, MINING_EXHAUSTION);
            if self.config.rules.block_drops {
                let breaker = LootEntity::of(Kind::Player);
                let context = LootContext::block(step.state, step.held.as_ref(), Some(&breaker));
                stacks = loot::block_drops(&context, &mut self.rng);
            }
        }
        self.ask_drop_items(id, pos, step.state, stacks, exp);
    }

    fn refuse_break(&mut self, id: PlayerId, pos: BlockPos, sequence: i32) {
        let around = NEIGHBOURS.map(|face| {
            let (dx, dy, dz) = face.offset();
            pos.offset(dx, dy, dz)
        });
        for at in std::iter::once(pos).chain(around) {
            if let Some(state) = self.world.chunks.block(at) {
                self.push(
                    Audience::One(id),
                    ClientEvent::BlockUpdate { pos: at, state },
                );
            }
        }
        self.ack_block(id, sequence);
        if let Some(update) = self.block_entity_update(pos) {
            self.push(Audience::One(id), update);
        }
        self.ack_block(id, sequence);
    }

    fn ask_drop_items(
        &mut self,
        id: PlayerId,
        pos: BlockPos,
        state: BlockStateId,
        stacks: Vec<ItemStack>,
        exp: i32,
    ) {
        let drops: Vec<(Announced, ItemStack)> = stacks
            .into_iter()
            .map(|stack| (self.block_drop_launch(pos), stack))
            .collect();
        let items: Vec<EntityView> = drops
            .iter()
            .map(|(launch, _)| EntityView {
                id: launch.base.id,
                uuid: launch.base.uuid,
                kind: Kind::Item.name().to_owned(),
                position: launch.base.position,
            })
            .collect();
        let block_state = snapshot_of(pos, state);
        let step = DropItemsStep { pos, drops, exp };
        self.fire(id, Continuation::DropItems(Box::new(step)), move |server| {
            Some(EventPayload::BlockDropItem(BlockDropItem {
                block: pos,
                block_state,
                items,
                player: server.snapshot_of(id)?,
            }))
        });
    }

    pub(crate) fn finish_drop_items(
        &mut self,
        step: DropItemsStep,
        cancelled: bool,
        patch: BlockDropItemPatch,
    ) {
        if !cancelled {
            let kept: Option<FxHashSet<_>> = patch
                .items
                .map(|items| items.into_iter().map(|item| item.id).collect());
            for (launch, stack) in step.drops {
                let wanted = kept
                    .as_ref()
                    .is_none_or(|kept| kept.contains(&launch.base.id));
                if wanted && self.world.chunks.is_loaded(launch.base.position.chunk()) {
                    self.drop_launched(launch, stack);
                }
            }
        }
        self.pay_block_exp(step.pos, step.exp);
    }

    fn pay_block_exp(&mut self, pos: BlockPos, exp: i32) {
        if exp > 0 && self.config.rules.block_drops {
            self.award_experience(pos.center(), exp);
        }
    }

    pub(crate) fn finish_aborting(&mut self, step: AbortStep) {
        let (id, pos) = (step.player, step.pos);
        let Some(player) = self.players.get_mut(&id) else {
            return;
        };
        let destroying = player.mining.take().is_some();
        let held = player.inventory.held().cloned();
        if !destroying && !self.within_reach(id, pos) {
            self.ack_block(id, step.sequence);
            return;
        }
        let item_in_hand = hand_spec(held.as_ref());
        self.fire(id, Continuation::Abort(step), move |server| {
            Some(EventPayload::BlockDamageAbort(BlockDamageAbort {
                block: pos,
                item_in_hand,
                player: server.snapshot_of(id)?,
            }))
        });
    }

    pub(crate) fn finish_abort(&mut self, step: &AbortStep) {
        self.ack_block(step.player, step.sequence);
    }
}
