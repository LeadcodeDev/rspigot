use crate::MobCategory;

macro_rules! mob_categories {
    (
        count: $count:literal,
        categories: [$(
            $category:ident {
                name: $name:literal,
                max_instances_per_chunk: $max:expr,
                friendly: $friendly:literal,
                persistent: $persistent:literal,
                despawn_distance: $despawn:literal,
                no_despawn_distance: $no_despawn:literal $(,)?
            }
        ),* $(,)?] $(,)?
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum MobCategory {
            $($category,)*
        }

        impl MobCategory {
            pub const ALL: [Self; $count] = [$(Self::$category,)*];

            pub const fn name(self) -> &'static str {
                match self {
                    $(Self::$category => $name,)*
                }
            }

            pub const fn max_instances_per_chunk(self) -> Option<u16> {
                match self {
                    $(Self::$category => $max,)*
                }
            }

            pub const fn is_friendly(self) -> bool {
                match self {
                    $(Self::$category => $friendly,)*
                }
            }

            pub const fn is_persistent(self) -> bool {
                match self {
                    $(Self::$category => $persistent,)*
                }
            }

            pub const fn despawn_distance(self) -> u16 {
                match self {
                    $(Self::$category => $despawn,)*
                }
            }

            pub const fn no_despawn_distance(self) -> u16 {
                match self {
                    $(Self::$category => $no_despawn,)*
                }
            }
        }
    };
}

pub(crate) use mob_categories;

impl MobCategory {
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|category| category.name() == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_roundtrip() {
        for category in MobCategory::ALL {
            assert_eq!(MobCategory::from_name(category.name()), Some(category));
        }
        assert_eq!(MobCategory::from_name("nope"), None);
    }
}
