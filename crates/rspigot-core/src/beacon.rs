use rspigot_blocks::BlockKind;
use rspigot_domain::chunk::BlockEntity;
use rspigot_domain::effect::EffectId;
use rspigot_domain::entity::physics::has_tag;
use rspigot_domain::player::PlayerId;
use rspigot_domain::{BlockPos, BlockStateId, Identifier};
use rspigot_nbt::Compound;

use crate::ports::ClientLink;
use crate::server::Server;

const LEVELS: &str = "Levels";
const PRIMARY: &str = "primary_effect";
const SECONDARY: &str = "secondary_effect";
const BASE_BLOCKS: &str = "minecraft:beacon_base_blocks";
const MAX_LEVELS: i32 = 4;
const SECONDARY_LEVELS: i32 = 4;
const RECHECK_TICKS: i64 = 80;
const LEVELS_DATA: i16 = 0;
const PRIMARY_DATA: i16 = 1;
const SECONDARY_DATA: i16 = 2;
const TIERS: [&[&str]; 4] = [
    &["minecraft:speed", "minecraft:haste"],
    &["minecraft:resistance", "minecraft:jump_boost"],
    &["minecraft:strength"],
    &["minecraft:regeneration"],
];

pub(crate) fn is_beacon(state: BlockStateId) -> bool {
    state.kind() == Some(BlockKind::Beacon)
}

fn required_levels(effect: Option<&str>) -> i32 {
    let Some(effect) = effect else {
        return 0;
    };
    TIERS
        .iter()
        .position(|tier| tier.contains(&effect))
        .map_or(i32::MAX, |index| index as i32 + 1)
}

fn valid_choice(primary: Option<&str>, secondary: Option<&str>, levels: i32) -> bool {
    if secondary.is_some() && levels < SECONDARY_LEVELS {
        return false;
    }
    let needed_primary = required_levels(primary);
    let needed_secondary = required_levels(secondary);
    if needed_primary > levels || needed_secondary > levels || needed_primary >= MAX_LEVELS {
        return false;
    }
    needed_secondary == 0 || needed_secondary >= MAX_LEVELS || primary == secondary
}

fn encode(effect: Option<&str>) -> i16 {
    effect
        .and_then(EffectId::from_name)
        .and_then(|id| i16::try_from(id.0).ok())
        .map_or(0, |id| id + 1)
}

fn name_of(id: i32) -> Option<&'static str> {
    u16::try_from(id).ok().and_then(|id| EffectId(id).name())
}

impl<L: ClientLink> Server<L> {
    fn beacon_data(&self, pos: BlockPos) -> Option<&Compound> {
        self.world
            .chunks
            .get(pos.chunk())
            .and_then(|chunk| chunk.block_entity(pos))
            .map(|entity| &entity.data)
    }

    fn beacon_effect(&self, pos: BlockPos, key: &str) -> Option<&str> {
        self.beacon_data(pos)
            .and_then(|data| data.get_str(key))
            .filter(|name| required_levels(Some(name)) != i32::MAX)
    }

    fn pyramid_levels(&self, pos: BlockPos) -> i32 {
        let floor = self.world.dimension.height.min_y;
        let mut levels = 0;
        for step in 1..=MAX_LEVELS {
            let y = pos.y - step;
            if y < floor {
                break;
            }
            let complete = (-step..=step).all(|dx| {
                (-step..=step).all(|dz| {
                    self.world
                        .chunks
                        .block(BlockPos::new(pos.x + dx, y, pos.z + dz))
                        .is_some_and(|state| has_tag(state, BASE_BLOCKS))
                })
            });
            if !complete {
                break;
            }
            levels = step;
        }
        levels
    }

    fn store_beacon(
        &mut self,
        pos: BlockPos,
        levels: i32,
        primary: Option<&str>,
        secondary: Option<&str>,
    ) {
        let Some(chunk) = self.world.chunks.chunk_mut(pos.chunk()) else {
            return;
        };
        let mut data = chunk
            .block_entity(pos)
            .map_or_else(Compound::new, |existing| existing.data.clone());
        data.insert(LEVELS, levels);
        for (key, effect) in [(PRIMARY, primary), (SECONDARY, secondary)] {
            match effect {
                Some(name) => {
                    data.insert(key, name);
                }
                None => {
                    data.remove(key);
                }
            }
        }
        chunk.set_block_entity(BlockEntity {
            pos,
            kind: Identifier::minecraft("beacon"),
            data,
        });
    }

    pub(crate) fn beacon_menu_data(&mut self, pos: BlockPos) -> Vec<i16> {
        let levels = self.pyramid_levels(pos);
        let primary = self.beacon_effect(pos, PRIMARY).map(str::to_owned);
        let secondary = self.beacon_effect(pos, SECONDARY).map(str::to_owned);
        self.store_beacon(pos, levels, primary.as_deref(), secondary.as_deref());
        vec![
            i16::try_from(levels).unwrap_or(0),
            encode(primary.as_deref()),
            encode(secondary.as_deref()),
        ]
    }

    pub(crate) fn recheck_beacon_levels(&mut self) {
        if self.world.level.game_time % RECHECK_TICKS != 0 {
            return;
        }
        let beacons: Vec<BlockPos> = self
            .players
            .values()
            .filter_map(|player| player.open_menu.as_ref())
            .filter(|open| open.backing.block == BlockKind::Beacon)
            .map(|open| open.backing.pos)
            .collect();
        for pos in beacons {
            let levels = self.pyramid_levels(pos);
            let stored = self
                .beacon_data(pos)
                .and_then(|data| data.get_i32(LEVELS))
                .unwrap_or(0);
            if levels != stored {
                let primary = self.beacon_effect(pos, PRIMARY).map(str::to_owned);
                let secondary = self.beacon_effect(pos, SECONDARY).map(str::to_owned);
                self.store_beacon(pos, levels, primary.as_deref(), secondary.as_deref());
                self.send_menu_data(pos, LEVELS_DATA, i16::try_from(levels).unwrap_or(0));
            }
        }
    }

    pub(crate) fn handle_set_beacon(
        &mut self,
        id: PlayerId,
        primary: Option<i32>,
        secondary: Option<i32>,
    ) {
        let Some(open) = self
            .players
            .get(&id)
            .and_then(|player| player.open_menu.as_ref())
            .filter(|open| open.backing.block == BlockKind::Beacon)
        else {
            return;
        };
        let pos = open.backing.pos;
        let paid = open.local.first().is_some_and(Option::is_some);
        let chosen_primary = primary.map(name_of);
        let chosen_secondary = secondary.map(name_of);
        if !paid || chosen_primary == Some(None) || chosen_secondary == Some(None) {
            return;
        }
        let (primary, secondary) = (chosen_primary.flatten(), chosen_secondary.flatten());
        let levels = self.pyramid_levels(pos);
        if !valid_choice(primary, secondary, levels) {
            return;
        }
        self.store_beacon(pos, levels, primary, secondary);
        self.use_up_payment(id);
        self.send_menu_data(pos, PRIMARY_DATA, encode(primary));
        self.send_menu_data(pos, SECONDARY_DATA, encode(secondary));
    }
}
