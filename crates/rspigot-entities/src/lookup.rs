use crate::kind::{NAMES, SPAWN_EGGS};
use crate::{EntityDimensions, EntityKind, EntityType, EntityTypeId};

const NAMESPACE: &str = "minecraft:";

macro_rules! entity_kinds {
    (count: $count:literal, kinds: [$($kind:ident),* $(,)?] $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[repr(u16)]
        pub enum EntityKind {
            $($kind,)*
        }

        impl EntityKind {
            pub const ALL: [Self; $count] = [$(Self::$kind,)*];

            pub const fn name(self) -> &'static str {
                match self {
                    $(Self::$kind => <$crate::types::$kind as $crate::EntityType>::NAME,)*
                }
            }

            pub const fn id(self) -> $crate::EntityTypeId {
                match self {
                    $(Self::$kind => <$crate::types::$kind as $crate::EntityType>::ID,)*
                }
            }

            pub const fn class(self) -> &'static str {
                match self {
                    $(Self::$kind => <$crate::types::$kind as $crate::EntityType>::CLASS,)*
                }
            }

            pub const fn category(self) -> $crate::MobCategory {
                match self {
                    $(Self::$kind => <$crate::types::$kind as $crate::EntityType>::CATEGORY,)*
                }
            }

            pub const fn dimensions(self) -> $crate::EntityDimensions {
                match self {
                    $(Self::$kind => $crate::lookup::dimensions_of::<$crate::types::$kind>(),)*
                }
            }

            pub const fn fire_immune(self) -> bool {
                match self {
                    $(Self::$kind => <$crate::types::$kind as $crate::EntityType>::FIRE_IMMUNE,)*
                }
            }

            pub const fn summonable(self) -> bool {
                match self {
                    $(Self::$kind => <$crate::types::$kind as $crate::EntityType>::SUMMONABLE,)*
                }
            }

            pub const fn can_spawn_far_from_player(self) -> bool {
                match self {
                    $(Self::$kind => <$crate::types::$kind as $crate::EntityType>::CAN_SPAWN_FAR_FROM_PLAYER,)*
                }
            }

            pub const fn tracking_range(self) -> u8 {
                match self {
                    $(Self::$kind => <$crate::types::$kind as $crate::EntityType>::CLIENT_TRACKING_RANGE,)*
                }
            }

            pub const fn update_interval(self) -> Option<u8> {
                match self {
                    $(Self::$kind => <$crate::types::$kind as $crate::EntityType>::UPDATE_INTERVAL,)*
                }
            }

            pub const fn spawn_egg(self) -> Option<u16> {
                match self {
                    $(Self::$kind => <$crate::types::$kind as $crate::EntityType>::SPAWN_EGG,)*
                }
            }

            pub const fn living(self) -> bool {
                match self {
                    $(Self::$kind => <$crate::types::$kind as $crate::EntityType>::LIVING,)*
                }
            }

            pub const fn default_attributes(self) -> &'static [(&'static str, f64)] {
                match self {
                    $(Self::$kind => <$crate::types::$kind as $crate::EntityType>::DEFAULT_ATTRIBUTES,)*
                }
            }
        }
    };
}

pub(crate) use entity_kinds;

pub(crate) const fn dimensions_of<T: EntityType>() -> EntityDimensions {
    EntityDimensions {
        width: T::WIDTH,
        height: T::HEIGHT,
        eye_height: T::EYE_HEIGHT,
    }
}

const _: () = {
    let mut index = 0;
    while index < EntityKind::ALL.len() {
        let kind = EntityKind::ALL[index];
        assert!(kind.id().0 as usize == index);
        assert!(kind.living() != kind.default_attributes().is_empty());
        index += 1;
    }
};

fn path(name: &str) -> &str {
    name.strip_prefix(NAMESPACE).unwrap_or(name)
}

impl EntityKind {
    pub fn from_id(id: EntityTypeId) -> Option<Self> {
        Self::ALL.get(usize::from(id.0)).copied()
    }

    pub fn from_name(name: &str) -> Option<Self> {
        let path = path(name);
        NAMES
            .binary_search_by_key(&path, |&(candidate, _)| candidate)
            .ok()
            .map(|index| NAMES[index].1)
    }

    pub fn from_spawn_egg(item: u16) -> Option<Self> {
        SPAWN_EGGS
            .binary_search_by_key(&item, |&(egg, _)| egg)
            .ok()
            .map(|index| SPAWN_EGGS[index].1)
    }

    pub fn default_attribute(self, name: &str) -> Option<f64> {
        let wanted = path(name);
        self.default_attributes()
            .iter()
            .find(|&&(candidate, _)| path(candidate) == wanted)
            .map(|&(_, value)| value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_table_is_sorted_and_complete() {
        assert_eq!(NAMES.len(), EntityKind::ALL.len());
        assert!(NAMES.windows(2).all(|pair| pair[0].0 < pair[1].0));
        for (path, kind) in NAMES {
            assert_eq!(kind.name().strip_prefix(NAMESPACE), Some(path));
        }
    }

    #[test]
    fn spawn_egg_table_points_back_to_its_kinds() {
        assert!(SPAWN_EGGS.windows(2).all(|pair| pair[0].0 < pair[1].0));
        for (item, kind) in SPAWN_EGGS {
            assert_eq!(kind.spawn_egg(), Some(item));
        }
        let with_egg = EntityKind::ALL
            .iter()
            .filter(|kind| kind.spawn_egg().is_some())
            .count();
        assert_eq!(with_egg, SPAWN_EGGS.len());
    }

    #[test]
    fn attribute_names_are_namespaced_and_unique() {
        for kind in EntityKind::ALL {
            let attributes = kind.default_attributes();
            for (index, (name, _)) in attributes.iter().enumerate() {
                assert!(name.starts_with(NAMESPACE));
                assert!(
                    attributes[index + 1..]
                        .iter()
                        .all(|(other, _)| other != name)
                );
            }
        }
    }
}
