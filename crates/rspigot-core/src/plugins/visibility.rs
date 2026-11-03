use std::collections::BTreeSet;

use rspigot_domain::player::PlayerId;
use rustc_hash::FxHashMap;

#[derive(Default)]
pub(crate) struct Visibility {
    hides: FxHashMap<(PlayerId, PlayerId), BTreeSet<String>>,
}

impl Visibility {
    pub(crate) fn hide(&mut self, plugin: &str, viewer: PlayerId, target: PlayerId) -> bool {
        let plugins = self.hides.entry((viewer, target)).or_default();
        let first = plugins.is_empty();
        if !plugins.contains(plugin) {
            plugins.insert(plugin.to_owned());
        }
        first
    }

    pub(crate) fn show(&mut self, plugin: &str, viewer: PlayerId, target: PlayerId) -> bool {
        let Some(plugins) = self.hides.get_mut(&(viewer, target)) else {
            return false;
        };
        plugins.remove(plugin);
        if !plugins.is_empty() {
            return false;
        }
        self.hides.remove(&(viewer, target));
        true
    }

    pub(crate) fn hides(&self, viewer: PlayerId, target: PlayerId) -> bool {
        self.hides.contains_key(&(viewer, target))
    }

    pub(crate) fn hidden_from(&self, viewer: PlayerId) -> Vec<PlayerId> {
        let mut hidden: Vec<PlayerId> = self
            .hides
            .keys()
            .filter(|(hider, _)| *hider == viewer)
            .map(|(_, target)| *target)
            .collect();
        hidden.sort_unstable();
        hidden
    }

    pub(crate) fn forget(&mut self, player: PlayerId) {
        self.hides
            .retain(|&(viewer, target), _| viewer != player && target != player);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn players() -> (PlayerId, PlayerId, PlayerId) {
        (
            PlayerId::offline("Alice"),
            PlayerId::offline("Bob"),
            PlayerId::offline("Carol"),
        )
    }

    #[test]
    fn hides_are_counted_per_plugin_name() {
        let (alice, bob, _) = players();
        let mut visibility = Visibility::default();
        assert!(!visibility.show("tester", alice, bob));
        assert!(visibility.hide("tester", alice, bob));
        assert!(!visibility.hide("tester", alice, bob));
        assert!(!visibility.hide("other", alice, bob));
        assert!(visibility.hides(alice, bob));
        assert!(!visibility.hides(bob, alice));
        assert_eq!(visibility.hidden_from(alice), [bob]);
        assert!(visibility.hidden_from(bob).is_empty());

        assert!(!visibility.show("tester", bob, alice));
        assert!(!visibility.show("tester", alice, bob));
        assert!(!visibility.show("tester", alice, bob));
        assert!(visibility.hides(alice, bob));
        assert!(visibility.show("other", alice, bob));
        assert!(!visibility.hides(alice, bob));
        assert!(visibility.hidden_from(alice).is_empty());
        assert!(visibility.hide("tester", alice, bob));
    }

    #[test]
    fn forgetting_a_player_ends_every_hide_it_takes_part_in() {
        let (alice, bob, carol) = players();
        let mut visibility = Visibility::default();
        visibility.hide("tester", alice, bob);
        visibility.hide("tester", alice, carol);
        visibility.hide("other", carol, alice);
        visibility.hide("tester", bob, carol);
        let mut expected = vec![bob, carol];
        expected.sort();
        assert_eq!(visibility.hidden_from(alice), expected);

        visibility.forget(alice);
        assert!(visibility.hidden_from(alice).is_empty());
        assert!(visibility.hidden_from(carol).is_empty());
        assert!(!visibility.hides(carol, alice));
        assert_eq!(visibility.hidden_from(bob), [carol]);
    }
}
