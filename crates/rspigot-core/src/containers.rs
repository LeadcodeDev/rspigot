use rand::Rng;
use rspigot_blocks::{BlockKind, ShapeId};
use rspigot_domain::chunk::BlockEntity;
use rspigot_domain::container_menu::{ContainerMenu, MenuKind};
use rspigot_domain::entity::physics::has_tag;
use rspigot_domain::entity::save::{item_stack_from_nbt, item_stack_to_nbt};
use rspigot_domain::item::Slot;
use rspigot_domain::menu::{Clicker, MenuEffect, OUTSIDE, RemoteUpdate};
use rspigot_domain::player::{GameMode, PlayerId};
use rspigot_domain::spawn::placement::is_redstone_conductor;
use rspigot_domain::text::Component;
use rspigot_domain::{BlockPos, BlockStateId, Identifier};
use rspigot_entities::SoundSource;
use rspigot_nbt::{Compound, Tag};

use crate::events::{ClientEvent, ContainerClick};
use crate::ports::ClientLink;
use crate::server::{Audience, Server};

const WINDOWS: i32 = 100;
const MAX_VIEW_DISTANCE_SQUARED: f64 = 64.0;
const ITEMS: &str = "Items";
const BLOCK_EVENT_RANGE: f64 = 64.0;
const LID_ACTION: u8 = 1;
const LID_VOLUME: f32 = 0.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Lid {
    None,
    Chest,
    Shulker,
    Barrel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Backing {
    pub pos: BlockPos,
    pub entity: &'static str,
    pub block: BlockKind,
    pub lid: Lid,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct OpenMenu {
    pub window: i32,
    pub menu: ContainerMenu,
    pub backing: Backing,
    pub state_id: i32,
}

impl OpenMenu {
    fn next_state(&mut self) -> i32 {
        self.state_id = self.state_id.wrapping_add(1) & 0x7FFF;
        self.state_id
    }
}

struct Opening {
    kind: MenuKind,
    entity: &'static str,
    title: &'static str,
    block: BlockKind,
    lid: Lid,
}

fn opening_of(state: BlockStateId) -> Option<Opening> {
    let block = state.kind()?;
    let storage = |kind, entity, title, lid| {
        Some(Opening {
            kind,
            entity,
            title,
            block,
            lid,
        })
    };
    let rows = MenuKind::Generic { rows: 3 };
    match block {
        BlockKind::Chest => storage(rows, "chest", "container.chest", Lid::Chest),
        BlockKind::TrappedChest => storage(rows, "trapped_chest", "container.chest", Lid::Chest),
        BlockKind::Barrel => storage(rows, "barrel", "container.barrel", Lid::Barrel),
        BlockKind::Hopper => storage(MenuKind::Hopper, "hopper", "container.hopper", Lid::None),
        BlockKind::Dispenser => storage(
            MenuKind::Generic3x3,
            "dispenser",
            "container.dispenser",
            Lid::None,
        ),
        BlockKind::Dropper => storage(
            MenuKind::Generic3x3,
            "dropper",
            "container.dropper",
            Lid::None,
        ),
        _ if has_tag(state, "minecraft:shulker_boxes") => storage(
            MenuKind::ShulkerBox,
            "shulker_box",
            "container.shulkerBox",
            Lid::Shulker,
        ),
        _ => None,
    }
}

fn facing_step(state: BlockStateId) -> (i32, i32, i32) {
    match state.property("facing") {
        Some("down") => (0, -1, 0),
        Some("north") => (0, 0, -1),
        Some("south") => (0, 0, 1),
        Some("west") => (-1, 0, 0),
        Some("east") => (1, 0, 0),
        _ => (0, 1, 0),
    }
}

fn lid_sounds(lid: Lid) -> Option<(&'static str, &'static str)> {
    match lid {
        Lid::Chest => Some(("minecraft:block.chest.open", "minecraft:block.chest.close")),
        Lid::Shulker => Some((
            "minecraft:block.shulker_box.open",
            "minecraft:block.shulker_box.close",
        )),
        Lid::Barrel => Some((
            "minecraft:block.barrel.open",
            "minecraft:block.barrel.close",
        )),
        Lid::None => None,
    }
}

pub(crate) fn read_items(data: &Compound, size: usize) -> Vec<Slot> {
    let mut items = vec![None; size];
    for entry in data.get_list(ITEMS).unwrap_or_default() {
        let Some(entry) = entry.as_compound() else {
            continue;
        };
        let slot = entry
            .get_i64("Slot")
            .and_then(|slot| usize::try_from(slot).ok())
            .filter(|slot| *slot < size);
        if let (Some(slot), Some(stack)) = (slot, item_stack_from_nbt(entry)) {
            items[slot] = Some(stack);
        }
    }
    items
}

pub(crate) fn write_items(data: &mut Compound, items: &[Slot]) {
    let list = items
        .iter()
        .enumerate()
        .filter_map(|(slot, stack)| {
            let mut entry = item_stack_to_nbt(stack.as_ref()?);
            entry.insert("Slot", slot as i8);
            Some(Tag::Compound(entry))
        })
        .collect();
    data.insert(ITEMS, Tag::List(list));
}

fn blocked_above(state_above: Option<BlockStateId>) -> bool {
    state_above.is_some_and(is_redstone_conductor)
}

impl<L: ClientLink> Server<L> {
    fn container_items(&self, backing: Backing, size: usize) -> Vec<Slot> {
        self.world
            .chunks
            .get(backing.pos.chunk())
            .and_then(|chunk| chunk.block_entity(backing.pos))
            .map_or_else(|| vec![None; size], |entity| read_items(&entity.data, size))
    }

    fn store_container(&mut self, backing: Backing, items: &[Slot]) {
        let Some(chunk) = self.world.chunks.chunk_mut(backing.pos.chunk()) else {
            return;
        };
        let mut data = chunk
            .block_entity(backing.pos)
            .map_or_else(Compound::new, |entity| entity.data.clone());
        write_items(&mut data, items);
        chunk.set_block_entity(BlockEntity {
            pos: backing.pos,
            kind: Identifier::minecraft(backing.entity),
            data,
        });
    }

    pub(crate) fn open_block_menu(
        &mut self,
        id: PlayerId,
        pos: BlockPos,
        state: BlockStateId,
    ) -> bool {
        let Some(opening) = opening_of(state) else {
            return false;
        };
        let viewers = self.viewer_count(pos);
        let shut = match opening.lid {
            Lid::Chest => blocked_above(self.world.chunks.block(pos.offset(0, 1, 0))),
            Lid::Shulker => viewers == 0 && self.shulker_is_obstructed(pos, state),
            Lid::Barrel | Lid::None => false,
        };
        if shut {
            return true;
        }
        let backing = Backing {
            pos,
            entity: opening.entity,
            block: opening.block,
            lid: opening.lid,
        };
        self.close_open_menu(id, true);
        let items = self.container_items(backing, opening.kind.container_size());
        let Some(player) = self.players.get_mut(&id) else {
            return false;
        };
        player.window_counter = player.window_counter % WINDOWS as u8 + 1;
        let window = i32::from(player.window_counter);
        let mut open = OpenMenu {
            window,
            menu: ContainerMenu::new(opening.kind),
            backing,
            state_id: 0,
        };
        let (slots, carried) = open.menu.sync_all(&items, &player.inventory);
        let state_id = open.next_state();
        player.link.send(ClientEvent::OpenScreen {
            window,
            menu: opening.kind.protocol_id(),
            title: Component::translatable(opening.title, Vec::new()),
        });
        player.link.send(ClientEvent::ContainerContent {
            window,
            state_id,
            slots,
            carried,
        });
        player.open_menu = Some(open);
        let count = self.viewer_count(pos);
        self.viewers_changed(backing, count, true);
        true
    }

    fn viewer_count(&self, pos: BlockPos) -> u8 {
        let count = self
            .players
            .values()
            .filter(|player| {
                player
                    .open_menu
                    .as_ref()
                    .is_some_and(|open| open.backing.pos == pos)
            })
            .count();
        u8::try_from(count).unwrap_or(u8::MAX)
    }

    fn shulker_is_obstructed(&self, pos: BlockPos, state: BlockStateId) -> bool {
        let (dx, dy, dz) = facing_step(state);
        self.world
            .chunks
            .block(pos.offset(dx, dy, dz))
            .is_some_and(|neighbour| neighbour.collision_shape_id() != ShapeId::EMPTY)
    }

    fn viewers_changed(&mut self, backing: Backing, count: u8, opened: bool) {
        let Some(state) = self.world.chunks.block(backing.pos) else {
            return;
        };
        if state.kind() != Some(backing.block) {
            return;
        }
        let first_or_last = if opened { count == 1 } else { count == 0 };
        let center = backing.pos.center();
        let mut position = center;
        if backing.lid == Lid::Barrel {
            let (dx, dy, dz) = facing_step(state);
            position = center
                + rspigot_domain::Vec3::new(
                    f64::from(dx) / 2.0,
                    f64::from(dy) / 2.0,
                    f64::from(dz) / 2.0,
                );
            if first_or_last
                && let Some(next) =
                    state.with_property("open", if opened { "true" } else { "false" })
            {
                self.set_block(backing.pos, next);
            }
        } else if backing.lid != Lid::None {
            self.push(
                Audience::Nearby {
                    position: center,
                    range: BLOCK_EVENT_RANGE,
                    except: None,
                },
                ClientEvent::BlockEvent {
                    pos: backing.pos,
                    action: LID_ACTION,
                    data: count,
                    block: backing.block.id().0,
                },
            );
        }
        if first_or_last && let Some((open, close)) = lid_sounds(backing.lid) {
            let pitch = self.rng.random::<f32>() * 0.1 + 0.9;
            let sound = if opened { open } else { close };
            self.play_sound(
                None,
                position,
                sound,
                SoundSource::Blocks,
                LID_VOLUME,
                pitch,
            );
        }
    }

    pub(crate) fn handle_open_menu_click(&mut self, id: PlayerId, click: &ContainerClick) -> bool {
        let Some(player) = self.players.get(&id) else {
            return false;
        };
        let Some(open) = player.open_menu.as_ref() else {
            return false;
        };
        if open.window != click.container {
            return false;
        }
        let backing = open.backing;
        let total = open.menu.kind().slot_count() as i16;
        if click.slot != -1 && click.slot != OUTSIDE && (click.slot < 0 || click.slot >= total) {
            return true;
        }
        let spectator = player.game_mode == GameMode::Spectator;
        let clicker = Clicker {
            infinite_materials: player.game_mode == GameMode::Creative,
        };
        let full_resync = click.state_id != open.state_id;
        let mut items = self.container_items(backing, open.menu.kind().container_size());
        let Some(player) = self.players.get_mut(&id) else {
            return true;
        };
        let Some(open) = player.open_menu.as_mut() else {
            return true;
        };
        let effects = if spectator {
            Vec::new()
        } else {
            open.menu.click(
                &mut items,
                &mut player.inventory,
                click.slot,
                click.button,
                click.input,
                clicker,
            )
        };
        for (slot, hash) in &click.changed {
            open.menu.receive_remote(*slot, hash.clone());
        }
        open.menu.receive_remote_carried(click.carried.clone());
        let resync = full_resync
            || effects
                .iter()
                .any(|effect| matches!(effect, MenuEffect::FullResync));
        let drops: Vec<MenuEffect> = effects
            .into_iter()
            .filter(|effect| matches!(effect, MenuEffect::Drop { .. }))
            .collect();
        if !spectator {
            self.store_container(backing, &items);
        }
        self.apply_menu_effects(id, drops);
        self.sync_open_menu(id, resync);
        self.sync_viewers(backing.pos, Some(id));
        self.sync_inventory(id);
        true
    }

    pub(crate) fn sync_open_menu(&mut self, id: PlayerId, full: bool) {
        let Some(player) = self.players.get(&id) else {
            return;
        };
        let Some(open) = player.open_menu.as_ref() else {
            return;
        };
        let backing = open.backing;
        let items = self.container_items(backing, open.menu.kind().container_size());
        let Some(player) = self.players.get_mut(&id) else {
            return;
        };
        let Some(open) = player.open_menu.as_mut() else {
            return;
        };
        let window = open.window;
        if full {
            let (slots, carried) = open.menu.sync_all(&items, &player.inventory);
            let state_id = open.next_state();
            player.link.send(ClientEvent::ContainerContent {
                window,
                state_id,
                slots,
                carried,
            });
            return;
        }
        for update in open.menu.changes(&items, &player.inventory) {
            match update {
                RemoteUpdate::Slot(slot, stack) => {
                    let state_id = open.next_state();
                    player.link.send(ClientEvent::ContainerSlot {
                        window,
                        state_id,
                        slot: slot as i16,
                        stack,
                    });
                }
                RemoteUpdate::Carried(stack) => player.link.send(ClientEvent::CursorItem(stack)),
            }
        }
    }

    fn sync_viewers(&mut self, pos: BlockPos, except: Option<PlayerId>) {
        let viewers: Vec<PlayerId> = self
            .players
            .iter()
            .filter(|(id, player)| {
                Some(**id) != except
                    && player
                        .open_menu
                        .as_ref()
                        .is_some_and(|open| open.backing.pos == pos)
            })
            .map(|(id, _)| *id)
            .collect();
        for viewer in viewers {
            self.sync_open_menu(viewer, false);
        }
    }

    pub(crate) fn close_open_menu(&mut self, id: PlayerId, notify_client: bool) -> bool {
        let Some(player) = self.players.get_mut(&id) else {
            return false;
        };
        let Some(mut open) = player.open_menu.take() else {
            return false;
        };
        let before = Self::equipment_of(player);
        let effects = open.menu.close(&mut player.inventory);
        if notify_client {
            player.link.send(ClientEvent::ContainerClose(open.window));
        }
        self.apply_menu_effects(id, effects);
        self.broadcast_equipment_changes(id, before);
        let count = self.viewer_count(open.backing.pos);
        self.viewers_changed(open.backing, count, false);
        true
    }

    pub(crate) fn handle_open_menu_close(&mut self, id: PlayerId, window: i32) -> bool {
        let matches = self
            .players
            .get(&id)
            .and_then(|player| player.open_menu.as_ref())
            .is_some_and(|open| open.window == window);
        if matches {
            self.close_open_menu(id, false);
        }
        matches
    }

    pub(crate) fn tick_open_menus(&mut self) {
        let ids: Vec<PlayerId> = self
            .players
            .iter()
            .filter(|(_, player)| player.open_menu.is_some())
            .map(|(id, _)| *id)
            .collect();
        for id in ids {
            if self.open_menu_is_valid(id) {
                self.sync_open_menu(id, false);
            } else {
                self.close_open_menu(id, true);
            }
        }
    }

    fn open_menu_is_valid(&self, id: PlayerId) -> bool {
        let Some(player) = self.players.get(&id) else {
            return false;
        };
        let Some(open) = player.open_menu.as_ref() else {
            return false;
        };
        let backing = open.backing;
        let present = self
            .world
            .chunks
            .block(backing.pos)
            .and_then(BlockStateId::kind)
            == Some(backing.block);
        let center = backing.pos.center();
        present
            && !player.vitals.is_dead()
            && player.position.distance_squared(center) <= MAX_VIEW_DISTANCE_SQUARED
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU8;

    use rspigot_items::ItemKind;

    use super::*;

    fn stack(name: &str, count: u8) -> rspigot_domain::item::ItemStack {
        let kind = ItemKind::from_name(name).expect("known item");
        rspigot_domain::item::ItemStack::new(kind.id(), NonZeroU8::new(count).expect("non zero"))
    }

    #[test]
    fn items_round_trip_through_the_block_entity_nbt() {
        let mut items = vec![None; 27];
        items[0] = Some(stack("minecraft:diamond", 3));
        items[26] = Some(stack("minecraft:stone", 64));
        let mut data = Compound::new();
        write_items(&mut data, &items);
        assert_eq!(read_items(&data, 27), items);
    }

    #[test]
    fn reading_ignores_slots_outside_the_container_and_unknown_items() {
        let mut data = Compound::new();
        data.insert(
            ITEMS,
            Tag::List(vec![
                Tag::Compound(
                    Compound::new()
                        .with("Slot", 30i8)
                        .with("id", "minecraft:stone")
                        .with("count", 1),
                ),
                Tag::Compound(
                    Compound::new()
                        .with("Slot", 1i8)
                        .with("id", "minecraft:not_an_item")
                        .with("count", 1),
                ),
            ]),
        );
        assert!(read_items(&data, 27).iter().all(Option::is_none));
    }

    #[test]
    fn storage_blocks_map_to_their_menu_and_title() {
        let menu = |block: BlockKind| {
            opening_of(block.default_state()).map(|o| (o.kind, o.entity, o.title))
        };
        assert_eq!(
            menu(BlockKind::Chest),
            Some((MenuKind::Generic { rows: 3 }, "chest", "container.chest"))
        );
        assert_eq!(
            menu(BlockKind::TrappedChest),
            Some((
                MenuKind::Generic { rows: 3 },
                "trapped_chest",
                "container.chest"
            ))
        );
        assert_eq!(
            menu(BlockKind::Barrel),
            Some((MenuKind::Generic { rows: 3 }, "barrel", "container.barrel"))
        );
        assert_eq!(
            menu(BlockKind::Hopper),
            Some((MenuKind::Hopper, "hopper", "container.hopper"))
        );
        assert_eq!(
            menu(BlockKind::Dispenser),
            Some((MenuKind::Generic3x3, "dispenser", "container.dispenser"))
        );
        assert_eq!(
            menu(BlockKind::Dropper),
            Some((MenuKind::Generic3x3, "dropper", "container.dropper"))
        );
        assert!(opening_of(BlockKind::Stone.default_state()).is_none());
    }

    #[test]
    fn every_shulker_box_colour_opens_the_shulker_menu() {
        let boxes: Vec<BlockKind> = BlockKind::ALL
            .into_iter()
            .filter(|block| block.name().ends_with("shulker_box"))
            .collect();
        assert_eq!(boxes.len(), 17);
        for block in boxes {
            let opening =
                opening_of(block.default_state()).unwrap_or_else(|| panic!("{}", block.name()));
            assert_eq!(opening.kind, MenuKind::ShulkerBox, "{}", block.name());
            assert_eq!(opening.entity, "shulker_box", "{}", block.name());
        }
    }

    #[test]
    fn a_facing_property_gives_the_step_of_the_block() {
        let state = BlockKind::Barrel
            .default_state()
            .with_property("facing", "east")
            .expect("facing");
        assert_eq!(facing_step(state), (1, 0, 0));
        assert_eq!(facing_step(BlockKind::Stone.default_state()), (0, 1, 0));
    }

    #[test]
    fn a_solid_block_above_blocks_the_chest() {
        assert!(blocked_above(Some(BlockKind::Stone.default_state())));
        assert!(!blocked_above(Some(BlockKind::Air.default_state())));
        assert!(!blocked_above(None));
    }
}
