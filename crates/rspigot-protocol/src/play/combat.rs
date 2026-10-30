use crate::Packet;
use crate::codec::{Decode, DecodeError, Encode};
use crate::ids::play::clientbound as id;
use crate::play::types::Vec3;
use crate::play::wire::{FieldCodec, Var, read_var, wire_struct, write_var};
use crate::types::Text;

pub struct OptionalEntityId;

impl FieldCodec<Option<i32>> for OptionalEntityId {
    fn write(value: &Option<i32>, out: &mut Vec<u8>) {
        write_var(value.map_or(0, |id| id.wrapping_add(1)), out);
    }

    fn read(input: &mut &[u8]) -> Result<Option<i32>, DecodeError> {
        let raw = read_var(input)?;
        Ok((raw != 0).then(|| raw.wrapping_sub(1)))
    }
}

wire_struct! {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct DamageEvent = id::DAMAGE_EVENT {
        pub entity_id: i32 as Var,
        pub source_type: i32 as Var,
        pub source_cause_id: Option<i32> as OptionalEntityId,
        pub source_direct_id: Option<i32> as OptionalEntityId,
        pub source_position: Option<Vec3>,
    }
}

wire_struct! {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct HurtAnimation = id::HURT_ANIMATION {
        pub entity_id: i32 as Var,
        pub yaw: f32,
    }
}

wire_struct! {
    #[derive(Debug, Clone, PartialEq)]
    pub struct PlayerCombatKill = id::PLAYER_COMBAT_KILL {
        pub player_id: i32 as Var,
        pub message: Text,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityEvent {
    pub entity_id: i32,
    pub event: i8,
}

impl EntityEvent {
    pub const DEATH: i8 = 3;
    pub const POOF: i8 = 60;
    pub const OPERATOR_LEVEL_BASE: i8 = 24;
}

impl Encode for EntityEvent {
    fn encode(&self, out: &mut Vec<u8>) {
        self.entity_id.encode(out);
        self.event.encode(out);
    }
}

impl Decode for EntityEvent {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            entity_id: i32::decode(input)?,
            event: i8::decode(input)?,
        })
    }
}

impl Packet for EntityEvent {
    const ID: i32 = id::ENTITY_EVENT;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::play::wire::testing::roundtrip;
    use rspigot_nbt::Tag;

    #[test]
    fn damage_event_without_sources_matches_vanilla_layout() {
        let packet = DamageEvent {
            entity_id: 1,
            source_type: 10,
            source_cause_id: None,
            source_direct_id: None,
            source_position: None,
        };
        assert_eq!(roundtrip(&packet), vec![0x01, 0x0A, 0x00, 0x00, 0x00]);
        assert_eq!(<DamageEvent as Packet>::ID, 25);
    }

    #[test]
    fn damage_event_offsets_entity_ids_and_writes_the_position() {
        let packet = DamageEvent {
            entity_id: 300,
            source_type: 34,
            source_cause_id: Some(7),
            source_direct_id: Some(0),
            source_position: Some(Vec3::new(1.0, -2.0, 0.5)),
        };
        let mut expected = vec![0xAC, 0x02, 0x22, 0x08, 0x01, 0x01];
        expected.extend_from_slice(&1.0f64.to_be_bytes());
        expected.extend_from_slice(&(-2.0f64).to_be_bytes());
        expected.extend_from_slice(&0.5f64.to_be_bytes());
        assert_eq!(roundtrip(&packet), expected);
    }

    #[test]
    fn hurt_animation_layout() {
        let packet = HurtAnimation {
            entity_id: 5,
            yaw: 90.0,
        };
        assert_eq!(roundtrip(&packet), vec![0x05, 0x42, 0xB4, 0x00, 0x00]);
        assert_eq!(<HurtAnimation as Packet>::ID, 43);
    }

    #[test]
    fn combat_kill_layout() {
        let packet = PlayerCombatKill {
            player_id: 2,
            message: Text(Tag::String("x".into())),
        };
        assert_eq!(roundtrip(&packet), vec![0x02, 0x08, 0x00, 0x01, b'x']);
        assert_eq!(<PlayerCombatKill as Packet>::ID, 69);
    }

    #[test]
    fn entity_event_writes_a_plain_int_and_a_byte() {
        let packet = EntityEvent {
            entity_id: 258,
            event: EntityEvent::DEATH,
        };
        assert_eq!(roundtrip(&packet), vec![0x00, 0x00, 0x01, 0x02, 0x03]);
        assert_eq!(<EntityEvent as Packet>::ID, 34);
    }
}
