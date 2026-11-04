use std::collections::{BTreeMap, VecDeque};

use rspigot_domain::Vec3;
use rspigot_domain::player::PlayerId;
use rustc_hash::FxHashMap;

pub(crate) use crate::plugins::steps::{Continuation, Held, MoveStep, Undo};
use crate::plugins::{EventKind, EventPatch, EventPayload, Verdict};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) enum Lane {
    Player(PlayerId),
    Login(u64),
}

impl From<PlayerId> for Lane {
    fn from(player: PlayerId) -> Self {
        Self::Player(player)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Asked {
    pub(crate) connection: u64,
    pub(crate) handler: u32,
    pub(crate) kind: EventKind,
}

#[derive(Debug, Clone, PartialEq)]
struct Answer {
    cancel: bool,
    patch: Option<EventPatch>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Settlement {
    Cancelled(Option<EventPatch>),
    Applied(Option<EventPatch>),
}

pub(crate) struct Pending {
    lane: Lane,
    payload: EventPayload,
    asked: Vec<Asked>,
    answers: Vec<Option<Answer>>,
    waiting: Vec<u64>,
    deadline: u64,
    until: u64,
    then: Continuation,
    held: Vec<Held>,
}

impl Pending {
    pub(crate) fn new(
        lane: impl Into<Lane>,
        payload: EventPayload,
        asked: Vec<Asked>,
        deadline: u64,
        until: u64,
        then: Continuation,
    ) -> Self {
        let mut waiting: Vec<u64> = Vec::new();
        for ask in &asked {
            if !waiting.contains(&ask.connection) {
                waiting.push(ask.connection);
            }
        }
        Self {
            lane: lane.into(),
            payload,
            answers: vec![None; asked.len()],
            asked,
            waiting,
            deadline,
            until,
            then,
            held: Vec::new(),
        }
    }

    fn cancelled(&self) -> bool {
        self.answers.iter().flatten().any(|answer| answer.cancel)
    }

    fn ready(&self, tick: u64) -> bool {
        let final_cancel = self.cancelled() && !self.payload.kind().writes_when_cancelled();
        final_cancel || self.waiting.is_empty() || tick >= self.deadline
    }

    fn take(
        &mut self,
        connection: u64,
        event: u64,
        verdicts: Vec<Verdict>,
        warnings: &mut Vec<String>,
    ) -> Taken {
        let Some(index) = self
            .waiting
            .iter()
            .position(|waiting| *waiting == connection)
        else {
            if self.asked.iter().any(|ask| ask.connection == connection) {
                warnings.push(answered_twice(event));
            }
            return Taken::Ignored;
        };
        self.waiting.swap_remove(index);
        let kind = self.payload.kind();
        let admitted = admit(&self.asked, connection, event, verdicts, warnings);
        let cancelled = kind.cancellable() && admitted.iter().any(|(_, verdict)| verdict.cancel);
        for (slot, verdict) in admitted {
            let taken_as = self.asked[slot].kind;
            let mut patch = match verdict.patch {
                Some(patch) if patch.kind() == taken_as => patch.lift(kind),
                Some(misfit) => {
                    warnings.push(format!(
                        "a {:?} patch does not fit event {event}, which is a {taken_as:?}",
                        misfit.kind()
                    ));
                    None
                }
                None => None,
            };
            if let Some(reason) = patch.as_ref().and_then(EventPatch::refusal) {
                warnings.push(format!("patch of event {event} was refused: {reason}"));
                patch = None;
            }
            let mut cancel = verdict.cancel;
            if cancel && !kind.cancellable() {
                warnings.push(format!(
                    "event {event} cannot be cancelled, the cancel of handler {} is ignored",
                    verdict.handler
                ));
                cancel = false;
            }
            if cancel && let Some(denial) = EventPatch::denial(kind) {
                patch = Some(match patch {
                    Some(own) => own.then(denial),
                    None => denial,
                });
                cancel = false;
            }
            self.answers[slot] = Some(Answer { cancel, patch });
        }
        if !cancelled {
            for (ask, answer) in self.asked.iter().zip(&self.answers) {
                if ask.connection == connection && answer.is_none() {
                    warnings.push(format!(
                        "handler {} of event {event} gave no verdict",
                        ask.handler
                    ));
                }
            }
        }
        Taken::Counted
    }

    fn record(&self) -> Record {
        let (silent, answered) = self
            .asked
            .iter()
            .partition(|ask| self.waiting.contains(&ask.connection));
        Record {
            lane: self.lane,
            undo: self.then.undo(),
            silent,
            answered,
            until: self.until,
            deadline: self.deadline,
            applied: None,
        }
    }

    fn settle(self, event: u64) -> Ready {
        let record = self.record();
        let (cancelled, patch) = fold(self.answers);
        let settlement = if cancelled || denied(&self.payload, patch.as_ref()) {
            Settlement::Cancelled(patch.and_then(EventPatch::when_cancelled))
        } else {
            Settlement::Applied(patch)
        };
        Ready {
            event,
            lane: self.lane,
            payload: self.payload,
            settlement,
            then: self.then,
            held: self.held,
            record,
        }
    }
}

fn answered_twice(event: u64) -> String {
    format!("event {event} was already answered by this plugin")
}

fn admit(
    asked: &[Asked],
    connection: u64,
    event: u64,
    verdicts: Vec<Verdict>,
    warnings: &mut Vec<String>,
) -> Vec<(usize, Verdict)> {
    let mut admitted: Vec<(usize, Verdict)> = Vec::with_capacity(verdicts.len());
    for verdict in verdicts {
        let handler = verdict.handler;
        match asked
            .iter()
            .position(|ask| ask.connection == connection && ask.handler == handler)
        {
            None => warnings.push(format!(
                "handler {handler} was not asked to decide event {event}"
            )),
            Some(slot) if admitted.iter().any(|(seen, _)| *seen == slot) => warnings.push(format!(
                "handler {handler} answered event {event} more than once"
            )),
            Some(slot) => admitted.push((slot, verdict)),
        }
    }
    admitted
}

fn fold(answers: Vec<Option<Answer>>) -> (bool, Option<EventPatch>) {
    let mut cancelled = false;
    let mut patch: Option<EventPatch> = None;
    for answer in answers.into_iter().flatten() {
        cancelled |= answer.cancel;
        patch = match (patch, answer.patch) {
            (Some(earlier), Some(later)) => Some(earlier.then(later)),
            (earlier, later) => later.or(earlier),
        };
    }
    (cancelled, patch)
}

fn denied(payload: &EventPayload, patch: Option<&EventPatch>) -> bool {
    match patch {
        Some(patch) if payload.kind().cancels_by_result() => {
            let mut settled = payload.clone();
            patch.clone().settle(&mut settled);
            settled.cancelled()
        }
        _ => payload.cancelled(),
    }
}

pub(crate) struct Ready {
    pub(crate) event: u64,
    pub(crate) lane: Lane,
    pub(crate) payload: EventPayload,
    pub(crate) settlement: Settlement,
    pub(crate) then: Continuation,
    pub(crate) held: Vec<Held>,
    pub(crate) record: Record,
}

pub(crate) struct Due {
    pub(crate) event: u64,
    pub(crate) lane: Lane,
    pub(crate) then: Continuation,
}

pub(crate) enum Flushed {
    Ready(Box<Ready>),
    Due(Due),
}

pub(crate) struct Record {
    lane: Lane,
    undo: Option<Undo>,
    silent: Vec<Asked>,
    answered: Vec<Asked>,
    until: u64,
    deadline: u64,
    pub(crate) applied: Option<EventPayload>,
}

impl Record {
    fn silent_from(&self, connection: u64) -> bool {
        self.silent.iter().any(|ask| ask.connection == connection)
    }

    fn answered_by(&self, connection: u64) -> bool {
        self.answered.iter().any(|ask| ask.connection == connection)
    }

    fn kept(&self, tick: u64) -> bool {
        tick <= self.until.max(self.deadline)
    }
}

pub(crate) type Undone = Vec<(u64, EventPayload)>;

pub(crate) struct Rollback {
    pub(crate) lane: Lane,
    pub(crate) undo: Undo,
    pub(crate) undone: Undone,
    pub(crate) dropped: Undone,
}

pub(crate) enum Taken {
    Counted,
    Ignored,
    RolledBack(Rollback),
    TooLate(&'static str),
}

enum Queued {
    Asked(u64, Box<Pending>),
    Due(u64, Continuation),
}

impl Queued {
    const fn then(&self) -> &Continuation {
        match self {
            Self::Asked(_, pending) => &pending.then,
            Self::Due(_, then) => then,
        }
    }

    fn moving(&self) -> bool {
        matches!(self, Self::Asked(_, pending) if pending.then.is_move())
    }
}

#[derive(Default)]
pub(crate) struct Decisions {
    lanes: FxHashMap<Lane, VecDeque<Queued>>,
    settled: BTreeMap<u64, Record>,
    heads: Vec<(Lane, usize)>,
}

impl Decisions {
    pub(crate) fn reference(&self, lane: impl Into<Lane>) -> Option<Vec3> {
        self.lanes
            .get(&lane.into())?
            .iter()
            .rev()
            .find_map(|queued| queued.then().destination())
    }

    pub(crate) fn busy(&self, lane: impl Into<Lane>) -> bool {
        let lane = lane.into();
        match self.heads.iter().rev().find(|(head, _)| *head == lane) {
            Some((_, cursor)) => *cursor > 0,
            None => self.lanes.get(&lane).is_some_and(|queue| !queue.is_empty()),
        }
    }

    pub(crate) fn in_flight(&self, lane: Lane, matches: impl Fn(&Continuation) -> bool) -> bool {
        self.lanes
            .get(&lane)
            .is_some_and(|queue| queue.iter().any(|queued| matches(queued.then())))
    }

    pub(crate) fn queue(&mut self, event: u64, pending: Pending) {
        let lane = pending.lane;
        self.insert(lane, Queued::Asked(event, Box::new(pending)));
    }

    pub(crate) fn defer(&mut self, event: u64, lane: Lane, then: Continuation) {
        self.insert(lane, Queued::Due(event, then));
    }

    fn insert(&mut self, lane: Lane, queued: Queued) {
        let queue = self.lanes.entry(lane).or_default();
        match self.heads.iter_mut().rev().find(|(head, _)| *head == lane) {
            Some((_, cursor)) => {
                queue.insert((*cursor).min(queue.len()), queued);
                *cursor += 1;
            }
            None => queue.push_back(queued),
        }
    }

    pub(crate) fn enter(&mut self, lane: Lane) {
        self.heads.push((lane, 0));
    }

    pub(crate) fn leave(&mut self) {
        self.heads.pop();
    }

    pub(crate) fn hold(&mut self, lane: impl Into<Lane>, held: Held) -> Result<(), Held> {
        let last = self.lanes.get_mut(&lane.into()).and_then(|queue| {
            queue.iter_mut().rev().find_map(|queued| match queued {
                Queued::Asked(_, pending) if pending.then.is_move() => Some(pending),
                _ => None,
            })
        });
        match last {
            Some(last) => {
                last.held.push(held);
                Ok(())
            }
            None => Err(held),
        }
    }

    fn pending_mut(&mut self, event: u64) -> Option<&mut Pending> {
        self.lanes
            .values_mut()
            .flatten()
            .find_map(|queued| match queued {
                Queued::Asked(order, pending) if *order == event => Some(pending.as_mut()),
                _ => None,
            })
    }

    pub(crate) fn decide(
        &mut self,
        connection: u64,
        event: u64,
        verdicts: Vec<Verdict>,
        tick: u64,
    ) -> (Taken, Vec<String>) {
        let mut warnings = Vec::new();
        let taken = match self.pending_mut(event) {
            Some(pending) => pending.take(connection, event, verdicts, &mut warnings),
            None => self.late(connection, event, verdicts, tick, &mut warnings),
        };
        (taken, warnings)
    }

    fn late(
        &mut self,
        connection: u64,
        event: u64,
        verdicts: Vec<Verdict>,
        tick: u64,
        warnings: &mut Vec<String>,
    ) -> Taken {
        let Some(record) = self.settled.get(&event).filter(|record| record.kept(tick)) else {
            return if verdicts.iter().any(|verdict| verdict.cancel) {
                Taken::TooLate("cancel")
            } else if verdicts.iter().any(|verdict| verdict.patch.is_some()) {
                Taken::TooLate("patch")
            } else {
                Taken::Ignored
            };
        };
        if record.applied.is_none() {
            return Taken::Ignored;
        }
        if !record.silent_from(connection) {
            if record.answered_by(connection) {
                warnings.push(answered_twice(event));
            }
            return Taken::Ignored;
        }
        let admitted = admit(&record.silent, connection, event, verdicts, warnings);
        let (lane, undo) = (record.lane, record.undo.filter(|_| tick <= record.until));
        let cancel = admitted.iter().any(|(_, verdict)| verdict.cancel);
        let patched = admitted.iter().any(|(_, verdict)| verdict.patch.is_some());
        match (cancel, undo) {
            (true, Some(undo)) => Taken::RolledBack(self.roll_back(event, lane, undo)),
            (true, None) => Taken::TooLate("cancel"),
            (false, _) if patched => Taken::TooLate("patch"),
            (false, _) => Taken::Ignored,
        }
    }

    fn roll_back(&mut self, event: u64, lane: Lane, undo: Undo) -> Rollback {
        let undone = self
            .settled
            .range_mut(event..)
            .filter(|(_, later)| later.lane == lane && later.undo.is_some())
            .filter_map(|(id, later)| later.applied.take().map(|payload| (*id, payload)))
            .collect();
        let dropped = self.drop_pending(lane);
        Rollback {
            lane,
            undo,
            undone,
            dropped,
        }
    }

    pub(crate) fn release(&mut self, present: impl Fn(u64) -> bool) {
        for queued in self.lanes.values_mut().flatten() {
            if let Queued::Asked(_, pending) = queued {
                pending.waiting.retain(|connection| present(*connection));
            }
        }
    }

    pub(crate) fn next_settled(&mut self, tick: u64) -> Option<Ready> {
        let (lane, event) = self
            .lanes
            .iter()
            .filter_map(|(lane, queue)| match queue.front()? {
                Queued::Asked(event, pending) if pending.ready(tick) => Some((*lane, *event)),
                _ => None,
            })
            .min_by_key(|(_, event)| *event)?;
        match self.pop(lane)? {
            Queued::Asked(_, pending) => Some((*pending).settle(event)),
            Queued::Due(..) => None,
        }
    }

    pub(crate) fn next_due(&mut self) -> Option<Due> {
        let (lane, event) = self
            .lanes
            .iter()
            .filter_map(|(lane, queue)| match queue.front()? {
                Queued::Due(event, _) => Some((*lane, *event)),
                Queued::Asked(..) => None,
            })
            .min_by_key(|(_, event)| *event)?;
        match self.pop(lane)? {
            Queued::Due(_, then) => Some(Due { event, lane, then }),
            Queued::Asked(..) => None,
        }
    }

    pub(crate) fn first_lane(&self) -> Option<Lane> {
        self.lanes.keys().min().copied()
    }

    pub(crate) fn flush(&mut self, lane: Lane, logins: bool) -> Option<Flushed> {
        let front = self.lanes.get(&lane)?.front()?;
        if !logins && matches!(front.then(), Continuation::Login(_)) {
            return None;
        }
        let queued = self.pop(lane)?;
        for (head, cursor) in &mut self.heads {
            if *head == lane {
                *cursor = cursor.saturating_sub(1);
            }
        }
        Some(match queued {
            Queued::Asked(event, pending) => Flushed::Ready(Box::new((*pending).settle(event))),
            Queued::Due(event, then) => Flushed::Due(Due { event, lane, then }),
        })
    }

    fn pop(&mut self, lane: Lane) -> Option<Queued> {
        let queue = self.lanes.get_mut(&lane)?;
        let front = queue.pop_front();
        if queue.is_empty() {
            self.lanes.remove(&lane);
        }
        front
    }

    pub(crate) fn remember(&mut self, event: u64, record: Record) {
        self.settled.insert(event, record);
    }

    pub(crate) fn expire(&mut self, tick: u64) {
        self.settled.retain(|_, record| record.kept(tick));
    }

    pub(crate) fn drop_pending(&mut self, lane: impl Into<Lane>) -> Undone {
        let lane = lane.into();
        let Some(queue) = self.lanes.get_mut(&lane) else {
            return Vec::new();
        };
        let (moves, rest): (VecDeque<Queued>, VecDeque<Queued>) =
            std::mem::take(queue).into_iter().partition(Queued::moving);
        *queue = rest;
        if queue.is_empty() {
            self.lanes.remove(&lane);
        }
        let mut dropped = Vec::with_capacity(moves.len());
        for queued in moves {
            if let Queued::Asked(event, pending) = queued {
                self.remember(event, pending.record());
                dropped.push((event, pending.payload));
            }
        }
        dropped
    }

    pub(crate) fn forget(&mut self, lane: impl Into<Lane>) {
        let lane = lane.into();
        if let Some(queue) = self.lanes.get_mut(&lane) {
            queue.retain(|queued| !queued.moving());
            if queue.is_empty() {
                self.lanes.remove(&lane);
            }
        }
        self.settled
            .retain(|_, record| record.lane != lane || record.undo.is_none());
    }
}

#[cfg(test)]
mod tests {
    use rspigot_domain::Rotation;
    use rspigot_domain::player::GameMode;

    use super::*;
    use crate::plugins::events::{
        Action, ClickKind, EntityDamageByEntity, EntityDamageByEntityPatch, EntityDamageCause,
        EntityDamagePatch, EventResult, Face, InventoryAction, InventoryClick, InventoryClickPatch,
        InventoryTypeSlotType, PlayerInteract, PlayerInteractPatch, PlayerJoin, PlayerMove,
        PlayerMovePatch, PlayerRespawn, PlayerRespawnPatch, PlayerRespawnReason, PlayerTeleport,
        PlayerTeleportCause, PlayerTeleportPatch, PlayerVelocity, PlayerVelocityPatch,
    };
    use crate::plugins::{
        DamageSourceView, EntityView, InventoryOwner, InventoryRef, InventoryView, Location,
        PlayerSnapshot,
    };

    fn snapshot(player: PlayerId, position: Vec3) -> PlayerSnapshot {
        PlayerSnapshot {
            id: player,
            name: "Alice".to_owned(),
            entity_id: 1,
            world: "minecraft:overworld".to_owned(),
            position,
            rotation: Rotation::default(),
            game_mode: GameMode::Survival,
        }
    }

    fn location(position: Vec3) -> Location {
        Location {
            world: "minecraft:overworld".to_owned(),
            position,
            rotation: Rotation::default(),
        }
    }

    fn payload(player: PlayerId, from: Vec3, to: Vec3) -> EventPayload {
        EventPayload::PlayerMove(PlayerMove {
            player: snapshot(player, from),
            from: location(from),
            to: Some(location(to)),
        })
    }

    fn step(player: PlayerId, from: Vec3, to: Vec3) -> Continuation {
        Continuation::PlayerMove(MoveStep {
            player,
            from,
            to,
            rotation: None,
            on_ground: true,
        })
    }

    fn handlers(asked: &[(u64, u32)]) -> Vec<Asked> {
        asked
            .iter()
            .map(|&(connection, handler)| Asked {
                connection,
                handler,
                kind: EventKind::PlayerMove,
            })
            .collect()
    }

    fn asking(player: PlayerId, x: f64, asked: &[(u64, u32)]) -> Pending {
        let from = Vec3::new(x - 1.0, 64.0, 0.0);
        let to = Vec3::new(x, 64.0, 0.0);
        Pending::new(
            player,
            payload(player, from, to),
            handlers(asked),
            5,
            20,
            step(player, from, to),
        )
    }

    fn pending(player: PlayerId, x: f64, waiting: &[u64]) -> Pending {
        let asked: Vec<(u64, u32)> = waiting.iter().map(|&connection| (connection, 1)).collect();
        asking(player, x, &asked)
    }

    fn record(player: PlayerId, silent: &[u64], until: u64) -> Record {
        let from = Vec3::new(0.0, 64.0, 0.0);
        let silent: Vec<(u64, u32)> = silent.iter().map(|&connection| (connection, 1)).collect();
        Record {
            lane: player.into(),
            undo: Some(Undo::SendBack(from)),
            silent: handlers(&silent),
            answered: Vec::new(),
            until,
            deadline: 5,
            applied: Some(payload(player, from, from)),
        }
    }

    fn verdict(handler: u32, cancel: bool, x: Option<f64>) -> Verdict {
        Verdict {
            handler,
            cancel,
            patch: x.map(|x| {
                EventPatch::PlayerMove(PlayerMovePatch {
                    to: Some(location(Vec3::new(x, 64.0, 0.0))),
                })
            }),
        }
    }

    fn cancel(cancel: bool) -> Vec<Verdict> {
        vec![verdict(1, cancel, None)]
    }

    fn taken(
        decisions: &mut Decisions,
        connection: u64,
        event: u64,
        verdicts: Vec<Verdict>,
        tick: u64,
    ) -> Taken {
        decisions.decide(connection, event, verdicts, tick).0
    }

    fn settled_ids(decisions: &mut Decisions, tick: u64) -> Vec<u64> {
        std::iter::from_fn(|| decisions.next_settled(tick).map(|ready| ready.event)).collect()
    }

    fn ids(undone: &Undone) -> Vec<u64> {
        undone.iter().map(|(event, _)| *event).collect()
    }

    fn rolled_back(taken: Taken) -> (Vec<u64>, Vec<u64>) {
        match taken {
            Taken::RolledBack(rollback) => (ids(&rollback.undone), ids(&rollback.dropped)),
            _ => panic!("expected a rollback"),
        }
    }

    fn arrivals(count: usize) -> Vec<Vec<usize>> {
        (0..count).fold(vec![Vec::new()], |known, next| {
            known
                .into_iter()
                .flat_map(|order: Vec<usize>| {
                    (0..=order.len()).map(move |at| {
                        let mut longer = order.clone();
                        longer.insert(at, next);
                        longer
                    })
                })
                .collect()
        })
    }

    #[test]
    fn a_move_waits_for_the_earlier_moves_of_its_player_only() {
        let alice = PlayerId::offline("Alice");
        let bob = PlayerId::offline("Bob");
        let mut decisions = Decisions::default();
        decisions.queue(1, pending(alice, 1.0, &[7]));
        decisions.queue(2, pending(alice, 2.0, &[]));
        decisions.queue(3, pending(bob, 1.0, &[]));
        assert_eq!(decisions.reference(alice), Some(Vec3::new(2.0, 64.0, 0.0)));
        assert!(decisions.busy(alice));
        assert_eq!(settled_ids(&mut decisions, 0), [3]);
        assert!(!decisions.busy(bob));
        assert!(matches!(
            taken(&mut decisions, 7, 1, cancel(false), 0),
            Taken::Counted
        ));
        assert_eq!(settled_ids(&mut decisions, 0), [1, 2]);
        assert_eq!(decisions.reference(alice), None);
        assert!(!decisions.busy(alice));
    }

    fn notice(player: PlayerId, waiting: &[u64]) -> Pending {
        let at = Vec3::new(0.0, 64.0, 0.0);
        let asked = waiting
            .iter()
            .map(|&connection| Asked {
                connection,
                handler: 1,
                kind: EventKind::PlayerJoin,
            })
            .collect();
        Pending::new(
            player,
            EventPayload::PlayerJoin(PlayerJoin {
                player: snapshot(player, at),
                join_message: None,
            }),
            asked,
            5,
            20,
            Continuation::Notice(EventKind::PlayerJoin),
        )
    }

    #[test]
    fn an_event_born_while_the_head_of_its_lane_applies_goes_before_the_rest_of_the_lane() {
        let alice = PlayerId::offline("Alice");
        let lane = Lane::Player(alice);
        let mut decisions = Decisions::default();
        decisions.queue(1, pending(alice, 1.0, &[]));
        decisions.queue(2, notice(alice, &[]));
        let head = decisions.next_settled(0).expect("the first move is ready");
        assert_eq!(head.event, 1);
        decisions.enter(head.lane);
        assert!(!decisions.busy(alice));
        decisions.queue(3, notice(alice, &[7]));
        assert!(decisions.busy(alice));
        decisions.queue(4, notice(alice, &[]));
        decisions.leave();
        assert!(decisions.busy(lane));
        assert!(decisions.next_settled(0).is_none());
        assert!(matches!(
            taken(&mut decisions, 7, 3, cancel(false), 0),
            Taken::Counted
        ));
        assert_eq!(settled_ids(&mut decisions, 0), [3, 4, 2]);
    }

    #[test]
    fn a_grandchild_settles_before_the_second_child_of_its_grandparent() {
        let alice = PlayerId::offline("Alice");
        let mut decisions = Decisions::default();
        decisions.queue(1, notice(alice, &[]));
        decisions.queue(9, notice(alice, &[]));
        let parent = decisions.next_settled(0).expect("the parent is ready");
        decisions.enter(parent.lane);
        decisions.queue(2, notice(alice, &[]));
        decisions.queue(3, notice(alice, &[]));
        decisions.leave();
        let child = decisions.next_settled(0).expect("the first child is ready");
        assert_eq!(child.event, 2);
        decisions.enter(child.lane);
        decisions.queue(4, notice(alice, &[]));
        decisions.leave();
        assert_eq!(settled_ids(&mut decisions, 0), [4, 3, 9]);
    }

    #[test]
    fn a_deferred_entry_comes_out_at_the_head_of_its_lane_to_be_described() {
        let alice = PlayerId::offline("Alice");
        let bob = PlayerId::offline("Bob");
        let mut decisions = Decisions::default();
        decisions.queue(1, pending(alice, 1.0, &[7]));
        decisions.defer(
            2,
            alice.into(),
            Continuation::Notice(EventKind::PlayerInput),
        );
        decisions.defer(3, bob.into(), Continuation::Notice(EventKind::PlayerInput));
        let due = decisions.next_due().expect("Bob's entry is at the head");
        assert_eq!((due.event, due.lane), (3, Lane::Player(bob)));
        assert!(decisions.next_due().is_none());
        assert!(matches!(
            taken(&mut decisions, 7, 1, cancel(false), 0),
            Taken::Counted
        ));
        assert!(decisions.next_due().is_none());
        assert_eq!(settled_ids(&mut decisions, 0), [1]);
        let due = decisions
            .next_due()
            .expect("Alice's entry reached the head");
        assert_eq!(due.event, 2);
        assert!(matches!(
            due.then,
            Continuation::Notice(EventKind::PlayerInput)
        ));
        assert!(!decisions.busy(alice));
    }

    #[test]
    fn dropping_or_forgetting_a_lane_removes_its_moves_only() {
        let alice = PlayerId::offline("Alice");
        let mut decisions = Decisions::default();
        decisions.queue(1, notice(alice, &[7]));
        decisions.queue(2, pending(alice, 1.0, &[7]));
        decisions.queue(3, notice(alice, &[7]));
        decisions.queue(4, pending(alice, 2.0, &[7]));
        let dropped: Vec<u64> = ids(&decisions.drop_pending(alice));
        assert_eq!(dropped, [2, 4]);
        assert_eq!(decisions.reference(alice), None);
        decisions.queue(5, pending(alice, 3.0, &[7]));
        decisions.forget(alice);
        assert_eq!(decisions.reference(alice), None);
        for event in [1, 3] {
            assert!(matches!(
                taken(&mut decisions, 7, event, cancel(false), 0),
                Taken::Counted
            ));
        }
        assert_eq!(settled_ids(&mut decisions, 0), [1, 3]);
        assert!(!decisions.busy(alice));
    }

    #[test]
    fn an_entry_in_flight_is_found_by_its_continuation() {
        let alice = PlayerId::offline("Alice");
        let mut decisions = Decisions::default();
        decisions.queue(1, notice(alice, &[7]));
        let lane = Lane::Player(alice);
        assert!(decisions.in_flight(lane, |then| then.kind() == EventKind::PlayerJoin));
        assert!(!decisions.in_flight(lane, Continuation::is_move));
        assert!(!decisions.in_flight(Lane::Login(1), |_| true));
    }

    #[test]
    fn a_packet_that_is_not_an_event_waits_behind_the_last_pending_move() {
        let alice = PlayerId::offline("Alice");
        let mut decisions = Decisions::default();
        let turn = Held {
            position: None,
            rotation: Some(Rotation::new(90.0, 0.0)),
            on_ground: true,
        };
        assert_eq!(decisions.hold(alice, turn), Err(turn));
        decisions.queue(1, pending(alice, 1.0, &[]));
        decisions.queue(2, pending(alice, 2.0, &[]));
        assert_eq!(decisions.hold(alice, turn), Ok(()));
        let settled: Vec<Vec<Held>> =
            std::iter::from_fn(|| decisions.next_settled(0).map(|ready| ready.held)).collect();
        assert_eq!(settled, [Vec::new(), vec![turn]]);
    }

    #[test]
    fn a_late_cancel_counts_only_from_a_connection_that_stayed_silent() {
        let alice = PlayerId::offline("Alice");
        let mut decisions = Decisions::default();
        decisions.remember(4, record(alice, &[1, 2], 20));
        decisions.remember(5, record(alice, &[], 20));
        assert!(matches!(
            taken(&mut decisions, 3, 4, cancel(true), 3),
            Taken::Ignored
        ));
        assert!(matches!(
            taken(&mut decisions, 1, 4, cancel(false), 3),
            Taken::Ignored
        ));
        assert!(matches!(
            taken(&mut decisions, 1, 4, cancel(true), 21),
            Taken::TooLate("cancel")
        ));
        assert!(matches!(
            taken(&mut decisions, 1, 5, cancel(true), 3),
            Taken::Ignored
        ));
        assert_eq!(
            rolled_back(taken(&mut decisions, 1, 4, cancel(true), 20)),
            (vec![4, 5], vec![])
        );
        assert!(matches!(
            taken(&mut decisions, 2, 4, cancel(true), 20),
            Taken::Ignored
        ));
        decisions.expire(21);
        assert!(matches!(
            taken(&mut decisions, 2, 4, cancel(true), 21),
            Taken::TooLate("cancel")
        ));
    }

    #[test]
    fn a_rollback_undoes_what_came_after_it_for_that_player_only() {
        let alice = PlayerId::offline("Alice");
        let bob = PlayerId::offline("Bob");
        let mut decisions = Decisions::default();
        decisions.remember(1, record(alice, &[1], 20));
        decisions.remember(2, record(alice, &[1], 20));
        decisions.remember(3, record(bob, &[1], 20));
        decisions.remember(4, record(alice, &[1], 20));
        decisions.queue(5, pending(alice, 1.0, &[1]));
        decisions.queue(6, pending(bob, 1.0, &[1]));
        decisions.queue(7, pending(alice, 2.0, &[]));
        assert_eq!(
            rolled_back(taken(&mut decisions, 1, 2, cancel(true), 0)),
            (vec![2, 4], vec![5, 7])
        );
        assert!(matches!(
            taken(&mut decisions, 1, 4, cancel(true), 0),
            Taken::Ignored
        ));
        assert!(matches!(
            taken(&mut decisions, 1, 5, cancel(true), 0),
            Taken::Ignored
        ));
        assert!(matches!(
            taken(&mut decisions, 1, 7, cancel(true), 0),
            Taken::Ignored
        ));
        assert_eq!(decisions.reference(alice), None);
        assert!(decisions.reference(bob).is_some());
        assert_eq!(
            rolled_back(taken(&mut decisions, 1, 3, cancel(true), 0)),
            (vec![3], vec![6])
        );
        assert_eq!(
            rolled_back(taken(&mut decisions, 1, 1, cancel(true), 0)),
            (vec![1], vec![])
        );
    }

    #[test]
    fn answers_fold_in_the_order_handlers_were_asked_whatever_order_they_arrive() {
        let alice = PlayerId::offline("Alice");
        let asked = [(3, 1), (1, 1), (4, 1), (2, 1), (2, 2)];
        let patches = [
            (1, vec![verdict(1, false, Some(1.0))]),
            (
                2,
                vec![verdict(2, false, Some(3.0)), verdict(1, false, Some(2.0))],
            ),
            (3, vec![verdict(1, false, None)]),
            (4, vec![verdict(1, false, Some(4.0))]),
        ];
        let mut with_a_cancel = patches.clone();
        with_a_cancel[2].1 = vec![verdict(1, true, Some(9.0))];
        let three = EventPatch::PlayerMove(PlayerMovePatch {
            to: Some(location(Vec3::new(3.0, 64.0, 0.0))),
        });
        for (rulings, expected) in [
            (&patches, Settlement::Applied(Some(three))),
            (&with_a_cancel, Settlement::Cancelled(None)),
        ] {
            for arrival in arrivals(rulings.len()) {
                let mut decisions = Decisions::default();
                decisions.queue(1, asking(alice, 1.0, &asked));
                let mut settled = Vec::new();
                for (count, index) in arrival.iter().enumerate() {
                    let tick = if count < 2 { 0 } else { 1 };
                    let (connection, verdicts) = &rulings[*index];
                    let (_, warnings) = decisions.decide(*connection, 1, verdicts.clone(), tick);
                    assert!(warnings.is_empty());
                    while let Some(ready) = decisions.next_settled(tick) {
                        settled.push(ready.settlement);
                        decisions.remember(ready.event, ready.record);
                    }
                }
                assert_eq!(
                    settled,
                    std::slice::from_ref(&expected),
                    "arrival order {arrival:?}"
                );
            }
        }
    }

    #[test]
    fn a_patch_of_another_kind_is_ignored_with_a_warning() {
        let alice = PlayerId::offline("Alice");
        let at = Vec3::new(0.0, 64.0, 0.0);
        let mut decisions = Decisions::default();
        decisions.queue(
            1,
            Pending::new(
                alice,
                EventPayload::PlayerJoin(PlayerJoin {
                    player: snapshot(alice, at),
                    join_message: None,
                }),
                vec![Asked {
                    connection: 1,
                    handler: 1,
                    kind: EventKind::PlayerJoin,
                }],
                5,
                20,
                step(alice, at, at),
            ),
        );
        let (taken, warnings) = decisions.decide(1, 1, vec![verdict(1, false, Some(4.0))], 0);
        assert!(matches!(taken, Taken::Counted));
        assert_eq!(
            warnings,
            ["a PlayerMove patch does not fit event 1, which is a PlayerJoin"]
        );
        let ready = decisions.next_settled(0).expect("the event settles");
        assert_eq!(ready.settlement, Settlement::Applied(None));
    }

    fn refused_and_dropped(payload: &EventPayload, patching: impl Fn(Location) -> EventPatch) {
        let alice = PlayerId::offline("Alice");
        let at = Vec3::new(0.0, 64.0, 0.0);
        let nowhere = location(Vec3::new(f64::NAN, 64.0, 0.0));
        let elsewhere = location(Vec3::new(10.0, 64.0, 0.0));
        let kind = payload.kind();
        for (later, expected) in [
            (
                Some(elsewhere.clone()),
                Settlement::Applied(Some(patching(elsewhere.clone()))),
            ),
            (None, Settlement::Applied(None)),
        ] {
            let asked = [1, 2]
                .map(|connection| Asked {
                    connection,
                    handler: 1,
                    kind,
                })
                .to_vec();
            let mut decisions = Decisions::default();
            decisions.queue(
                1,
                Pending::new(alice, payload.clone(), asked, 5, 20, step(alice, at, at)),
            );
            let mut heard = Vec::new();
            for (connection, to) in [(1, Some(nowhere.clone())), (2, later)] {
                let ruling = Verdict {
                    handler: 1,
                    cancel: false,
                    patch: to.map(&patching),
                };
                let (taken, warnings) = decisions.decide(connection, 1, vec![ruling], 0);
                assert!(matches!(taken, Taken::Counted));
                heard.push(warnings);
            }
            assert_eq!(
                heard,
                [
                    vec![
                        "patch of event 1 was refused: position NaN 64 0 is outside the world"
                            .to_owned()
                    ],
                    Vec::new(),
                ],
                "{kind:?}"
            );
            let ready = decisions.next_settled(0).expect("the event settles");
            assert_eq!(ready.settlement, expected, "{kind:?}");
        }
    }

    #[test]
    fn a_refused_value_in_any_generated_patch_is_dropped_and_never_cancels() {
        let alice = PlayerId::offline("Alice");
        let at = Vec3::new(0.0, 64.0, 0.0);
        let respawning = EventPayload::PlayerRespawn(PlayerRespawn {
            anchor_spawn: false,
            bed_spawn: false,
            player: snapshot(alice, at),
            respawn_location: location(at),
            respawn_reason: PlayerRespawnReason::Death,
        });
        refused_and_dropped(&respawning, |to| {
            EventPatch::PlayerRespawn(PlayerRespawnPatch {
                respawn_location: Some(to),
            })
        });
        let teleporting = EventPayload::PlayerTeleport(PlayerTeleport {
            cause: PlayerTeleportCause::Command,
            from: location(at),
            player: snapshot(alice, at),
            to: Some(location(at)),
        });
        refused_and_dropped(&teleporting, |to| {
            EventPatch::PlayerTeleport(PlayerTeleportPatch {
                from: None,
                to: Some(to),
            })
        });
    }

    fn interacting(player: PlayerId) -> EventPayload {
        let at = Vec3::new(0.0, 64.0, 0.0);
        EventPayload::PlayerInteract(PlayerInteract {
            action: Action::RightClickBlock,
            block: true,
            block_face: Face::Up,
            block_in_hand: false,
            clicked_block: None,
            clicked_position: None,
            hand: None,
            has_item: false,
            item: None,
            material: "minecraft:air".to_owned(),
            player: snapshot(player, at),
            use_interacted_block: EventResult::Default,
            use_item_in_hand: EventResult::Default,
        })
    }

    fn clicking(player: PlayerId) -> EventPayload {
        let at = Vec3::new(0.0, 64.0, 0.0);
        let menu = InventoryRef {
            owner: InventoryOwner::Custom(7),
            size: 27,
            kind: "CHEST".to_owned(),
        };
        EventPayload::InventoryClick(InventoryClick {
            action: InventoryAction::Unknown,
            click: ClickKind::Left,
            clicked_inventory: Some(menu.clone()),
            current_item: None,
            cursor: None,
            hotbar_button: -1,
            inventory: menu.clone(),
            left_click: true,
            raw_slot: 4,
            result: EventResult::Default,
            right_click: false,
            shift_click: false,
            slot: 4,
            slot_type: InventoryTypeSlotType::Container,
            view: InventoryView {
                top: menu.clone(),
                bottom: menu,
                title: "Menu".to_owned(),
                player: snapshot(player, at),
            },
            viewers: Vec::new(),
            who_clicked: snapshot(player, at),
        })
    }

    fn ruling(cancel: bool, patch: Option<EventPatch>) -> Vec<Verdict> {
        vec![Verdict {
            handler: 1,
            cancel,
            patch,
        }]
    }

    fn deciding(
        payload: EventPayload,
        connections: u64,
        rulings: Vec<Vec<Verdict>>,
    ) -> (Decisions, Vec<String>) {
        let alice = PlayerId::offline("Alice");
        let at = Vec3::new(0.0, 64.0, 0.0);
        let kind = payload.kind();
        let asked = (1..=connections)
            .map(|connection| Asked {
                connection,
                handler: 1,
                kind,
            })
            .collect();
        let mut decisions = Decisions::default();
        decisions.queue(
            1,
            Pending::new(alice, payload, asked, 5, 20, step(alice, at, at)),
        );
        let mut warnings = Vec::new();
        for (connection, verdicts) in (1..).zip(rulings) {
            let (taken, heard) = decisions.decide(connection, 1, verdicts, 0);
            assert!(matches!(taken, Taken::Counted));
            warnings.extend(heard);
        }
        (decisions, warnings)
    }

    fn settled(payload: EventPayload, rulings: Vec<Vec<Verdict>>) -> (Settlement, Vec<String>) {
        let connections = rulings.len() as u64;
        let (mut decisions, warnings) = deciding(payload, connections, rulings);
        let ready = decisions.next_settled(0).expect("every handler answered");
        (ready.settlement, warnings)
    }

    #[test]
    fn a_result_event_is_cancelled_by_its_main_result_only() {
        let alice = PlayerId::offline("Alice");
        let item_denied = EventPatch::PlayerInteract(PlayerInteractPatch {
            use_item_in_hand: Some(EventResult::Deny),
            ..PlayerInteractPatch::default()
        });
        assert_eq!(
            settled(
                interacting(alice),
                vec![ruling(false, Some(item_denied.clone()))]
            ),
            (Settlement::Applied(Some(item_denied)), Vec::new())
        );
        let denied = EventPatch::PlayerInteract(PlayerInteractPatch {
            use_interacted_block: Some(EventResult::Deny),
            use_item_in_hand: Some(EventResult::Deny),
        });
        assert_eq!(
            settled(interacting(alice), vec![ruling(true, None)]),
            (Settlement::Cancelled(Some(denied)), Vec::new())
        );
    }

    #[test]
    fn a_later_handler_lifts_the_cancel_of_a_result_event() {
        let alice = PlayerId::offline("Alice");
        let result = |result| {
            EventPatch::InventoryClick(InventoryClickPatch {
                result: Some(result),
                ..InventoryClickPatch::default()
            })
        };
        let (mut decisions, warnings) = deciding(
            clicking(alice),
            2,
            vec![
                ruling(true, None),
                ruling(false, Some(result(EventResult::Allow))),
            ],
        );
        assert!(warnings.is_empty());
        let ready = decisions.next_settled(0).expect("every handler answered");
        assert_eq!(
            ready.settlement,
            Settlement::Applied(Some(result(EventResult::Allow)))
        );
        assert_eq!(
            settled(
                clicking(alice),
                vec![
                    ruling(false, Some(result(EventResult::Allow))),
                    ruling(true, None)
                ]
            ),
            (
                Settlement::Cancelled(Some(result(EventResult::Deny))),
                Vec::new()
            )
        );
    }

    #[test]
    fn a_cancel_on_an_event_that_cannot_be_cancelled_is_ignored_with_a_warning() {
        let alice = PlayerId::offline("Alice");
        let at = Vec3::new(0.0, 64.0, 0.0);
        let respawning = EventPayload::PlayerRespawn(PlayerRespawn {
            anchor_spawn: false,
            bed_spawn: false,
            player: snapshot(alice, at),
            respawn_location: location(at),
            respawn_reason: PlayerRespawnReason::Death,
        });
        let elsewhere = EventPatch::PlayerRespawn(PlayerRespawnPatch {
            respawn_location: Some(location(Vec3::new(10.0, 64.0, 0.0))),
        });
        assert_eq!(
            settled(respawning, vec![ruling(true, Some(elsewhere.clone()))]),
            (
                Settlement::Applied(Some(elsewhere)),
                vec!["event 1 cannot be cancelled, the cancel of handler 1 is ignored".to_owned()]
            )
        );
    }

    #[test]
    fn a_velocity_that_is_not_finite_is_dropped_and_does_not_cancel() {
        let alice = PlayerId::offline("Alice");
        let pushed = EventPayload::PlayerVelocity(PlayerVelocity {
            player: snapshot(alice, Vec3::new(0.0, 64.0, 0.0)),
            velocity: Vec3::new(0.0, 0.5, 0.0),
        });
        let nowhere = EventPatch::PlayerVelocity(PlayerVelocityPatch {
            velocity: Some(Vec3::new(f64::NAN, 0.5, 0.0)),
        });
        assert_eq!(
            settled(pushed, vec![ruling(false, Some(nowhere))]),
            (
                Settlement::Applied(None),
                vec!["patch of event 1 was refused: velocity NaN 0.5 0 is not finite".to_owned()]
            )
        );
    }

    #[test]
    fn a_cancelled_click_keeps_its_direct_writes_and_waits_for_every_handler() {
        let alice = PlayerId::offline("Alice");
        let cleared = EventPatch::InventoryClick(InventoryClickPatch {
            current_item: Some(None),
            ..InventoryClickPatch::default()
        });
        let (mut decisions, warnings) =
            deciding(clicking(alice), 2, vec![ruling(true, Some(cleared))]);
        assert!(warnings.is_empty());
        assert!(decisions.next_settled(0).is_none());
        let (taken, _) = decisions.decide(2, 1, ruling(false, None), 0);
        assert!(matches!(taken, Taken::Counted));
        let ready = decisions.next_settled(0).expect("every handler answered");
        assert_eq!(
            ready.settlement,
            Settlement::Cancelled(Some(EventPatch::InventoryClick(InventoryClickPatch {
                current_item: Some(None),
                cursor: None,
                result: Some(EventResult::Deny),
            })))
        );
    }

    #[test]
    fn a_decision_without_a_cancel_warns_for_each_handler_left_without_a_verdict() {
        let alice = PlayerId::offline("Alice");
        let mut decisions = Decisions::default();
        decisions.queue(1, asking(alice, 1.0, &[(1, 1), (2, 1), (1, 2), (1, 3)]));
        let (taken, warnings) = decisions.decide(1, 1, vec![verdict(2, false, None)], 0);
        assert!(matches!(taken, Taken::Counted));
        assert_eq!(
            warnings,
            [
                "handler 1 of event 1 gave no verdict",
                "handler 3 of event 1 gave no verdict",
            ]
        );
        let (_, warnings) = decisions.decide(2, 1, Vec::new(), 0);
        assert_eq!(warnings, ["handler 1 of event 1 gave no verdict"]);
    }

    #[test]
    fn a_decision_with_a_cancel_leaves_its_other_handlers_without_a_warning() {
        let alice = PlayerId::offline("Alice");
        let mut decisions = Decisions::default();
        decisions.queue(1, asking(alice, 1.0, &[(1, 1), (1, 2), (1, 3)]));
        let (taken, warnings) = decisions.decide(1, 1, vec![verdict(2, true, None)], 0);
        assert!(matches!(taken, Taken::Counted));
        assert!(warnings.is_empty());
        let ready = decisions.next_settled(0).expect("the event settles");
        assert_eq!(ready.settlement, Settlement::Cancelled(None));
    }

    #[test]
    fn a_second_decision_from_a_plugin_is_ignored_with_a_warning() {
        let alice = PlayerId::offline("Alice");
        let mut decisions = Decisions::default();
        decisions.queue(1, pending(alice, 1.0, &[1, 2]));
        assert!(matches!(
            taken(&mut decisions, 1, 1, cancel(false), 0),
            Taken::Counted
        ));
        let (again, warnings) = decisions.decide(1, 1, cancel(true), 0);
        assert!(matches!(again, Taken::Ignored));
        assert_eq!(warnings, ["event 1 was already answered by this plugin"]);
        let (stranger, warnings) = decisions.decide(3, 1, cancel(true), 0);
        assert!(matches!(stranger, Taken::Ignored));
        assert!(warnings.is_empty());
        assert!(decisions.next_settled(0).is_none());
    }

    #[test]
    fn a_late_verdict_on_an_event_that_did_not_go_through_is_ignored_silently() {
        let alice = PlayerId::offline("Alice");
        let mut decisions = Decisions::default();
        let mut called_off = record(alice, &[1], 20);
        called_off.answered = handlers(&[(2, 1)]);
        called_off.applied = None;
        decisions.remember(1, called_off);
        for (connection, verdicts) in [
            (1, vec![verdict(1, true, Some(4.0))]),
            (1, vec![verdict(7, false, None)]),
            (2, cancel(true)),
            (3, vec![verdict(1, false, Some(4.0))]),
        ] {
            let (taken, warnings) = decisions.decide(connection, 1, verdicts, 20);
            assert!(matches!(taken, Taken::Ignored));
            assert!(warnings.is_empty());
        }
    }

    #[test]
    fn a_second_decision_on_an_applied_event_warns_the_plugin_that_answered() {
        let alice = PlayerId::offline("Alice");
        let mut decisions = Decisions::default();
        let mut applied = record(alice, &[1], 20);
        applied.answered = handlers(&[(2, 1)]);
        decisions.remember(1, applied);
        let (again, warnings) = decisions.decide(2, 1, cancel(true), 3);
        assert!(matches!(again, Taken::Ignored));
        assert_eq!(warnings, ["event 1 was already answered by this plugin"]);
        let (stranger, warnings) = decisions.decide(3, 1, cancel(true), 3);
        assert!(matches!(stranger, Taken::Ignored));
        assert!(warnings.is_empty());
    }

    #[test]
    fn a_settled_event_is_kept_until_its_deadline_but_rolls_back_only_within_its_window() {
        let alice = PlayerId::offline("Alice");
        let mut decisions = Decisions::default();
        let mut short = record(alice, &[1], 2);
        short.deadline = 5;
        decisions.remember(1, short);
        decisions.expire(5);
        assert!(matches!(
            taken(&mut decisions, 1, 1, cancel(true), 4),
            Taken::TooLate("cancel")
        ));
        assert!(matches!(
            taken(&mut decisions, 1, 1, vec![verdict(1, false, Some(4.0))], 4),
            Taken::TooLate("patch")
        ));
        assert_eq!(
            rolled_back(taken(&mut decisions, 1, 1, cancel(true), 2)),
            (vec![1], vec![])
        );
        let mut settled = Decisions::default();
        settled.queue(2, asking(alice, 1.0, &[(1, 1)]));
        let ready = settled.next_settled(9).expect("the deadline has passed");
        let mut applied = ready.record;
        applied.applied = Some(ready.payload);
        settled.remember(ready.event, applied);
        settled.expire(20);
        assert!(matches!(
            taken(&mut settled, 1, 2, cancel(true), 20),
            Taken::RolledBack(_)
        ));
        settled.expire(21);
        assert!(matches!(
            taken(&mut settled, 1, 2, cancel(true), 21),
            Taken::TooLate("cancel")
        ));
    }

    #[test]
    fn a_late_patch_is_too_late_unless_the_event_was_called_off() {
        let alice = PlayerId::offline("Alice");
        let mut decisions = Decisions::default();
        decisions.remember(1, record(alice, &[1], 20));
        let mut cancelled = record(alice, &[1], 20);
        cancelled.applied = None;
        decisions.remember(2, cancelled);
        let patch = || vec![verdict(1, false, Some(4.0))];
        assert!(matches!(
            taken(&mut decisions, 1, 1, patch(), 3),
            Taken::TooLate("patch")
        ));
        assert!(matches!(
            taken(&mut decisions, 2, 1, patch(), 3),
            Taken::Ignored
        ));
        assert!(matches!(
            taken(&mut decisions, 1, 2, patch(), 3),
            Taken::Ignored
        ));
        assert!(matches!(
            taken(&mut decisions, 1, 9, patch(), 3),
            Taken::TooLate("patch")
        ));
        assert!(matches!(
            taken(&mut decisions, 1, 1, Vec::new(), 3),
            Taken::Ignored
        ));
        let (unknown, warnings) = decisions.decide(1, 1, vec![verdict(5, true, None)], 3);
        assert!(matches!(unknown, Taken::Ignored));
        assert_eq!(warnings, ["handler 5 was not asked to decide event 1"]);
        assert_eq!(
            rolled_back(taken(
                &mut decisions,
                1,
                1,
                vec![verdict(1, true, Some(4.0))],
                3
            )),
            (vec![1], vec![])
        );
    }

    fn entity(id: i32) -> EntityView {
        EntityView {
            id: rspigot_domain::EntityId(id),
            uuid: uuid::Uuid::from_u128(u128::try_from(id).unwrap_or_default()),
            kind: "minecraft:zombie".to_owned(),
            position: Vec3::new(0.0, 64.0, 0.0),
        }
    }

    fn hit(damage: f64) -> EventPayload {
        EventPayload::EntityDamageByEntity(EntityDamageByEntity {
            cause: EntityDamageCause::EntityAttack,
            damage,
            damage_source: DamageSourceView {
                kind: "minecraft:player_attack".to_owned(),
                causing: Some(entity(2)),
                direct: Some(entity(2)),
                location: None,
            },
            damager: entity(2),
            entity: entity(1),
            entity_type: "minecraft:zombie".to_owned(),
            final_damage: damage,
        })
    }

    fn damage(kind: EventKind, handler: u32, amount: f64) -> Verdict {
        let patch = if kind == EventKind::EntityDamage {
            EventPatch::EntityDamage(EntityDamagePatch {
                damage: Some(amount),
            })
        } else {
            EventPatch::EntityDamageByEntity(EntityDamageByEntityPatch {
                damage: Some(amount),
            })
        };
        Verdict {
            handler,
            cancel: false,
            patch: Some(patch),
        }
    }

    fn asked_in(order: &[(u64, u32, EventKind)]) -> Vec<Asked> {
        order
            .iter()
            .map(|&(connection, handler, kind)| Asked {
                connection,
                handler,
                kind,
            })
            .collect()
    }

    fn settle_hit(order: &[(u64, u32, EventKind)], arrival: &[usize]) -> (Settlement, Vec<String>) {
        let alice = PlayerId::offline("Alice");
        let at = Vec3::new(0.0, 64.0, 0.0);
        let mut decisions = Decisions::default();
        decisions.queue(
            1,
            Pending::new(alice, hit(4.0), asked_in(order), 5, 20, step(alice, at, at)),
        );
        let mut warnings = Vec::new();
        for &index in arrival {
            let (connection, handler, kind) = order[index];
            let amount = if kind == EventKind::EntityDamage {
                1.0
            } else {
                9.0
            };
            let (_, said) = decisions.decide(connection, 1, vec![damage(kind, handler, amount)], 0);
            warnings.extend(said);
        }
        let ready = decisions.next_settled(0).expect("the hit settles");
        (ready.settlement, warnings)
    }

    #[test]
    fn a_family_folds_the_patches_of_a_parent_and_a_child_handler_in_registry_order() {
        let parent_first = [
            (1, 1, EventKind::EntityDamage),
            (2, 1, EventKind::EntityDamageByEntity),
        ];
        let child_first = [parent_first[1], parent_first[0]];
        let by = |amount| {
            Settlement::Applied(Some(EventPatch::EntityDamageByEntity(
                EntityDamageByEntityPatch {
                    damage: Some(amount),
                },
            )))
        };
        for arrival in arrivals(2) {
            assert_eq!(settle_hit(&parent_first, &arrival), (by(9.0), Vec::new()));
            assert_eq!(settle_hit(&child_first, &arrival), (by(1.0), Vec::new()));
        }
    }

    #[test]
    fn a_handler_patches_only_the_event_it_registered_for() {
        let alice = PlayerId::offline("Alice");
        let at = Vec3::new(0.0, 64.0, 0.0);
        let mut decisions = Decisions::default();
        decisions.queue(
            1,
            Pending::new(
                alice,
                hit(4.0),
                asked_in(&[(1, 1, EventKind::EntityDamage)]),
                5,
                20,
                step(alice, at, at),
            ),
        );
        let (_, warnings) = decisions.decide(
            1,
            1,
            vec![damage(EventKind::EntityDamageByEntity, 1, 2.0)],
            0,
        );
        assert_eq!(
            warnings,
            ["a EntityDamageByEntity patch does not fit event 1, which is a EntityDamage"]
        );
        let ready = decisions.next_settled(0).expect("the hit settles");
        assert_eq!(ready.settlement, Settlement::Applied(None));
    }
}
