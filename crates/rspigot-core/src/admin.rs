use rspigot_domain::player::GameProfile;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListEntry {
    pub name: String,
    pub uuid: Option<Uuid>,
}

impl ListEntry {
    pub fn named(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            uuid: None,
        }
    }

    pub fn of(profile: &GameProfile) -> Self {
        Self {
            name: profile.name.clone(),
            uuid: Some(profile.id.0),
        }
    }

    pub fn matches_profile(&self, profile: &GameProfile) -> bool {
        self.uuid == Some(profile.id.0) || self.name.eq_ignore_ascii_case(&profile.name)
    }

    pub fn matches_name(&self, name: &str) -> bool {
        self.name.eq_ignore_ascii_case(name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AdminLists {
    pub operators: Vec<ListEntry>,
    pub whitelist: Vec<ListEntry>,
    pub whitelist_enabled: bool,
}

impl AdminLists {
    pub fn is_operator(&self, profile: &GameProfile) -> bool {
        self.operators.iter().any(|e| e.matches_profile(profile))
    }

    pub fn may_join(&self, profile: &GameProfile) -> bool {
        !self.whitelist_enabled
            || self.is_operator(profile)
            || self.whitelist.iter().any(|e| e.matches_profile(profile))
    }

    pub fn add(list: &mut Vec<ListEntry>, entry: ListEntry) -> bool {
        if let Some(existing) = list
            .iter_mut()
            .find(|e| e.matches_name(&entry.name) || (entry.uuid.is_some() && e.uuid == entry.uuid))
        {
            if existing.uuid.is_none() && entry.uuid.is_some() {
                *existing = entry;
            }
            return false;
        }
        list.push(entry);
        true
    }

    pub fn remove(list: &mut Vec<ListEntry>, name: &str) -> bool {
        let before = list.len();
        list.retain(|e| !e.matches_name(name));
        list.len() != before
    }
}

#[cfg(test)]
mod tests {
    use rspigot_domain::player::PlayerId;

    use super::*;

    fn profile(name: &str) -> GameProfile {
        GameProfile {
            id: PlayerId::offline(name),
            name: name.to_owned(),
            properties: Vec::new(),
        }
    }

    #[test]
    fn whitelist_blocks_unlisted_players_but_not_operators() {
        let mut lists = AdminLists {
            whitelist_enabled: true,
            ..AdminLists::default()
        };
        assert!(!lists.may_join(&profile("Alice")));
        AdminLists::add(&mut lists.whitelist, ListEntry::named("alice"));
        assert!(lists.may_join(&profile("Alice")));
        AdminLists::add(&mut lists.operators, ListEntry::of(&profile("Bob")));
        assert!(lists.may_join(&profile("Bob")));
        lists.whitelist_enabled = false;
        assert!(lists.may_join(&profile("Carol")));
    }

    #[test]
    fn add_is_idempotent_and_remove_reports_change() {
        let mut list = Vec::new();
        assert!(AdminLists::add(&mut list, ListEntry::named("Alice")));
        assert!(!AdminLists::add(
            &mut list,
            ListEntry::of(&profile("alice"))
        ));
        assert_eq!(list[0].uuid, Some(PlayerId::offline("alice").0));
        assert!(AdminLists::remove(&mut list, "ALICE"));
        assert!(!AdminLists::remove(&mut list, "alice"));
    }
}
