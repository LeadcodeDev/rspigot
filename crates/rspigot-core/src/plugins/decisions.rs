use std::collections::BTreeMap;

use rspigot_domain::player::PlayerId;
use rspigot_domain::{Rotation, Vec3};
use rustc_hash::FxHashSet;

use crate::plugins::{EventKind, EventPatch, EventPayload, Verdict};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Held {
    pub(crate) position: Option<Vec3>,
    pub(crate) rotation: Option<Rotation>,
    pub(crate) on_ground: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct MoveStep {
    pub(crate) player: PlayerId,
    pub(crate) from: Vec3,
    pub(crate) to: Vec3,
    pub(crate) rotation: Option<Rotation>,
    pub(crate) on_ground: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Continuation {
    PlayerMove(MoveStep),
}

impl Continuation {
    pub(crate) const fn kind(&self) -> EventKind {
        match self {
            Self::PlayerMove(_) => EventKind::PlayerMove,
        }
    }

    const fn destination(&self) -> Option<Vec3> {
        match self {
            Self::PlayerMove(step) => Some(step.to),
        }
    }

    const fn undo(&self) -> Option<Undo> {
        match self {
            Self::PlayerMove(step) => Some(Undo::SendBack(step.from)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Undo {
    SendBack(Vec3),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Asked {
    pub(crate) connection: u64,
    pub(crate) handler: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Answer {
    cancel: bool,
    patch: Option<EventPatch>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Settlement {
    Cancelled,
    Applied(Option<EventPatch>),
}

pub(crate) struct Pending {
    lane: PlayerId,
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
        lane: PlayerId,
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
            lane,
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
        self.cancelled() || self.waiting.is_empty() || tick >= self.deadline
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
            return Taken::Ignored;
        };
        self.waiting.swap_remove(index);
        let kind = self.payload.kind();
        for (slot, verdict) in admit(&self.asked, connection, event, verdicts, warnings) {
            let mut patch = verdict.patch;
            if let Some(misfit) = patch.filter(|patch| patch.kind() != kind) {
                warnings.push(format!(
                    "a {:?} patch does not fit event {event}, which is a {kind:?}",
                    misfit.kind()
                ));
                patch = None;
            }
            self.answers[slot] = Some(match patch.and_then(|patch| patch.refusal()) {
                Some(reason) => {
                    warnings.push(format!(
                        "patch of event {event} was refused, the event is cancelled: {reason}"
                    ));
                    Answer {
                        cancel: true,
                        patch: None,
                    }
                }
                None => Answer {
                    cancel: verdict.cancel,
                    patch,
                },
            });
        }
        Taken::Counted
    }

    fn record(&self) -> Record {
        Record {
            lane: self.lane,
            undo: self.then.undo(),
            silent: self
                .asked
                .iter()
                .filter(|ask| self.waiting.contains(&ask.connection))
                .copied()
                .collect(),
            until: self.until,
            applied: None,
        }
    }

    fn settle(self, event: u64) -> Ready {
        let record = self.record();
        Ready {
            event,
            payload: self.payload,
            settlement: fold(self.answers),
            then: self.then,
            held: self.held,
            record,
        }
    }
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
        match asked.iter().position(|ask| {
            *ask == Asked {
                connection,
                handler,
            }
        }) {
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

fn fold(answers: Vec<Option<Answer>>) -> Settlement {
    let mut patch: Option<EventPatch> = None;
    for answer in answers.into_iter().flatten() {
        if answer.cancel {
            return Settlement::Cancelled;
        }
        patch = match (patch, answer.patch) {
            (Some(earlier), Some(later)) => Some(earlier.then(later)),
            (earlier, later) => later.or(earlier),
        };
    }
    Settlement::Applied(patch)
}

pub(crate) struct Ready {
    pub(crate) event: u64,
    pub(crate) payload: EventPayload,
    pub(crate) settlement: Settlement,
    pub(crate) then: Continuation,
    pub(crate) held: Vec<Held>,
    pub(crate) record: Record,
}

pub(crate) struct Record {
    lane: PlayerId,
    undo: Option<Undo>,
    silent: Vec<Asked>,
    until: u64,
    pub(crate) applied: Option<EventPayload>,
}

impl Record {
    fn silent_from(&self, connection: u64) -> bool {
        self.silent.iter().any(|ask| ask.connection == connection)
    }
}

pub(crate) type Undone = Vec<(u64, EventPayload)>;

pub(crate) struct Rollback {
    pub(crate) lane: PlayerId,
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

#[derive(Default)]
pub(crate) struct Decisions {
    pending: BTreeMap<u64, Pending>,
    settled: BTreeMap<u64, Record>,
}

impl Decisions {
    pub(crate) fn reference(&self, lane: PlayerId) -> Option<Vec3> {
        self.pending
            .values()
            .rev()
            .filter(|pending| pending.lane == lane)
            .find_map(|pending| pending.then.destination())
    }

    pub(crate) fn busy(&self, lane: PlayerId) -> bool {
        self.pending.values().any(|pending| pending.lane == lane)
    }

    pub(crate) fn queue(&mut self, event: u64, pending: Pending) {
        self.pending.insert(event, pending);
    }

    pub(crate) fn hold(&mut self, lane: PlayerId, held: Held) -> Result<(), Held> {
        match self
            .pending
            .values_mut()
            .rev()
            .find(|pending| pending.lane == lane)
        {
            Some(last) => {
                last.held.push(held);
                Ok(())
            }
            None => Err(held),
        }
    }

    pub(crate) fn decide(
        &mut self,
        connection: u64,
        event: u64,
        verdicts: Vec<Verdict>,
        tick: u64,
    ) -> (Taken, Vec<String>) {
        let mut warnings = Vec::new();
        let taken = match self.pending.get_mut(&event) {
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
        let Some(record) = self
            .settled
            .get(&event)
            .filter(|record| tick <= record.until)
        else {
            return if verdicts.iter().any(|verdict| verdict.cancel) {
                Taken::TooLate("cancel")
            } else if verdicts.iter().any(|verdict| verdict.patch.is_some()) {
                Taken::TooLate("patch")
            } else {
                Taken::Ignored
            };
        };
        if !record.silent_from(connection) {
            return Taken::Ignored;
        }
        let admitted = admit(&record.silent, connection, event, verdicts, warnings);
        if record.applied.is_none() {
            return Taken::Ignored;
        }
        let (lane, undo) = (record.lane, record.undo);
        let cancel = admitted.iter().any(|(_, verdict)| verdict.cancel);
        let patched = admitted.iter().any(|(_, verdict)| verdict.patch.is_some());
        match (cancel, undo) {
            (true, Some(undo)) => Taken::RolledBack(self.roll_back(event, lane, undo)),
            (true, None) => Taken::TooLate("cancel"),
            (false, _) if patched => Taken::TooLate("patch"),
            (false, _) => Taken::Ignored,
        }
    }

    fn roll_back(&mut self, event: u64, lane: PlayerId, undo: Undo) -> Rollback {
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
        for pending in self.pending.values_mut() {
            pending.waiting.retain(|connection| present(*connection));
        }
    }

    pub(crate) fn next_settled(&mut self, tick: u64) -> Option<Ready> {
        let mut seen = FxHashSet::default();
        let event = self.pending.iter().find_map(|(event, pending)| {
            (seen.insert(pending.lane) && pending.ready(tick)).then_some(*event)
        })?;
        self.pending
            .remove(&event)
            .map(|pending| pending.settle(event))
    }

    pub(crate) fn remember(&mut self, event: u64, record: Record) {
        if record.applied.is_some() || !record.silent.is_empty() {
            self.settled.insert(event, record);
        }
    }

    pub(crate) fn expire(&mut self, tick: u64) {
        self.settled.retain(|_, record| tick <= record.until);
    }

    pub(crate) fn drop_pending(&mut self, lane: PlayerId) -> Undone {
        let events: Vec<u64> = self
            .pending
            .iter()
            .filter(|(_, pending)| pending.lane == lane)
            .map(|(event, _)| *event)
            .collect();
        let mut dropped = Vec::with_capacity(events.len());
        for event in events {
            if let Some(pending) = self.pending.remove(&event) {
                self.remember(event, pending.record());
                dropped.push((event, pending.payload));
            }
        }
        dropped
    }

    pub(crate) fn forget(&mut self, lane: PlayerId) {
        self.pending.retain(|_, pending| pending.lane != lane);
        self.settled.retain(|_, record| record.lane != lane);
    }
}

#[cfg(test)]
mod tests {
    use rspigot_domain::player::GameMode;

    use super::*;
    use crate::plugins::{PlayerMovePatch, PlayerSnapshot};

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

    fn payload(player: PlayerId, from: Vec3, to: Vec3) -> EventPayload {
        EventPayload::PlayerMove {
            player: snapshot(player, from),
            from,
            to,
        }
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
            lane: player,
            undo: Some(Undo::SendBack(from)),
            silent: handlers(&silent),
            until,
            applied: Some(payload(player, from, from)),
        }
    }

    fn verdict(handler: u32, cancel: bool, x: Option<f64>) -> Verdict {
        Verdict {
            handler,
            cancel,
            patch: x.map(|x| {
                EventPatch::PlayerMove(PlayerMovePatch {
                    to: Some(Vec3::new(x, 64.0, 0.0)),
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
            Taken::TooLate("cancel")
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
            to: Some(Vec3::new(3.0, 64.0, 0.0)),
        });
        for (rulings, expected) in [
            (&patches, Settlement::Applied(Some(three))),
            (&with_a_cancel, Settlement::Cancelled),
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
                assert_eq!(settled, [expected], "arrival order {arrival:?}");
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
                EventPayload::PlayerJoin {
                    player: snapshot(alice, at),
                },
                handlers(&[(1, 1)]),
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
}
