use crate::ids::play::clientbound as id;
use crate::play::wire::{Var, wire_struct};

pub const FLAG_AMBIENT: u8 = 0x01;
pub const FLAG_VISIBLE: u8 = 0x02;
pub const FLAG_SHOW_ICON: u8 = 0x04;
pub const FLAG_BLEND: u8 = 0x08;

wire_struct! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct UpdateMobEffect = id::UPDATE_MOB_EFFECT {
        pub entity_id: i32 as Var,
        pub effect: i32 as Var,
        pub amplifier: i32 as Var,
        pub duration: i32 as Var,
        pub flags: u8,
    }
}

wire_struct! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct RemoveMobEffect = id::REMOVE_MOB_EFFECT {
        pub entity_id: i32 as Var,
        pub effect: i32 as Var,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Packet;
    use crate::play::wire::testing::roundtrip;

    #[test]
    fn update_mob_effect_matches_vanilla_layout() {
        let packet = UpdateMobEffect {
            entity_id: 1,
            effect: 0,
            amplifier: 250,
            duration: 1_999_980,
            flags: FLAG_VISIBLE | FLAG_SHOW_ICON | FLAG_BLEND,
        };
        assert_eq!(
            roundtrip(&packet),
            vec![0x01, 0x00, 0xFA, 0x01, 0xEC, 0x88, 0x7A, 0x0E]
        );
        assert_eq!(<UpdateMobEffect as Packet>::ID, 135);
    }

    #[test]
    fn infinite_duration_is_minus_one() {
        let packet = UpdateMobEffect {
            entity_id: 2,
            effect: 1,
            amplifier: 0,
            duration: -1,
            flags: FLAG_AMBIENT,
        };
        assert_eq!(
            roundtrip(&packet),
            vec![0x02, 0x01, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0x0F, 0x01]
        );
    }

    #[test]
    fn remove_mob_effect_layout() {
        assert_eq!(
            roundtrip(&RemoveMobEffect {
                entity_id: 3,
                effect: 39
            }),
            vec![0x03, 0x27]
        );
        assert_eq!(<RemoveMobEffect as Packet>::ID, 79);
    }
}
