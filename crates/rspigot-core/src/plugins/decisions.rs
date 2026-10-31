use std::collections::BTreeMap;

use rspigot_domain::player::PlayerId;
use rspigot_domain::{Rotation, Vec3};
use rustc_hash::FxHashSet;

use crate::plugins::EventPayload;

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
}

impl PendingMove {
    fn ready(&self, tick: u64) -> bool {
        self.cancelled || self.waiting.is_empty() || tick >= self.deadline
    }
}

pub(crate) struct SettledMove {
    pub(crate) player: PlayerId,
    pub(crate) from: Vec3,
    pub(crate) payload: EventPayload,
    pub(crate) silent: Vec<u64>,
    pub(crate) applied: bool,
    pub(crate) until: u64,
}

pub(crate) enum Verdict {
    Counted,
    Ignored,
    RolledBack(Box<SettledMove>),
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
        match self.settled.get(&event) {
            Some(settled) if settled.until > tick => {
                if settled.applied && settled.silent.contains(&connection) {
                    self.roll_back(event).map_or(Verdict::TooLate, |rolled| {
                        Verdict::RolledBack(Box::new(rolled))
                    })
                } else {
                    Verdict::Ignored
                }
            }
            _ => Verdict::TooLate,
        }
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
        self.settled.insert(event, settled);
    }

    pub(crate) fn expire(&mut self, tick: u64) {
        self.settled.retain(|_, settled| settled.until > tick);
    }

    pub(crate) fn drop_pending(&mut self, player: PlayerId) {
        self.pending.retain(|_, pending| pending.player != player);
    }

    pub(crate) fn forget(&mut self, player: PlayerId) {
        self.drop_pending(player);
        self.settled.retain(|_, settled| settled.player != player);
    }

    fn roll_back(&mut self, event: u64) -> Option<SettledMove> {
        let rolled = self.settled.remove(&event)?;
        let player = rolled.player;
        self.settled
            .retain(|later, settled| *later < event || settled.player != player);
        self.drop_pending(player);
        Some(rolled)
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
        }
    }

    fn settled(player: PlayerId, silent: Vec<u64>, until: u64) -> SettledMove {
        let from = Vec3::new(0.0, 64.0, 0.0);
        SettledMove {
            player,
            from,
            payload: payload(player, from, from),
            silent,
            applied: true,
            until,
        }
    }

    fn settled_ids(decisions: &mut Decisions, tick: u64) -> Vec<u64> {
        std::iter::from_fn(|| decisions.next_settled(tick).map(|(event, _)| event)).collect()
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
    fn a_late_cancel_counts_only_from_a_connection_that_stayed_silent() {
        let alice = PlayerId::offline("Alice");
        let mut decisions = Decisions::default();
        decisions.remember(4, settled(alice, vec![1], 20));
        assert!(matches!(decisions.decide(2, 4, true, 3), Verdict::Ignored));
        assert!(matches!(decisions.decide(1, 4, false, 3), Verdict::Ignored));
        assert!(matches!(decisions.decide(1, 4, true, 20), Verdict::TooLate));
        assert!(matches!(
            decisions.decide(1, 4, true, 19),
            Verdict::RolledBack(rolled) if rolled.player == alice
        ));
        assert!(matches!(decisions.decide(1, 4, true, 19), Verdict::TooLate));

        let mut cancelled = settled(alice, vec![1], 20);
        cancelled.applied = false;
        decisions.remember(5, cancelled);
        assert!(matches!(decisions.decide(1, 5, true, 3), Verdict::Ignored));
        assert!(matches!(decisions.decide(1, 5, true, 20), Verdict::TooLate));
    }

    #[test]
    fn a_rollback_forgets_what_came_after_it_for_that_player_only() {
        let alice = PlayerId::offline("Alice");
        let bob = PlayerId::offline("Bob");
        let mut decisions = Decisions::default();
        decisions.remember(1, settled(alice, vec![1], 20));
        decisions.remember(2, settled(alice, vec![1], 20));
        decisions.remember(3, settled(bob, vec![1], 20));
        decisions.remember(4, settled(alice, vec![1], 20));
        decisions.queue(5, pending(alice, 1.0, vec![1]));
        decisions.queue(6, pending(bob, 1.0, vec![1]));
        assert!(matches!(
            decisions.decide(1, 2, true, 0),
            Verdict::RolledBack(_)
        ));
        assert!(matches!(decisions.decide(1, 4, true, 0), Verdict::TooLate));
        assert_eq!(decisions.reference(alice), None);
        assert!(decisions.reference(bob).is_some());
        assert!(matches!(
            decisions.decide(1, 3, true, 0),
            Verdict::RolledBack(_)
        ));
        assert!(matches!(
            decisions.decide(1, 1, true, 0),
            Verdict::RolledBack(_)
        ));
    }
}
