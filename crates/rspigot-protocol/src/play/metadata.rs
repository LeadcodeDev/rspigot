use uuid::Uuid;

use crate::Packet;
use crate::codec::{Decode, DecodeError, Encode, Position};
use crate::ids::play::clientbound as id;
use crate::play::slot::ItemStack;
use crate::play::types::{Direction, GlobalPos, Pose};
use crate::play::wire::{read_var, write_var};
use crate::types::Text;

pub const END_OF_DATA: u8 = 0xFF;

pub mod serializer {
    pub const BYTE: i32 = 0;
    pub const INT: i32 = 1;
    pub const LONG: i32 = 2;
    pub const FLOAT: i32 = 3;
    pub const STRING: i32 = 4;
    pub const COMPONENT: i32 = 5;
    pub const OPTIONAL_COMPONENT: i32 = 6;
    pub const ITEM_STACK: i32 = 7;
    pub const BOOLEAN: i32 = 8;
    pub const ROTATIONS: i32 = 9;
    pub const BLOCK_POS: i32 = 10;
    pub const OPTIONAL_BLOCK_POS: i32 = 11;
    pub const DIRECTION: i32 = 12;
    pub const OPTIONAL_LIVING_ENTITY_REFERENCE: i32 = 13;
    pub const BLOCK_STATE: i32 = 14;
    pub const OPTIONAL_BLOCK_STATE: i32 = 15;
    pub const PARTICLE: i32 = 16;
    pub const PARTICLES: i32 = 17;
    pub const VILLAGER_DATA: i32 = 18;
    pub const OPTIONAL_UNSIGNED_INT: i32 = 19;
    pub const POSE: i32 = 20;
    pub const CAT_VARIANT: i32 = 21;
    pub const CAT_SOUND_VARIANT: i32 = 22;
    pub const COW_VARIANT: i32 = 23;
    pub const COW_SOUND_VARIANT: i32 = 24;
    pub const WOLF_VARIANT: i32 = 25;
    pub const WOLF_SOUND_VARIANT: i32 = 26;
    pub const FROG_VARIANT: i32 = 27;
    pub const PIG_VARIANT: i32 = 28;
    pub const PIG_SOUND_VARIANT: i32 = 29;
    pub const CHICKEN_VARIANT: i32 = 30;
    pub const CHICKEN_SOUND_VARIANT: i32 = 31;
    pub const ZOMBIE_NAUTILUS_VARIANT: i32 = 32;
    pub const OPTIONAL_GLOBAL_POS: i32 = 33;
    pub const PAINTING_VARIANT: i32 = 34;
    pub const SNIFFER_STATE: i32 = 35;
    pub const ARMADILLO_STATE: i32 = 36;
    pub const COPPER_GOLEM_STATE: i32 = 37;
    pub const WEATHERING_COPPER_STATE: i32 = 38;
    pub const VECTOR3: i32 = 39;
    pub const QUATERNION: i32 = 40;
    pub const RESOLVABLE_PROFILE: i32 = 41;
    pub const HUMANOID_ARM: i32 = 42;
    pub const DYE_COLOR: i32 = 43;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegistryVariant {
    Cat,
    CatSound,
    Cow,
    CowSound,
    Wolf,
    WolfSound,
    Frog,
    Pig,
    PigSound,
    Chicken,
    ChickenSound,
    ZombieNautilus,
}

impl RegistryVariant {
    pub const fn serializer(self) -> i32 {
        match self {
            Self::Cat => serializer::CAT_VARIANT,
            Self::CatSound => serializer::CAT_SOUND_VARIANT,
            Self::Cow => serializer::COW_VARIANT,
            Self::CowSound => serializer::COW_SOUND_VARIANT,
            Self::Wolf => serializer::WOLF_VARIANT,
            Self::WolfSound => serializer::WOLF_SOUND_VARIANT,
            Self::Frog => serializer::FROG_VARIANT,
            Self::Pig => serializer::PIG_VARIANT,
            Self::PigSound => serializer::PIG_SOUND_VARIANT,
            Self::Chicken => serializer::CHICKEN_VARIANT,
            Self::ChickenSound => serializer::CHICKEN_SOUND_VARIANT,
            Self::ZombieNautilus => serializer::ZOMBIE_NAUTILUS_VARIANT,
        }
    }

    pub const fn from_serializer(id: i32) -> Option<Self> {
        Some(match id {
            serializer::CAT_VARIANT => Self::Cat,
            serializer::CAT_SOUND_VARIANT => Self::CatSound,
            serializer::COW_VARIANT => Self::Cow,
            serializer::COW_SOUND_VARIANT => Self::CowSound,
            serializer::WOLF_VARIANT => Self::Wolf,
            serializer::WOLF_SOUND_VARIANT => Self::WolfSound,
            serializer::FROG_VARIANT => Self::Frog,
            serializer::PIG_VARIANT => Self::Pig,
            serializer::PIG_SOUND_VARIANT => Self::PigSound,
            serializer::CHICKEN_VARIANT => Self::Chicken,
            serializer::CHICKEN_SOUND_VARIANT => Self::ChickenSound,
            serializer::ZOMBIE_NAUTILUS_VARIANT => Self::ZombieNautilus,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnumState {
    Sniffer,
    Armadillo,
    CopperGolem,
    WeatheringCopper,
    HumanoidArm,
    DyeColor,
}

impl EnumState {
    pub const fn serializer(self) -> i32 {
        match self {
            Self::Sniffer => serializer::SNIFFER_STATE,
            Self::Armadillo => serializer::ARMADILLO_STATE,
            Self::CopperGolem => serializer::COPPER_GOLEM_STATE,
            Self::WeatheringCopper => serializer::WEATHERING_COPPER_STATE,
            Self::HumanoidArm => serializer::HUMANOID_ARM,
            Self::DyeColor => serializer::DYE_COLOR,
        }
    }

    pub const fn from_serializer(id: i32) -> Option<Self> {
        Some(match id {
            serializer::SNIFFER_STATE => Self::Sniffer,
            serializer::ARMADILLO_STATE => Self::Armadillo,
            serializer::COPPER_GOLEM_STATE => Self::CopperGolem,
            serializer::WEATHERING_COPPER_STATE => Self::WeatheringCopper,
            serializer::HUMANOID_ARM => Self::HumanoidArm,
            serializer::DYE_COLOR => Self::DyeColor,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EntityDataValue {
    Byte(i8),
    Int(i32),
    Long(i64),
    Float(f32),
    String(String),
    Component(Text),
    OptionalComponent(Option<Text>),
    ItemStack(ItemStack),
    Boolean(bool),
    Rotations {
        x: f32,
        y: f32,
        z: f32,
    },
    BlockPos(Position),
    OptionalBlockPos(Option<Position>),
    Direction(Direction),
    OptionalLivingEntity(Option<Uuid>),
    BlockState(i32),
    OptionalBlockState(Option<i32>),
    VillagerData {
        kind: i32,
        profession: i32,
        level: i32,
    },
    OptionalUnsignedInt(Option<u32>),
    Pose(Pose),
    Variant {
        kind: RegistryVariant,
        id: i32,
    },
    OptionalGlobalPos(Option<GlobalPos>),
    PaintingVariant(i32),
    State {
        kind: EnumState,
        id: i32,
    },
    Vector3([f32; 3]),
    Quaternion([f32; 4]),
    Raw {
        serializer: i32,
        data: Vec<u8>,
    },
}

impl EntityDataValue {
    pub fn serializer(&self) -> i32 {
        match self {
            Self::Byte(_) => serializer::BYTE,
            Self::Int(_) => serializer::INT,
            Self::Long(_) => serializer::LONG,
            Self::Float(_) => serializer::FLOAT,
            Self::String(_) => serializer::STRING,
            Self::Component(_) => serializer::COMPONENT,
            Self::OptionalComponent(_) => serializer::OPTIONAL_COMPONENT,
            Self::ItemStack(_) => serializer::ITEM_STACK,
            Self::Boolean(_) => serializer::BOOLEAN,
            Self::Rotations { .. } => serializer::ROTATIONS,
            Self::BlockPos(_) => serializer::BLOCK_POS,
            Self::OptionalBlockPos(_) => serializer::OPTIONAL_BLOCK_POS,
            Self::Direction(_) => serializer::DIRECTION,
            Self::OptionalLivingEntity(_) => serializer::OPTIONAL_LIVING_ENTITY_REFERENCE,
            Self::BlockState(_) => serializer::BLOCK_STATE,
            Self::OptionalBlockState(_) => serializer::OPTIONAL_BLOCK_STATE,
            Self::VillagerData { .. } => serializer::VILLAGER_DATA,
            Self::OptionalUnsignedInt(_) => serializer::OPTIONAL_UNSIGNED_INT,
            Self::Pose(_) => serializer::POSE,
            Self::Variant { kind, .. } => kind.serializer(),
            Self::OptionalGlobalPos(_) => serializer::OPTIONAL_GLOBAL_POS,
            Self::PaintingVariant(_) => serializer::PAINTING_VARIANT,
            Self::State { kind, .. } => kind.serializer(),
            Self::Vector3(_) => serializer::VECTOR3,
            Self::Quaternion(_) => serializer::QUATERNION,
            Self::Raw { serializer, .. } => *serializer,
        }
    }

    fn write_value(&self, out: &mut Vec<u8>) {
        match self {
            Self::Byte(v) => v.encode(out),
            Self::Int(v) => write_var(*v, out),
            Self::Long(v) => crate::codec::VarLong(*v).encode(out),
            Self::Float(v) => v.encode(out),
            Self::String(v) => v.encode(out),
            Self::Component(v) => v.encode(out),
            Self::OptionalComponent(v) => v.encode(out),
            Self::ItemStack(v) => v.encode(out),
            Self::Boolean(v) => v.encode(out),
            Self::Rotations { x, y, z } => {
                x.encode(out);
                y.encode(out);
                z.encode(out);
            }
            Self::BlockPos(v) => v.encode(out),
            Self::OptionalBlockPos(v) => v.encode(out),
            Self::Direction(v) => v.encode(out),
            Self::OptionalLivingEntity(v) => v.encode(out),
            Self::BlockState(v) => write_var(*v, out),
            Self::OptionalBlockState(v) => write_var(v.unwrap_or(0), out),
            Self::VillagerData {
                kind,
                profession,
                level,
            } => {
                write_var(*kind, out);
                write_var(*profession, out);
                write_var(*level, out);
            }
            Self::OptionalUnsignedInt(v) => {
                write_var(v.map_or(0, |v| (v as i32).wrapping_add(1)), out)
            }
            Self::Pose(v) => v.encode(out),
            Self::Variant { id, .. } | Self::State { id, .. } => write_var(*id, out),
            Self::OptionalGlobalPos(v) => v.encode(out),
            Self::PaintingVariant(v) => write_var(v.wrapping_add(1), out),
            Self::Vector3(v) => v.iter().for_each(|c| c.encode(out)),
            Self::Quaternion(v) => v.iter().for_each(|c| c.encode(out)),
            Self::Raw { data, .. } => out.extend_from_slice(data),
        }
    }

    fn read_value(serializer: i32, input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(match serializer {
            serializer::BYTE => Self::Byte(i8::decode(input)?),
            serializer::INT => Self::Int(read_var(input)?),
            serializer::LONG => Self::Long(crate::codec::VarLong::decode(input)?.0),
            serializer::FLOAT => Self::Float(f32::decode(input)?),
            serializer::STRING => Self::String(String::decode(input)?),
            serializer::COMPONENT => Self::Component(Text::decode(input)?),
            serializer::OPTIONAL_COMPONENT => Self::OptionalComponent(Option::decode(input)?),
            serializer::ITEM_STACK => Self::ItemStack(ItemStack::decode(input)?),
            serializer::BOOLEAN => Self::Boolean(bool::decode(input)?),
            serializer::ROTATIONS => Self::Rotations {
                x: f32::decode(input)?,
                y: f32::decode(input)?,
                z: f32::decode(input)?,
            },
            serializer::BLOCK_POS => Self::BlockPos(Position::decode(input)?),
            serializer::OPTIONAL_BLOCK_POS => Self::OptionalBlockPos(Option::decode(input)?),
            serializer::DIRECTION => Self::Direction(Direction::decode(input)?),
            serializer::OPTIONAL_LIVING_ENTITY_REFERENCE => {
                Self::OptionalLivingEntity(Option::decode(input)?)
            }
            serializer::BLOCK_STATE => Self::BlockState(read_var(input)?),
            serializer::OPTIONAL_BLOCK_STATE => Self::OptionalBlockState(match read_var(input)? {
                0 => None,
                state => Some(state),
            }),
            serializer::VILLAGER_DATA => Self::VillagerData {
                kind: read_var(input)?,
                profession: read_var(input)?,
                level: read_var(input)?,
            },
            serializer::OPTIONAL_UNSIGNED_INT => {
                Self::OptionalUnsignedInt(match read_var(input)? {
                    0 => None,
                    v => Some(v.wrapping_sub(1) as u32),
                })
            }
            serializer::POSE => Self::Pose(Pose::decode(input)?),
            serializer::OPTIONAL_GLOBAL_POS => Self::OptionalGlobalPos(Option::decode(input)?),
            serializer::PAINTING_VARIANT => match read_var(input)? {
                0 => {
                    return Err(DecodeError::InvalidValue {
                        field: "inline painting variant",
                        value: 0,
                    });
                }
                v => Self::PaintingVariant(v.wrapping_sub(1)),
            },
            serializer::VECTOR3 => Self::Vector3([
                f32::decode(input)?,
                f32::decode(input)?,
                f32::decode(input)?,
            ]),
            serializer::QUATERNION => Self::Quaternion([
                f32::decode(input)?,
                f32::decode(input)?,
                f32::decode(input)?,
                f32::decode(input)?,
            ]),
            other => {
                if let Some(kind) = RegistryVariant::from_serializer(other) {
                    Self::Variant {
                        kind,
                        id: read_var(input)?,
                    }
                } else if let Some(kind) = EnumState::from_serializer(other) {
                    Self::State {
                        kind,
                        id: read_var(input)?,
                    }
                } else {
                    return Err(DecodeError::InvalidValue {
                        field: "entity data serializer",
                        value: i64::from(other),
                    });
                }
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntityDataItem {
    pub index: u8,
    pub value: EntityDataValue,
}

impl Encode for EntityDataItem {
    fn encode(&self, out: &mut Vec<u8>) {
        out.push(self.index);
        write_var(self.value.serializer(), out);
        self.value.write_value(out);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SetEntityData {
    pub entity_id: i32,
    pub items: Vec<EntityDataItem>,
}

impl Encode for SetEntityData {
    fn encode(&self, out: &mut Vec<u8>) {
        write_var(self.entity_id, out);
        for item in &self.items {
            item.encode(out);
        }
        out.push(END_OF_DATA);
    }
}

impl Decode for SetEntityData {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        let entity_id = read_var(input)?;
        let mut items = Vec::new();
        loop {
            let index = u8::decode(input)?;
            if index == END_OF_DATA {
                break;
            }
            let serializer = read_var(input)?;
            items.push(EntityDataItem {
                index,
                value: EntityDataValue::read_value(serializer, input)?,
            });
        }
        Ok(Self { entity_id, items })
    }
}

impl Packet for SetEntityData {
    const ID: i32 = id::SET_ENTITY_DATA;
}

#[cfg(test)]
mod tests {
    use rspigot_nbt::Tag;

    use super::*;
    use crate::codec::decode_exact;
    use crate::play::wire::testing::{encoded, roundtrip};

    fn item(index: u8, value: EntityDataValue) -> EntityDataItem {
        EntityDataItem { index, value }
    }

    #[test]
    fn shared_flags_and_pose_golden() {
        let packet = SetEntityData {
            entity_id: 42,
            items: vec![
                item(0, EntityDataValue::Byte(0x02)),
                item(6, EntityDataValue::Pose(Pose::Crouching)),
            ],
        };
        assert_eq!(
            roundtrip(&packet),
            vec![42, 0, 0, 0x02, 6, 20, 5, END_OF_DATA]
        );
    }

    #[test]
    fn every_modelled_value_roundtrips() {
        let values = vec![
            EntityDataValue::Byte(-3),
            EntityDataValue::Int(300),
            EntityDataValue::Long(-5),
            EntityDataValue::Float(1.5),
            EntityDataValue::String("name".into()),
            EntityDataValue::Component(Text(Tag::String("hi".into()))),
            EntityDataValue::OptionalComponent(None),
            EntityDataValue::OptionalComponent(Some(Text(Tag::String("x".into())))),
            EntityDataValue::ItemStack(ItemStack::new(1, 3)),
            EntityDataValue::Boolean(true),
            EntityDataValue::Rotations {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            },
            EntityDataValue::BlockPos(Position { x: 1, y: -2, z: 3 }),
            EntityDataValue::OptionalBlockPos(Some(Position { x: 0, y: 0, z: 0 })),
            EntityDataValue::Direction(Direction::West),
            EntityDataValue::OptionalLivingEntity(Some(Uuid::from_u128(7))),
            EntityDataValue::BlockState(9),
            EntityDataValue::OptionalBlockState(None),
            EntityDataValue::OptionalBlockState(Some(1)),
            EntityDataValue::VillagerData {
                kind: 1,
                profession: 2,
                level: 3,
            },
            EntityDataValue::OptionalUnsignedInt(Some(0)),
            EntityDataValue::Pose(Pose::Sleeping),
            EntityDataValue::Variant {
                kind: RegistryVariant::Wolf,
                id: 4,
            },
            EntityDataValue::OptionalGlobalPos(None),
            EntityDataValue::PaintingVariant(3),
            EntityDataValue::State {
                kind: EnumState::DyeColor,
                id: 14,
            },
            EntityDataValue::Vector3([0.5, 0.5, 0.5]),
            EntityDataValue::Quaternion([0.0, 0.0, 0.0, 1.0]),
        ];
        let packet = SetEntityData {
            entity_id: 1,
            items: values
                .into_iter()
                .enumerate()
                .map(|(i, v)| item(i as u8, v))
                .collect(),
        };
        roundtrip(&packet);
    }

    #[test]
    fn optional_encodings_follow_vanilla_offsets() {
        let bytes = encoded(&item(1, EntityDataValue::OptionalUnsignedInt(Some(4))));
        assert_eq!(bytes, vec![1, serializer::OPTIONAL_UNSIGNED_INT as u8, 5]);
        let bytes = encoded(&item(1, EntityDataValue::OptionalBlockState(Some(9))));
        assert_eq!(bytes, vec![1, serializer::OPTIONAL_BLOCK_STATE as u8, 9]);
        let bytes = encoded(&item(1, EntityDataValue::PaintingVariant(0)));
        assert_eq!(bytes, vec![1, serializer::PAINTING_VARIANT as u8, 1]);
    }

    #[test]
    fn raw_values_encode_but_unknown_serializers_fail_to_decode() {
        let packet = SetEntityData {
            entity_id: 1,
            items: vec![item(
                2,
                EntityDataValue::Raw {
                    serializer: serializer::PARTICLE,
                    data: vec![0x05],
                },
            )],
        };
        let bytes = encoded(&packet);
        assert_eq!(bytes, vec![1, 2, 16, 5, END_OF_DATA]);
        assert!(matches!(
            decode_exact::<SetEntityData>(&bytes),
            Err(DecodeError::InvalidValue { value: 16, .. })
        ));
    }
}
