pub use crate::generated::tags::TAGS;

pub struct RegistryTags {
    pub registry: &'static str,
    pub tags: &'static [(&'static str, &'static [u32])],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HolderSet {
    Tag(&'static str),
    Direct(&'static [u32]),
}

impl HolderSet {
    pub fn ids(&self, registry: &str) -> &'static [u32] {
        match self {
            Self::Tag(name) => tag(registry, name).unwrap_or(&[]),
            Self::Direct(ids) => ids,
        }
    }

    pub fn contains(&self, registry: &str, id: u32) -> bool {
        self.ids(registry).binary_search(&id).is_ok()
    }
}

pub fn tag(registry: &str, name: &str) -> Option<&'static [u32]> {
    let registry = TAGS
        .binary_search_by(|candidate| candidate.registry.cmp(registry))
        .ok()?;
    let tags = TAGS[registry].tags;
    tags.binary_search_by(|(candidate, _)| (*candidate).cmp(name))
        .ok()
        .map(|index| tags[index].1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registries;

    #[test]
    fn tag_lookup() {
        let stone = registries::static_index("minecraft:block", "minecraft:stone").unwrap();
        let dirt = registries::static_index("minecraft:block", "minecraft:dirt").unwrap();
        let pickaxe = tag("minecraft:block", "minecraft:mineable/pickaxe").unwrap();
        assert!(pickaxe.contains(&stone));
        assert!(tag("minecraft:block", "minecraft:nope").is_none());
        assert!(tag("minecraft:nope", "minecraft:mineable/pickaxe").is_none());
        let set = HolderSet::Tag("minecraft:mineable/pickaxe");
        assert!(set.contains("minecraft:block", stone));
        assert!(!set.contains("minecraft:block", dirt));
        assert!(!HolderSet::Tag("minecraft:nope").contains("minecraft:block", stone));
        assert_eq!(set.ids("minecraft:block"), pickaxe);
        let direct = HolderSet::Direct(&[3, 9]);
        assert!(direct.contains("minecraft:block", 9));
        assert!(!direct.contains("minecraft:block", 4));
        assert_eq!(direct.ids("minecraft:block"), &[3, 9]);
        assert!(
            HolderSet::Tag("minecraft:nope")
                .ids("minecraft:block")
                .is_empty()
        );
    }
}
