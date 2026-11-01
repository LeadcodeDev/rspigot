use std::collections::BTreeMap;

use rspigot_domain::player::PlayerId;
use rspigot_domain::{Rotation, Vec3};
use rustc_hash::FxHashSet;

use crate::plugins::EventPayload;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Held {
    pub(crate) position: Option<Vec3>,
    pub(crate) rotation: Option<Rotation>,
    pub(crate) on_ground: bool,
}

pub(crate) struct PendingMove {
    pub(crate) player: PlayerId,
    pub(crate) from: Vec3,
    pub(crate) to: Vec3,
    pub(crate) rotation: Option<Rotation>,
    pub(crate) on_ground: bool,
    pub(crate) payload: EventPayload,
    pub(crate) waiting: Vec<u64>,
    pub(crate) cancelled: bool,
    pub(crate) deadline: u64,
    pub(crate) until: u64,
    pub(crate) held: Vec<Held>,
}

impl PendingMove {
    fn ready(&self, tick: u64) -> bool {
        self.cancelled || self.waiting.is_empty() || tick >= self.deadline
    }
}

pub(crate) struct SettledMove {
    pub(crate) player: PlayerId,
    pub(crate) from: Vec3,
    pub(crate) silent: Vec<u64>,
    pub(crate) until: u64,
    pub(crate) applied: Option<EventPayload>,
}

pub(crate) type Undone = Vec<(u64, EventPayload)>;

pub(crate) struct Rollback {
    pub(crate) player: PlayerId,
    pub(crate) from: Vec3,
    pub(crate) undone: Undone,
    pub(crate) dropped: Undone,
}

pub(crate) enum Verdict {
    Counted,
    Ignored,
    RolledBack(Rollback),
    TooLate,
}

#[derive(Default)]
pub(crate) struct Decisions {
    pending: BTreeMap<u64, PendingMove>,
    settled: BTreeMap<u64, SettledMove>,
}

impl Decisions {
    pub(crate) fn reference(&self, player: PlayerId) -> Option<Vec3> {
        self.pending
            .values()
            .rev()
            .find(|pending| pending.player == player)
            .map(|pending| pending.to)
    }

    pub(crate) fn queue(&mut self, event: u64, pending: PendingMove) {
        self.pending.insert(event, pending);
    }

    pub(crate) fn hold(&mut self, player: PlayerId, held: Held) -> Result<(), Held> {
        match self
            .pending
            .values_mut()
            .rev()
            .find(|pending| pending.player == player)
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
        cancel: bool,
        tick: u64,
    ) -> Verdict {
        if let Some(pending) = self.pending.get_mut(&event) {
            let Some(index) = pending
                .waiting
                .iter()
                .position(|waiting| *waiting == connection)
            else {
                return Verdict::Ignored;
            };
            pending.waiting.swap_remove(index);
            pending.cancelled |= cancel;
            return Verdict::Counted;
        }
        if !cancel {
            return Verdict::Ignored;
        }
        let Some(settled) = self.settled.get(&event).filter(|s| tick <= s.until) else {
            return Verdict::TooLate;
        };
        if settled.applied.is_none() || !settled.silent.contains(&connection) {
            return Verdict::Ignored;
        }
        let (player, from) = (settled.player, settled.from);
        let undone = self
            .settled
            .range_mut(event..)
            .filter(|(_, later)| later.player == player)
            .filter_map(|(id, later)| later.applied.take().map(|payload| (*id, payload)))
            .collect();
        let dropped = self.drop_pending(player);
        Verdict::RolledBack(Rollback {
            player,
            from,
            undone,
            dropped,
        })
    }

    pub(crate) fn release(&mut self, present: impl Fn(u64) -> bool) {
        for pending in self.pending.values_mut() {
            pending.waiting.retain(|connection| present(*connection));
        }
    }

    pub(crate) fn next_settled(&mut self, tick: u64) -> Option<(u64, PendingMove)> {
        let mut seen = FxHashSet::default();
        let event = self.pending.iter().find_map(|(event, pending)| {
            (seen.insert(pending.player) && pending.ready(tick)).then_some(*event)
        })?;
        self.pending.remove_entry(&event)
    }

    pub(crate) fn remember(&mut self, event: u64, settled: SettledMove) {
        if settled.applied.is_some() || !settled.silent.is_empty() {
            self.settled.insert(event, settled);
        }
    }

    pub(crate) fn expire(&mut self, tick: u64) {
        self.settled.retain(|_, settled| tick <= settled.until);
    }

    pub(crate) fn drop_pending(&mut self, player: PlayerId) -> Undone {
        let events: Vec<u64> = self
            .pending
            .iter()
            .filter(|(_, pending)| pending.player == player)
            .map(|(event, _)| *event)
            .collect();
        let mut dropped = Vec::with_capacity(events.len());
        for event in events {
            if let Some(pending) = self.pending.remove(&event) {
                self.remember(
                    event,
                    SettledMove {
                        player,
                        from: pending.from,
                        silent: pending.waiting,
                        until: pending.until,
                        applied: None,
                    },
                );
                dropped.push((event, pending.payload));
            }
        }
        dropped
    }

    pub(crate) fn forget(&mut self, player: PlayerId) {
        self.pending.retain(|_, pending| pending.player != player);
        self.settled.retain(|_, settled| settled.player != player);
    }
}

#[cfg(test)]
mod tests {
    use rspigot_domain::player::GameMode;

    use super::*;
    use crate::plugins::PlayerSnapshot;

    fn payload(player: PlayerId, from: Vec3, to: Vec3) -> EventPayload {
        EventPayload::PlayerMove {
            player: PlayerSnapshot {
                id: player,
                name: "Alice".to_owned(),
                entity_id: 1,
                world: "minecraft:overworld".to_owned(),
                position: from,
                rotation: Rotation::default(),
                game_mode: GameMode::Survival,
            },
            from,
            to,
        }
    }

    fn pending(player: PlayerId, x: f64, waiting: Vec<u64>) -> PendingMove {
        let from = Vec3::new(x - 1.0, 64.0, 0.0);
        let to = Vec3::new(x, 64.0, 0.0);
        PendingMove {
            player,
            from,
            to,
            rotation: None,
            on_ground: true,
            payload: payload(player, from, to),
            waiting,
            cancelled: false,
            deadline: 5,
            until: 20,
            held: Vec::new(),
        }
    }

    fn settled(player: PlayerId, silent: Vec<u64>, until: u64) -> SettledMove {
        let from = Vec3::new(0.0, 64.0, 0.0);
        SettledMove {
            player,
            from,
            silent,
            until,
            applied: Some(payload(player, from, from)),
        }
    }

    fn settled_ids(decisions: &mut Decisions, tick: u64) -> Vec<u64> {
        std::iter::from_fn(|| decisions.next_settled(tick).map(|(event, _)| event)).collect()
    }

    fn ids(undone: &Undone) -> Vec<u64> {
        undone.iter().map(|(event, _)| *event).collect()
    }

    fn rolled_back(verdict: Verdict) -> (Vec<u64>, Vec<u64>) {
        match verdict {
            Verdict::RolledBack(rollback) => (ids(&rollback.undone), ids(&rollback.dropped)),
            _ => panic!("expected a rollback"),
        }
    }

    #[test]
    fn a_move_waits_for_the_earlier_moves_of_its_player_only() {
        let alice = PlayerId::offline("Alice");
        let bob = PlayerId::offline("Bob");
        let mut decisions = Decisions::default();
        decisions.queue(1, pending(alice, 1.0, vec![7]));
        decisions.queue(2, pending(alice, 2.0, Vec::new()));
        decisions.queue(3, pending(bob, 1.0, Vec::new()));
        assert_eq!(decisions.reference(alice), Some(Vec3::new(2.0, 64.0, 0.0)));
        assert_eq!(settled_ids(&mut decisions, 0), [3]);
        assert!(matches!(decisions.decide(7, 1, false, 0), Verdict::Counted));
        assert_eq!(settled_ids(&mut decisions, 0), [1, 2]);
        assert_eq!(decisions.reference(alice), None);
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
        decisions.queue(1, pending(alice, 1.0, Vec::new()));
        decisions.queue(2, pending(alice, 2.0, Vec::new()));
        assert_eq!(decisions.hold(alice, turn), Ok(()));
        let settled: Vec<Vec<Held>> =
            std::iter::from_fn(|| decisions.next_settled(0).map(|(_, settled)| settled.held))
                .collect();
        assert_eq!(settled, [Vec::new(), vec![turn]]);
    }

    #[test]
    fn a_late_cancel_counts_only_from_a_connection_that_stayed_silent() {
        let alice = PlayerId::offline("Alice");
        let mut decisions = Decisions::default();
        decisions.remember(4, settled(alice, vec![1, 2], 20));
        decisions.remember(5, settled(alice, Vec::new(), 20));
        assert!(matches!(decisions.decide(3, 4, true, 3), Verdict::Ignored));
        assert!(matches!(decisions.decide(1, 4, false, 3), Verdict::Ignored));
        assert!(matches!(decisions.decide(1, 4, true, 21), Verdict::TooLate));
        assert!(matches!(decisions.decide(1, 5, true, 3), Verdict::Ignored));
        assert_eq!(
            rolled_back(decisions.decide(1, 4, true, 20)),
            (vec![4, 5], vec![])
        );
        assert!(matches!(decisions.decide(2, 4, true, 20), Verdict::Ignored));
        decisions.expire(21);
        assert!(matches!(decisions.decide(2, 4, true, 21), Verdict::TooLate));
    }

    #[test]
    fn a_rollback_undoes_what_came_after_it_for_that_player_only() {
        let alice = PlayerId::offline("Alice");
        let bob = PlayerId::offline("Bob");
        let mut decisions = Decisions::default();
        decisions.remember(1, settled(alice, vec![1], 20));
        decisions.remember(2, settled(alice, vec![1], 20));
        decisions.remember(3, settled(bob, vec![1], 20));
        decisions.remember(4, settled(alice, vec![1], 20));
        decisions.queue(5, pending(alice, 1.0, vec![1]));
        decisions.queue(6, pending(bob, 1.0, vec![1]));
        decisions.queue(7, pending(alice, 2.0, Vec::new()));
        assert_eq!(
            rolled_back(decisions.decide(1, 2, true, 0)),
            (vec![2, 4], vec![5, 7])
        );
        assert!(matches!(decisions.decide(1, 4, true, 0), Verdict::Ignored));
        assert!(matches!(decisions.decide(1, 5, true, 0), Verdict::Ignored));
        assert!(matches!(decisions.decide(1, 7, true, 0), Verdict::TooLate));
        assert_eq!(decisions.reference(alice), None);
        assert!(decisions.reference(bob).is_some());
        assert_eq!(
            rolled_back(decisions.decide(1, 3, true, 0)),
            (vec![3], vec![6])
        );
        assert_eq!(
            rolled_back(decisions.decide(1, 1, true, 0)),
            (vec![1], vec![])
        );
    }
}
