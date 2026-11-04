use rspigot_blocks::BlockKind;
use rspigot_domain::campfire::{Campfire, SLOTS};
use rspigot_domain::chunk::BlockEntity;
use rspigot_domain::item::ItemStack;
use rspigot_domain::player::{GameMode, PlayerId};
use rspigot_domain::{BlockPos, BlockStateId, Identifier};
use rspigot_nbt::{Compound, Tag};

use crate::containers::{read_items, write_items};
use crate::events::{Animation, ClientEvent, Hand};
use crate::ports::ClientLink;
use crate::server::{Audience, Server};

const ENTITY: &str = "campfire";
const COOKING_TIMES: &str = "CookingTimes";
const COOKING_TOTAL_TIMES: &str = "CookingTotalTimes";
const LIT: &str = "lit";

pub(crate) fn is_campfire(state: BlockStateId) -> bool {
    matches!(
        state.kind(),
        Some(BlockKind::Campfire | BlockKind::SoulCampfire)
    )
}

fn is_lit(state: BlockStateId) -> bool {
    state.property(LIT) == Some("true")
}

fn read_counters(data: &Compound, key: &str) -> [i32; SLOTS] {
    let mut counters = [0; SLOTS];
    if let Some(values) = data.get_int_array(key) {
        for (counter, value) in counters.iter_mut().zip(values) {
            *counter = *value;
        }
    }
    counters
}

pub(crate) fn read_campfire(data: &Compound) -> Campfire {
    let mut items = read_items(data, SLOTS).into_iter();
    Campfire {
        items: std::array::from_fn(|_| items.next().flatten()),
        progress: read_counters(data, COOKING_TIMES),
        total: read_counters(data, COOKING_TOTAL_TIMES),
    }
}

pub(crate) fn write_campfire(campfire: &Campfire, data: &mut Compound) {
    write_items(data, &campfire.items);
    data.insert(COOKING_TIMES, Tag::IntArray(campfire.progress.to_vec()));
    data.insert(COOKING_TOTAL_TIMES, Tag::IntArray(campfire.total.to_vec()));
}

fn update_tag(campfire: &Campfire) -> Compound {
    let mut tag = Compound::new();
    write_items(&mut tag, &campfire.items);
    tag
}

impl<L: ClientLink> Server<L> {
    fn campfire_at(&self, pos: BlockPos) -> Option<Campfire> {
        self.world
            .chunks
            .get(pos.chunk())
            .and_then(|chunk| chunk.block_entity(pos))
            .map(|entity| read_campfire(&entity.data))
    }

    fn store_campfire(&mut self, pos: BlockPos, campfire: &Campfire) {
        let Some(chunk) = self.world.chunks.chunk_mut(pos.chunk()) else {
            return;
        };
        let (kind, mut data) = chunk.block_entity(pos).map_or_else(
            || (Identifier::minecraft(ENTITY), Compound::new()),
            |existing| (existing.kind.clone(), existing.data.clone()),
        );
        write_campfire(campfire, &mut data);
        chunk.set_block_entity(BlockEntity { pos, kind, data });
    }

    fn send_campfire_update(&mut self, pos: BlockPos, campfire: &Campfire) {
        self.push(
            Audience::ChunkViewers(pos.chunk()),
            ClientEvent::BlockEntityUpdate {
                pos,
                kind: ENTITY,
                data: update_tag(campfire),
            },
        );
    }

    pub(crate) fn has_block_entity(&self, pos: BlockPos) -> bool {
        self.world
            .chunks
            .get(pos.chunk())
            .is_some_and(|chunk| chunk.block_entity(pos).is_some())
    }

    pub(crate) fn block_entity_update(&self, pos: BlockPos) -> Option<ClientEvent> {
        let entity = self.world.chunks.get(pos.chunk())?.block_entity(pos)?;
        if entity.kind.as_str() != "minecraft:campfire" {
            return None;
        }
        let campfire = read_campfire(&entity.data);
        Some(ClientEvent::BlockEntityUpdate {
            pos,
            kind: ENTITY,
            data: update_tag(&campfire),
        })
    }

    pub(crate) fn campfire_items_lost_by(
        &self,
        pos: BlockPos,
        old: BlockStateId,
        new: BlockStateId,
    ) -> Vec<ItemStack> {
        if old.kind() == new.kind() || !is_campfire(old) {
            return Vec::new();
        }
        self.campfire_at(pos)
            .map(|campfire| campfire.items.into_iter().flatten().collect())
            .unwrap_or_default()
    }

    pub(crate) fn tick_campfires(&mut self) {
        let mut found: Vec<BlockPos> = Vec::new();
        for chunk_pos in self.world.chunks.loaded_positions() {
            let Some(chunk) = self.world.chunks.get(chunk_pos) else {
                continue;
            };
            found.extend(
                chunk
                    .block_entities
                    .iter()
                    .filter(|entity| entity.kind.as_str() == "minecraft:campfire")
                    .map(|entity| entity.pos),
            );
        }
        for pos in found {
            self.tick_campfire(pos);
        }
    }

    fn tick_campfire(&mut self, pos: BlockPos) {
        let Some(state) = self.world.chunks.block(pos).filter(|s| is_campfire(*s)) else {
            return;
        };
        let Some(mut campfire) = self.campfire_at(pos) else {
            return;
        };
        let before = campfire.clone();
        let done = if is_lit(state) {
            campfire.cook()
        } else {
            campfire.cool();
            Vec::new()
        };
        if campfire == before {
            return;
        }
        self.store_campfire(pos, &campfire);
        if campfire.items != before.items {
            self.send_campfire_update(pos, &campfire);
        }
        self.drop_contents(pos, done);
    }

    pub(crate) fn place_food_on_campfire(
        &mut self,
        id: PlayerId,
        hand: Hand,
        slot: usize,
        pos: BlockPos,
        clicked: BlockStateId,
    ) -> bool {
        if !is_campfire(clicked) {
            return false;
        }
        let Some(player) = self.players.get(&id) else {
            return false;
        };
        let mode = player.game_mode;
        let entity = player.entity_id;
        let Some(held) = player
            .inventory
            .get(slot)
            .filter(|stack| Campfire::can_cook(stack))
            .cloned()
        else {
            return false;
        };
        if mode == GameMode::Spectator {
            return false;
        }
        self.ensure_container_entity(pos, clicked);
        let Some(mut campfire) = self.campfire_at(pos) else {
            return true;
        };
        if !campfire.place(&held) {
            return true;
        }
        self.store_campfire(pos, &campfire);
        self.send_campfire_update(pos, &campfire);
        if mode != GameMode::Creative {
            self.consume_one(id, slot, held);
        }
        let swing = ClientEvent::EntityAnimation {
            entity,
            animation: match hand {
                Hand::Main => Animation::SwingMainArm,
                Hand::Off => Animation::SwingOffHand,
            },
        };
        self.push(Audience::One(id), swing.clone());
        self.push(Audience::Trackers(entity), swing);
        true
    }
}
