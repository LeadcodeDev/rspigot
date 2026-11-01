use std::net::IpAddr;
use std::time::SystemTime;

use rspigot_domain::player::GameProfile;
use rspigot_domain::text::Component;
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

    pub fn matches_entry(&self, other: &Self) -> bool {
        self.matches_name(&other.name) || (other.uuid.is_some() && self.uuid == other.uuid)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ban<T> {
    pub target: T,
    pub created: SystemTime,
    pub source: String,
    pub expires: Option<SystemTime>,
    pub reason: Option<String>,
}

pub type BanEntry = Ban<ListEntry>;
pub type IpBanEntry = Ban<IpAddr>;

pub fn ban_reason(reason: Option<&str>) -> Component {
    reason.map_or_else(
        || Component::translatable("multiplayer.disconnect.banned.reason.default", Vec::new()),
        Component::text,
    )
}

impl<T> Ban<T> {
    pub fn reason_message(&self) -> Component {
        ban_reason(self.reason.as_deref())
    }

    pub fn has_expired(&self, now: SystemTime) -> bool {
        self.expires.is_some_and(|expires| expires < now)
    }
}

impl BanEntry {
    fn bans(&self, profile: &GameProfile) -> bool {
        match self.target.uuid {
            Some(uuid) => uuid == profile.id.0,
            None => self.target.matches_name(&profile.name),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AdminLists {
    pub operators: Vec<ListEntry>,
    pub whitelist: Vec<ListEntry>,
    pub whitelist_enabled: bool,
    pub bans: Vec<BanEntry>,
    pub ip_bans: Vec<IpBanEntry>,
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

    pub fn profile_ban(&self, profile: &GameProfile, now: SystemTime) -> Option<&BanEntry> {
        self.bans
            .iter()
            .find(|ban| ban.bans(profile) && !ban.has_expired(now))
    }

    pub fn ip_ban(&self, address: IpAddr, now: SystemTime) -> Option<&IpBanEntry> {
        let address = address.to_canonical();
        self.ip_bans
            .iter()
            .find(|ban| ban.target == address && !ban.has_expired(now))
    }

    pub fn learn_uuid(&mut self, profile: &GameProfile) -> bool {
        let mut learned = false;
        for ban in &mut self.bans {
            if ban.target.uuid.is_none() && ban.target.matches_name(&profile.name) {
                ban.target.uuid = Some(profile.id.0);
                learned = true;
            }
        }
        learned
    }

    pub fn prune_expired(&mut self, now: SystemTime) -> bool {
        let before = self.bans.len() + self.ip_bans.len();
        self.bans.retain(|ban| !ban.has_expired(now));
        self.ip_bans.retain(|ban| !ban.has_expired(now));
        self.bans.len() + self.ip_bans.len() != before
    }

    pub fn add(list: &mut Vec<ListEntry>, entry: ListEntry) -> bool {
        if let Some(existing) = list.iter_mut().find(|e| e.matches_entry(&entry)) {
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
    use std::time::{Duration, UNIX_EPOCH};

    use rspigot_domain::player::PlayerId;

    use super::*;

    fn noon() -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(1_793_534_400)
    }

    fn ban<T>(target: T, expires: Option<SystemTime>) -> Ban<T> {
        Ban {
            target,
            created: noon(),
            source: "Server".to_owned(),
            expires,
            reason: None,
        }
    }

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

    #[test]
    fn a_uuid_entry_matches_the_uuid_and_a_name_only_entry_the_name() {
        let renamed = GameProfile {
            name: "Alex".to_owned(),
            ..profile("Steve")
        };
        let mut lists = AdminLists {
            bans: vec![
                ban(ListEntry::of(&profile("Steve")), None),
                ban(ListEntry::named("CAROL"), None),
            ],
            ..AdminLists::default()
        };
        assert_eq!(
            lists.profile_ban(&renamed, noon()).map(|b| &b.target),
            Some(&ListEntry::of(&profile("Steve")))
        );
        assert!(lists.profile_ban(&profile("steve"), noon()).is_none());
        assert_eq!(
            lists
                .profile_ban(&profile("carol"), noon())
                .map(|b| &b.target),
            Some(&ListEntry::named("CAROL"))
        );
        assert!(lists.profile_ban(&profile("Dave"), noon()).is_none());
        assert!(lists.learn_uuid(&profile("carol")));
        assert_eq!(
            lists.bans[1].target.uuid,
            Some(PlayerId::offline("carol").0)
        );
        assert!(!lists.learn_uuid(&profile("carol")));
    }

    #[test]
    fn expired_entries_are_pruned() {
        let hour = Duration::from_secs(3600);
        let address = IpAddr::from([203, 0, 113, 7]);
        let mut lists = AdminLists {
            bans: vec![
                ban(ListEntry::named("Gone"), Some(noon() - hour)),
                ban(ListEntry::named("Kept"), Some(noon() + hour)),
                ban(ListEntry::named("Forever"), None),
            ],
            ip_bans: vec![ban(address, Some(noon() - hour))],
            ..AdminLists::default()
        };
        assert!(lists.profile_ban(&profile("Gone"), noon()).is_none());
        assert!(lists.ip_ban(address, noon()).is_none());
        assert!(lists.ip_ban(address, noon() - hour - hour).is_some());
        let mapped = "::ffff:203.0.113.7".parse().expect("an address");
        assert!(lists.ip_ban(mapped, noon() - hour - hour).is_some());
        assert!(lists.prune_expired(noon()));
        let names: Vec<&str> = lists.bans.iter().map(|b| b.target.name.as_str()).collect();
        assert_eq!(names, ["Kept", "Forever"]);
        assert!(lists.ip_bans.is_empty());
        assert!(!lists.prune_expired(noon()));
        assert_eq!(
            lists.bans[0].reason_message(),
            Component::translatable("multiplayer.disconnect.banned.reason.default", Vec::new())
        );
    }
}
