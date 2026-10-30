use crate::damage::{DamageTag, DamageType};
use crate::text::Component;

const SIGNIFICANT_FALL: f32 = 5.0;
const COMBAT_TIMEOUT: u64 = 300;
const DAMAGE_TIMEOUT: u64 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallLocation {
    Generic,
    Water,
}

impl FallLocation {
    pub const fn language_key(self) -> &'static str {
        match self {
            Self::Generic => "death.fell.accident.generic",
            Self::Water => "death.fell.accident.water",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CombatEntry {
    pub kind: DamageType,
    pub causing: Option<Component>,
    pub direct: Option<Component>,
    pub living_cause: bool,
    pub damage: f32,
    pub fall_distance: f32,
    pub location: Option<FallLocation>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CombatTracker {
    entries: Vec<CombatEntry>,
    last_damage: u64,
    taking_damage: bool,
    in_combat: bool,
}

impl CombatTracker {
    pub fn record(&mut self, entry: CombatEntry, now: u64, alive: bool) {
        self.recheck(now, alive);
        let living = entry.living_cause;
        self.entries.push(entry);
        self.last_damage = now;
        self.taking_damage = true;
        if !self.in_combat && alive && living {
            self.in_combat = true;
        }
    }

    pub fn recheck(&mut self, now: u64, alive: bool) {
        let timeout = if self.in_combat {
            COMBAT_TIMEOUT
        } else {
            DAMAGE_TIMEOUT
        };
        if self.taking_damage && (!alive || now.saturating_sub(self.last_damage) > timeout) {
            self.taking_damage = false;
            self.in_combat = false;
            self.entries.clear();
        }
    }

    pub fn entries(&self) -> &[CombatEntry] {
        &self.entries
    }

    fn most_significant_fall(&self) -> Option<&CombatEntry> {
        let mut fall: Option<&CombatEntry> = None;
        let mut fall_distance = 0.0f32;
        let mut located: Option<&CombatEntry> = None;
        let mut located_damage = 0.0f32;
        for (index, entry) in self.entries.iter().enumerate() {
            let always = entry.kind.is(DamageTag::AlwaysMostSignificantFall);
            let distance = if always {
                f32::MAX
            } else {
                entry.fall_distance
            };
            if (entry.kind.is(DamageTag::IsFall) || always)
                && distance > 0.0
                && (fall.is_none() || distance > fall_distance)
            {
                fall = if index > 0 {
                    self.entries.get(index - 1)
                } else {
                    Some(entry)
                };
                fall_distance = distance;
            }
            if entry.location.is_some() && (located.is_none() || entry.damage > located_damage) {
                located = Some(entry);
                located_damage = entry.damage;
            }
        }
        if fall_distance > SIGNIFICANT_FALL && fall.is_some() {
            fall
        } else if located_damage > SIGNIFICANT_FALL {
            located
        } else {
            None
        }
    }

    pub fn death_message(&self, victim: &Component, kill_credit: Option<&Component>) -> Component {
        let Some(last) = self.entries.last() else {
            return Component::translatable("death.attack.generic", vec![victim.clone()]);
        };
        if last.kind.fall_variants()
            && let Some(fall) = self.most_significant_fall()
        {
            return fall_message(fall, last.causing.as_ref(), victim);
        }
        localized(last, victim, kill_credit)
    }
}

fn fall_message(fall: &CombatEntry, killer: Option<&Component>, victim: &Component) -> Component {
    if fall.kind.is(DamageTag::IsFall) || fall.kind.is(DamageTag::AlwaysMostSignificantFall) {
        let location = fall.location.unwrap_or(FallLocation::Generic);
        return Component::translatable(location.language_key(), vec![victim.clone()]);
    }
    match (fall.causing.as_ref(), killer) {
        (Some(assist), _) if killer != Some(assist) => {
            Component::translatable("death.fell.assist", vec![victim.clone(), assist.clone()])
        }
        (_, Some(killer)) => {
            Component::translatable("death.fell.finish", vec![victim.clone(), killer.clone()])
        }
        _ => Component::translatable("death.fell.killer", vec![victim.clone()]),
    }
}

fn localized(
    entry: &CombatEntry,
    victim: &Component,
    kill_credit: Option<&Component>,
) -> Component {
    let key = format!("death.attack.{}", entry.kind.message_id());
    match (
        entry.causing.as_ref().or(entry.direct.as_ref()),
        kill_credit,
    ) {
        (Some(source), _) => Component::translatable(key, vec![victim.clone(), source.clone()]),
        (None, Some(killer)) => Component::translatable(
            format!("{key}.player"),
            vec![victim.clone(), killer.clone()],
        ),
        (None, None) => Component::translatable(key, vec![victim.clone()]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(kind: DamageType, damage: f32, fall_distance: f32) -> CombatEntry {
        CombatEntry {
            kind,
            causing: None,
            direct: None,
            living_cause: false,
            damage,
            fall_distance,
            location: None,
        }
    }

    fn alice() -> Component {
        Component::text("Alice")
    }

    fn key(component: &Component) -> String {
        component.plain_text()
    }

    #[test]
    fn no_entries_means_generic_death() {
        let tracker = CombatTracker::default();
        assert_eq!(
            key(&tracker.death_message(&alice(), None)),
            "death.attack.generic Alice"
        );
    }

    #[test]
    fn long_fall_is_an_accident_and_a_short_one_hits_the_ground() {
        let mut tracker = CombatTracker::default();
        tracker.record(entry(DamageType::Fall, 17.0, 20.0), 1, true);
        assert_eq!(
            key(&tracker.death_message(&alice(), None)),
            "death.fell.accident.generic Alice"
        );
        let mut short = CombatTracker::default();
        short.record(entry(DamageType::Fall, 2.0, 4.5), 1, true);
        assert_eq!(
            key(&short.death_message(&alice(), None)),
            "death.attack.fall Alice"
        );
    }

    #[test]
    fn falling_after_a_hit_credits_the_attacker() {
        let mut tracker = CombatTracker::default();
        let mut hit = entry(DamageType::PlayerAttack, 1.0, 0.0);
        hit.causing = Some(Component::text("Bob"));
        hit.direct = Some(Component::text("Bob"));
        hit.living_cause = true;
        tracker.record(hit, 1, true);
        tracker.record(entry(DamageType::Fall, 18.0, 21.0), 30, true);
        assert_eq!(
            key(&tracker.death_message(&alice(), None)),
            "death.fell.assist Alice Bob"
        );
    }

    #[test]
    fn sources_with_entities_name_them_and_kill_credit_adds_the_player_variant() {
        let mut tracker = CombatTracker::default();
        let mut anvil = entry(DamageType::FallingAnvil, 20.0, 0.0);
        anvil.causing = Some(Component::text("Falling Block"));
        tracker.record(anvil, 1, true);
        assert_eq!(
            key(&tracker.death_message(&alice(), None)),
            "death.attack.anvil Alice Falling Block"
        );
        let mut starving = CombatTracker::default();
        starving.record(entry(DamageType::Starve, 1.0, 0.0), 1, true);
        let bob = Component::text("Bob");
        assert_eq!(
            key(&starving.death_message(&alice(), Some(&bob))),
            "death.attack.starve.player Alice Bob"
        );
        let mut void = CombatTracker::default();
        void.record(entry(DamageType::OutOfWorld, 4.0, 0.0), 1, true);
        assert_eq!(
            key(&void.death_message(&alice(), None)),
            "death.attack.outOfWorld Alice"
        );
    }

    #[test]
    fn entries_expire_after_a_quiet_period_or_death() {
        let mut tracker = CombatTracker::default();
        tracker.record(entry(DamageType::Magic, 1.0, 0.0), 0, true);
        tracker.recheck(100, true);
        assert_eq!(tracker.entries().len(), 1);
        tracker.recheck(101, true);
        assert!(tracker.entries().is_empty());
        tracker.record(entry(DamageType::Magic, 1.0, 0.0), 200, true);
        tracker.recheck(201, false);
        assert!(tracker.entries().is_empty());
    }
}
