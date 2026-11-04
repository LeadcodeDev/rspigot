use rspigot_domain::furnace::{Answers, Ask, BurnAnswer, Furnace, Progress, SmeltAnswer};
use rspigot_domain::player::PlayerId;
use rspigot_domain::{BlockPos, ItemId};
use rspigot_items::CookingKind;

use crate::plugins::decisions::{Continuation, Lane};
use crate::plugins::events::{
    FurnaceBurn, FurnaceBurnPatch, FurnaceExtract, FurnaceExtractPatch, FurnaceSmelt,
    FurnaceSmeltPatch, FurnaceStartSmelt, FurnaceStartSmeltPatch,
};
use crate::plugins::steps::ExpStep;
use crate::plugins::{EventKind, EventPayload, spec_of, stack_of};
use crate::ports::ClientLink;
use crate::server::Server;

#[derive(Debug)]
pub(crate) struct FurnaceStep {
    pub(crate) pos: BlockPos,
    pub(crate) kind: CookingKind,
    pub(crate) snapshot: Furnace,
    pub(crate) answers: Answers,
    pub(crate) ask: Ask,
}

#[derive(Debug)]
pub(crate) struct ExtractStep {
    pub(crate) player: PlayerId,
    pub(crate) exp: i32,
}

pub(crate) enum FurnaceAnswer {
    Burn(FurnaceBurnPatch),
    Start(FurnaceStartSmeltPatch),
    Smelt(FurnaceSmeltPatch),
}

const fn event_of(ask: &Ask) -> EventKind {
    match ask {
        Ask::Burn { .. } => EventKind::FurnaceBurn,
        Ask::StartSmelt { .. } => EventKind::FurnaceStartSmelt,
        Ask::Smelt { .. } => EventKind::FurnaceSmelt,
    }
}

fn payload_of(pos: BlockPos, ask: &Ask) -> EventPayload {
    match ask {
        Ask::Burn { fuel, burn_time } => EventPayload::FurnaceBurn(FurnaceBurn {
            block: pos,
            burn_time: *burn_time,
            burning: true,
            fuel: spec_of(fuel),
        }),
        Ask::StartSmelt {
            source,
            recipe,
            total_cook_time,
        } => EventPayload::FurnaceStartSmelt(FurnaceStartSmelt {
            block: pos,
            recipe: recipe.id.to_owned(),
            source: spec_of(source),
            total_cook_time: *total_cook_time,
        }),
        Ask::Smelt { source, result } => EventPayload::FurnaceSmelt(FurnaceSmelt {
            block: pos,
            result: spec_of(result),
            source: spec_of(source),
        }),
    }
}

impl<L: ClientLink> Server<L> {
    pub(crate) fn furnace_asking(&self, pos: BlockPos) -> bool {
        self.decisions.in_flight(Lane::Block(pos), |then| {
            matches!(then, Continuation::Furnace(..))
        })
    }

    pub(crate) fn run_furnace(
        &mut self,
        pos: BlockPos,
        kind: CookingKind,
        furnace: &Furnace,
        mut answers: Answers,
    ) {
        loop {
            let mut ticked = furnace.clone();
            match ticked.step(kind, &answers) {
                Progress::Done(tick) => {
                    if ticked != *furnace {
                        self.furnace_ticked(pos, &ticked, tick);
                    }
                    return;
                }
                Progress::Ask(ask) => {
                    let event = event_of(&ask);
                    if !self.listened(event) {
                        answers.settle_default(&ask);
                        continue;
                    }
                    let payload = payload_of(pos, &ask);
                    let step = FurnaceStep {
                        pos,
                        kind,
                        snapshot: furnace.clone(),
                        answers,
                        ask,
                    };
                    self.fire(
                        Lane::Block(pos),
                        Continuation::Furnace(event, Box::new(step)),
                        move |_| Some(payload),
                    );
                    return;
                }
            }
        }
    }

    pub(crate) fn finish_furnace(
        &mut self,
        step: FurnaceStep,
        cancelled: bool,
        answer: FurnaceAnswer,
    ) {
        let FurnaceStep {
            pos,
            kind,
            snapshot,
            mut answers,
            ask,
        } = step;
        let Some(current) = self.cooking_furnace(pos, kind) else {
            return;
        };
        if current != snapshot {
            self.run_furnace(pos, kind, &current, Answers::default());
            return;
        }
        match (ask, answer) {
            (Ask::Burn { burn_time, .. }, FurnaceAnswer::Burn(patch)) => {
                answers.burn = Some(if cancelled {
                    BurnAnswer::Cancelled
                } else {
                    BurnAnswer::Burn {
                        burn_time: patch.burn_time.unwrap_or(burn_time),
                        burning: patch.burning.unwrap_or(true),
                    }
                });
            }
            (
                Ask::StartSmelt {
                    total_cook_time, ..
                },
                FurnaceAnswer::Start(patch),
            ) => {
                answers.start = Some(patch.total_cook_time.unwrap_or(total_cook_time));
            }
            (Ask::Smelt { result, .. }, FurnaceAnswer::Smelt(patch)) => {
                answers.smelt = Some(if cancelled {
                    SmeltAnswer::Cancelled
                } else {
                    let made = patch
                        .result
                        .and_then(|spec| stack_of(&spec).ok())
                        .unwrap_or(Some(result));
                    SmeltAnswer::Result(made)
                });
            }
            _ => return,
        }
        self.run_furnace(pos, kind, &current, answers);
    }

    pub(crate) fn pay_furnace_experience(
        &mut self,
        id: PlayerId,
        pos: BlockPos,
        item: ItemId,
        amount: u32,
    ) {
        let Some(mut furnace) = self.furnace_at(pos) else {
            return;
        };
        let Some(at) = self.players.get(&id).map(|player| player.position) else {
            return;
        };
        let rng = &mut self.rng;
        let owed = furnace.owed(&mut || rand::Rng::random::<f32>(rng));
        if owed.is_empty() {
            return;
        }
        furnace.recipes_used.clear();
        self.store_furnace(pos, &furnace);
        let item_type = item.kind().map_or("minecraft:air", |kind| kind.name());
        let item_amount = i32::try_from(amount).unwrap_or(i32::MAX);
        for exp in owed {
            if amount == 0 {
                self.ask_block_exp(ExpStep {
                    pos,
                    amount: exp,
                    at,
                });
                continue;
            }
            self.fire(
                id,
                Continuation::Extract(ExtractStep { player: id, exp }),
                move |server| {
                    Some(EventPayload::FurnaceExtract(FurnaceExtract {
                        block: pos,
                        exp_to_drop: exp,
                        item_amount,
                        item_type: item_type.to_owned(),
                        player: server.snapshot_of(id)?,
                    }))
                },
            );
        }
    }

    pub(crate) fn finish_extract(&mut self, step: ExtractStep, patch: FurnaceExtractPatch) {
        let Some(at) = self.players.get(&step.player).map(|player| player.position) else {
            return;
        };
        self.award_experience(at, patch.exp_to_drop.unwrap_or(step.exp));
    }
}
