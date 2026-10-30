use crate::kind::{ITEMS, NAMES, STATE_COUNT};
use crate::{
    Aabb, BlockId, BlockKind, Face, FaceSupport, PerState, Property, SHAPE_COUNT, ShapeId,
    StateFlags, StateId, Support,
};

macro_rules! block_kinds {
    (blocks: $blocks:literal, states: $states:literal, kinds: [$($kind:ident),* $(,)?] $(,)?) => {
        pub const STATE_COUNT: usize = $states;

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[repr(u16)]
        pub enum BlockKind {
            $($kind,)*
        }

        impl BlockKind {
            pub const ALL: [Self; $blocks] = [$(Self::$kind,)*];

            pub const fn name(self) -> &'static str {
                match self {
                    $(Self::$kind => <$crate::blocks::$kind as $crate::Block>::NAME,)*
                }
            }

            pub const fn id(self) -> $crate::BlockId {
                match self {
                    $(Self::$kind => <$crate::blocks::$kind as $crate::Block>::ID,)*
                }
            }

            pub const fn first_state(self) -> $crate::StateId {
                match self {
                    $(Self::$kind => <$crate::blocks::$kind as $crate::Block>::FIRST_STATE,)*
                }
            }

            pub const fn state_count(self) -> u16 {
                match self {
                    $(Self::$kind => <$crate::blocks::$kind as $crate::Block>::STATE_COUNT,)*
                }
            }

            pub const fn default_state(self) -> $crate::StateId {
                match self {
                    $(Self::$kind => <$crate::blocks::$kind as $crate::Block>::DEFAULT_STATE,)*
                }
            }

            pub const fn properties(self) -> &'static [$crate::Property] {
                match self {
                    $(Self::$kind => <$crate::blocks::$kind as $crate::Block>::PROPERTIES,)*
                }
            }

            pub const fn item(self) -> Option<u16> {
                match self {
                    $(Self::$kind => <$crate::blocks::$kind as $crate::Block>::ITEM,)*
                }
            }

            pub fn wall_variant(self) -> Option<Self> {
                let wall = match self {
                    $(Self::$kind => <$crate::blocks::$kind as $crate::Block>::WALL_VARIANT,)*
                };
                wall.and_then(Self::from_id)
            }

            pub const fn settings(self) -> $crate::BlockSettings {
                match self {
                    $(Self::$kind => <$crate::blocks::$kind as $crate::Block>::SETTINGS,)*
                }
            }

            pub const fn spawn_surface(self) -> $crate::SpawnSurface {
                match self {
                    $(Self::$kind => <$crate::blocks::$kind as $crate::Block>::SPAWN_SURFACE,)*
                }
            }

            pub(crate) const fn light_emission_per_state(self) -> $crate::PerState<u8> {
                match self {
                    $(Self::$kind => <$crate::blocks::$kind as $crate::Block>::LIGHT_EMISSION,)*
                }
            }

            pub(crate) const fn light_dampening_per_state(self) -> $crate::PerState<u8> {
                match self {
                    $(Self::$kind => <$crate::blocks::$kind as $crate::Block>::LIGHT_DAMPENING,)*
                }
            }

            pub(crate) const fn flags_per_state(self) -> $crate::PerState<$crate::StateFlags> {
                match self {
                    $(Self::$kind => <$crate::blocks::$kind as $crate::Block>::FLAGS,)*
                }
            }

            pub(crate) const fn collision_per_state(self) -> $crate::PerState<$crate::ShapeId> {
                match self {
                    $(Self::$kind => <$crate::blocks::$kind as $crate::Block>::COLLISION,)*
                }
            }

            pub(crate) const fn face_support_per_state(self) -> $crate::PerState<$crate::FaceSupport> {
                match self {
                    $(Self::$kind => <$crate::blocks::$kind as $crate::Block>::FACE_SUPPORT,)*
                }
            }

            pub fn place(self, context: &$crate::PlaceContext) -> Option<$crate::Placement> {
                match self {
                    $(Self::$kind => <<$crate::blocks::$kind as $crate::Block>::Behavior as $crate::BlockBehavior>::place(context),)*
                }
            }

            pub const fn gravity(self) -> $crate::Gravity {
                match self {
                    $(Self::$kind => <<$crate::blocks::$kind as $crate::Block>::Behavior as $crate::BlockBehavior>::GRAVITY,)*
                }
            }

            pub const fn falls(self) -> bool {
                self.gravity().falls()
            }

            pub fn use_block(self, context: &$crate::UseContext) -> Option<$crate::UseResult> {
                match self {
                    $(Self::$kind => <<$crate::blocks::$kind as $crate::Block>::Behavior as $crate::BlockBehavior>::use_block(context),)*
                }
            }
        }
    };
}

pub(crate) use block_kinds;

#[derive(Clone, Copy)]
struct StateInfo {
    kind: BlockKind,
    flags: StateFlags,
    light_emission: u8,
    light_dampening: u8,
    collision: ShapeId,
    support: FaceSupport,
}

static STATES: [StateInfo; STATE_COUNT] = build_states();

const fn covers<T: Copy>(values: &PerState<T>, count: u16) -> bool {
    match values {
        PerState::Uniform(_) => true,
        PerState::Varies(values) => values.len() == count as usize,
    }
}

const fn combinations(properties: &[Property]) -> usize {
    let mut total = 1;
    let mut index = 0;
    while index < properties.len() {
        total *= properties[index].values.len();
        index += 1;
    }
    total
}

const fn build_states() -> [StateInfo; STATE_COUNT] {
    let mut states = [StateInfo {
        kind: BlockKind::Air,
        flags: StateFlags(0),
        light_emission: 0,
        light_dampening: 0,
        collision: ShapeId::EMPTY,
        support: FaceSupport::NONE,
    }; STATE_COUNT];
    let mut next = 0;
    let mut index = 0;
    while index < BlockKind::ALL.len() {
        let kind = BlockKind::ALL[index];
        let first = kind.first_state().0;
        let count = kind.state_count();
        let default = kind.default_state().0;
        let emission = kind.light_emission_per_state();
        let dampening = kind.light_dampening_per_state();
        let flags = kind.flags_per_state();
        let collision = kind.collision_per_state();
        let support = kind.face_support_per_state();
        assert!(kind.id().0 as usize == index);
        assert!(first as usize == next);
        assert!(count > 0);
        assert!(combinations(kind.properties()) == count as usize);
        assert!(default >= first && default - first < count);
        assert!(covers(&emission, count) && covers(&dampening, count) && covers(&flags, count));
        assert!(covers(&collision, count) && covers(&support, count));
        let mut offset = 0;
        while offset < count {
            let shape = collision.get(offset);
            assert!((shape.0 as usize) < SHAPE_COUNT);
            states[next] = StateInfo {
                kind,
                flags: flags.get(offset),
                light_emission: emission.get(offset),
                light_dampening: dampening.get(offset),
                collision: shape,
                support: support.get(offset),
            };
            next += 1;
            offset += 1;
        }
        index += 1;
    }
    assert!(next == STATE_COUNT);
    states
}

fn stride(properties: &[Property]) -> usize {
    properties
        .iter()
        .map(|property| property.values.len())
        .product()
}

fn value_at(properties: &'static [Property], index: usize, offset: usize) -> &'static str {
    let values = properties[index].values;
    values[offset / stride(&properties[index + 1..]) % values.len()]
}

impl StateId {
    pub const AIR: Self = Self(0);

    fn info(self) -> Option<&'static StateInfo> {
        STATES.get(usize::from(self.0))
    }

    fn locate(self) -> Option<(&'static [Property], usize)> {
        let kind = self.kind()?;
        Some((
            kind.properties(),
            usize::from(self.0 - kind.first_state().0),
        ))
    }

    pub fn kind(self) -> Option<BlockKind> {
        self.info().map(|info| info.kind)
    }

    pub fn is_air(self) -> bool {
        self.flags().contains(StateFlags::AIR)
    }

    pub fn property(self, name: &str) -> Option<&'static str> {
        let (properties, offset) = self.locate()?;
        let index = properties
            .iter()
            .position(|property| property.name == name)?;
        Some(value_at(properties, index, offset))
    }

    pub fn properties(self) -> impl Iterator<Item = (&'static str, &'static str)> {
        let (properties, offset) = self.locate().unwrap_or((&[], 0));
        properties
            .iter()
            .enumerate()
            .map(move |(index, property)| (property.name, value_at(properties, index, offset)))
    }

    pub fn with_property(self, name: &str, value: &str) -> Option<StateId> {
        let (properties, offset) = self.locate()?;
        let index = properties
            .iter()
            .position(|property| property.name == name)?;
        let values = properties[index].values;
        let target = values.iter().position(|candidate| *candidate == value)?;
        let stride = stride(&properties[index + 1..]);
        let current = offset / stride % values.len();
        let id = usize::from(self.0) + target * stride - current * stride;
        u16::try_from(id).ok().map(StateId)
    }

    pub fn light_emission(self) -> u8 {
        self.info().map_or(0, |info| info.light_emission)
    }

    pub fn light_dampening(self) -> u8 {
        self.info().map_or(0, |info| info.light_dampening)
    }

    pub fn flags(self) -> StateFlags {
        self.info().map_or(StateFlags(0), |info| info.flags)
    }

    pub fn collision_shape_id(self) -> ShapeId {
        self.info().map_or(ShapeId::EMPTY, |info| info.collision)
    }

    pub fn collision_shape(self) -> &'static [Aabb] {
        self.collision_shape_id().boxes()
    }

    pub fn face_support(self) -> FaceSupport {
        self.info().map_or(FaceSupport::NONE, |info| info.support)
    }

    pub fn is_face_sturdy(self, face: Face, support: Support) -> bool {
        self.face_support().is_sturdy(face, support)
    }
}

impl BlockKind {
    pub fn from_state(state: StateId) -> Option<Self> {
        state.kind()
    }

    pub fn from_id(id: BlockId) -> Option<Self> {
        Self::ALL.get(usize::from(id.0)).copied()
    }

    pub fn from_name(name: &str) -> Option<Self> {
        let path = name.strip_prefix("minecraft:").unwrap_or(name);
        NAMES
            .binary_search_by_key(&path, |&(candidate, _)| candidate)
            .ok()
            .map(|index| NAMES[index].1)
    }

    pub fn from_item(item: u16) -> Option<Self> {
        ITEMS.get(usize::from(item)).copied().flatten()
    }

    pub fn state_with(self, properties: &[(&str, &str)]) -> Option<StateId> {
        properties
            .iter()
            .try_fold(self.default_state(), |state, &(name, value)| {
                state.with_property(name, value)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_table_is_sorted_and_complete() {
        assert_eq!(NAMES.len(), BlockKind::ALL.len());
        assert!(NAMES.windows(2).all(|pair| pair[0].0 < pair[1].0));
        for (path, kind) in NAMES {
            assert_eq!(kind.name().strip_prefix("minecraft:"), Some(path));
        }
    }

    #[test]
    fn item_table_points_back_to_its_items() {
        for (item, kind) in ITEMS.iter().enumerate() {
            if let Some(kind) = kind {
                assert_eq!(kind.item().map(usize::from), Some(item));
            }
        }
        assert!(ITEMS.last().is_some_and(Option::is_some));
    }

    #[test]
    fn state_table_covers_every_state() {
        assert_eq!(STATES.len(), STATE_COUNT);
        assert_eq!(STATE_COUNT, 35723);
        assert!(StateId(STATE_COUNT as u16).kind().is_none());
    }
}
