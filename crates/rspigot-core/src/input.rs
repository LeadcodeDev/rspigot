use std::num::NonZeroU8;

use rspigot_domain::item::{HOTBAR_START, ItemStack};
use rspigot_domain::player::{GameMode, PlayerId};
use rspigot_domain::text::Component;
use rspigot_domain::{BlockPos, BlockStateId, ItemId, Rotation, Vec3};

use crate::events::{
    Animation, ClientEvent, DigAction, Direction, EquipmentSlot, Hand, MovementAction, PlayerInput,
};
use crate::placement::{is_replaceable, placed_state};
use crate::ports::ClientLink;
use crate::server::{Audience, Server};

const OFFHAND_SLOT: usize = 45;
const MAX_REACH_SQUARED: f64 = 64.0;
const MAX_MOVE_SQUARED: f64 = 100.0 * 100.0;

impl<L: ClientLink> Server<L> {
    pub(crate) fn handle_input(&mut self, id: PlayerId, input: PlayerInput) {
        if !self.players.contains_key(&id) {
            return;
        }
        match input {
            PlayerInput::Move {
                position,
                rotation,
                on_ground,
            } => self.handle_move(id, position, rotation, on_ground),
            PlayerInput::TeleportConfirm(teleport) => {
                if let Some(player) = self.players.get_mut(&id)
                    && player.pending_teleport == Some(teleport)
                {
                    player.pending_teleport = None;
                }
            }
            PlayerInput::Chat(message) => self.handle_chat(id, &message),
            PlayerInput::Command(command) => {
                let name = self
                    .players
                    .get(&id)
                    .map(|p| p.profile.name.clone())
                    .unwrap_or_default();
                tracing::info!("{name} issued server command: /{command}");
                crate::commands::execute(
                    self,
                    crate::commands::CommandSender::Player(id),
                    &command,
                );
            }
            PlayerInput::Dig {
                action,
                pos,
                face,
                sequence,
            } => self.handle_dig(id, action, pos, face, sequence),
            PlayerInput::UseItemOn {
                hand,
                pos,
                face,
                cursor,
                sequence,
                ..
            } => self.handle_use_item_on(id, hand, pos, face, cursor, sequence),
            PlayerInput::UseItem { sequence, .. } => {
                self.push(Audience::One(id), ClientEvent::BlockChangedAck(sequence))
            }
            PlayerInput::HeldSlot(slot) => {
                if slot < 9
                    && let Some(player) = self.players.get_mut(&id)
                {
                    player.inventory.selected_hotbar = slot;
                    self.broadcast_held_item(id);
                }
            }
            PlayerInput::CreativeSlot { slot, stack } => self.handle_creative_slot(id, slot, stack),
            PlayerInput::Swing(hand) => {
                if let Some(entity) = self.players.get(&id).map(|p| p.entity_id) {
                    let animation = match hand {
                        Hand::Main => Animation::SwingMainArm,
                        Hand::Off => Animation::SwingOffHand,
                    };
                    self.push(
                        Audience::Trackers(entity),
                        ClientEvent::EntityAnimation { entity, animation },
                    );
                }
            }
            PlayerInput::Movement(action) => {
                if let Some(player) = self.players.get_mut(&id) {
                    match action {
                        MovementAction::StartSprinting => player.sprinting = true,
                        MovementAction::StopSprinting => player.sprinting = false,
                        _ => return,
                    }
                    self.broadcast_metadata(id);
                }
            }
            PlayerInput::Input(state) => {
                if let Some(player) = self.players.get_mut(&id)
                    && player.sneaking != state.shift
                {
                    player.sneaking = state.shift;
                    self.broadcast_metadata(id);
                }
            }
            PlayerInput::Settings(settings) => {
                let max = self.config.view_distance;
                if let Some(player) = self.players.get_mut(&id) {
                    let view = max.min(settings.view_distance.max(2));
                    player.settings = settings;
                    player.streamer.set_view_distance(
                        view,
                        &mut self.world.chunks,
                        &self.workers,
                        &player.link,
                    );
                }
            }
            PlayerInput::ChunkBatchReceived { chunks_per_tick } => {
                if let Some(player) = self.players.get_mut(&id) {
                    player.streamer.on_batch_received(chunks_per_tick);
                }
            }
            PlayerInput::Loaded => {
                if let Some(player) = self.players.get_mut(&id) {
                    player.loaded = true;
                }
            }
            PlayerInput::Abilities { flying } => {
                if let Some(player) = self.players.get_mut(&id) {
                    player.flying = flying && player.abilities().may_fly;
                }
            }
            PlayerInput::ChangeGameMode(mode) => {
                if self.players.get(&id).is_some_and(|p| p.operator) {
                    self.apply_game_mode(id, mode);
                }
            }
            PlayerInput::PickBlock { pos, .. } => self.handle_pick_block(id, pos),
            PlayerInput::CommandSuggestion { id: request, text } => {
                self.handle_suggestion(id, request, &text)
            }
            PlayerInput::Respawn | PlayerInput::Attack(_) | PlayerInput::Interact { .. } => {}
        }
    }

    fn handle_move(
        &mut self,
        id: PlayerId,
        position: Option<Vec3>,
        rotation: Option<Rotation>,
        on_ground: bool,
    ) {
        let Some(player) = self.players.get_mut(&id) else {
            return;
        };
        if player.pending_teleport.is_some() {
            return;
        }
        let from = player.position;
        if let Some(target) = position {
            if !target.is_finite()
                || target.x.abs() > 3.0e7
                || target.z.abs() > 3.0e7
                || target.y.abs() > 2.0e7
            {
                player.link.disconnect(Component::translatable(
                    "multiplayer.disconnect.invalid_player_movement",
                    Vec::new(),
                ));
                return;
            }
            if from.distance_squared(target) > MAX_MOVE_SQUARED {
                tracing::warn!(
                    "{} moved too quickly! {:.2},{:.2},{:.2}",
                    player.profile.name,
                    target.x - from.x,
                    target.y - from.y,
                    target.z - from.z
                );
                let teleport = player.next_teleport_id();
                player.link.send(ClientEvent::Teleport {
                    id: teleport,
                    position: from,
                    rotation: player.rotation,
                });
                return;
            }
            player.position = target;
        }
        if let Some(rotation) = rotation {
            player.rotation = Rotation::new(
                rotation.yaw.rem_euclid(360.0),
                rotation.pitch.clamp(-90.0, 90.0),
            );
        }
        player.on_ground = on_ground;
        let chunk = player.position.chunk();
        if chunk != player.streamer.center() {
            player
                .streamer
                .move_to(chunk, &mut self.world.chunks, &self.workers, &player.link);
        }
        let entity = player.entity_id;
        let to = player.position;
        let yaw = player.rotation.yaw;
        if position.is_some() || rotation.is_some() {
            self.push(
                Audience::Trackers(entity),
                ClientEvent::EntityMoved {
                    entity,
                    from,
                    to,
                    rotation: rotation.map(|_| Rotation::new(yaw, to_pitch(rotation))),
                    on_ground,
                },
            );
        }
        if rotation.is_some() {
            self.push(
                Audience::Trackers(entity),
                ClientEvent::EntityHeadRotation { entity, yaw },
            );
        }
    }

    fn handle_chat(&mut self, id: PlayerId, message: &str) {
        let Some(player) = self.players.get(&id) else {
            return;
        };
        if message.chars().count() > 256 || message.chars().any(|c| c == '§' || c.is_control()) {
            player.link.disconnect(Component::translatable(
                "multiplayer.disconnect.illegal_characters",
                Vec::new(),
            ));
            return;
        }
        let name = player.profile.name.clone();
        tracing::info!("<{name}> {message}");
        self.push(
            Audience::All,
            ClientEvent::PlayerChat {
                sender: id,
                sender_name: Component::text(name),
                message: message.to_owned(),
            },
        );
    }

    fn within_reach(&self, id: PlayerId, pos: BlockPos) -> bool {
        self.players.get(&id).is_some_and(|p| {
            p.eye_position()
                .distance_squared(pos.center() + Vec3::new(0.0, 0.5, 0.0))
                <= MAX_REACH_SQUARED
        })
    }

    fn is_protected(&self, id: PlayerId, pos: BlockPos) -> bool {
        let radius = self.config.spawn_protection as i32;
        if radius == 0
            || self.players.get(&id).is_some_and(|p| p.operator)
            || self.players.len() <= 1 && self.operators.is_empty()
        {
            return false;
        }
        let spawn = self.world.spawn.block();
        (pos.x - spawn.x).abs() <= radius && (pos.z - spawn.z).abs() <= radius
    }

    fn refuse_block(&mut self, id: PlayerId, pos: BlockPos, sequence: i32) {
        if let Some(state) = self.world.chunks.block(pos) {
            self.push(Audience::One(id), ClientEvent::BlockUpdate { pos, state });
        }
        self.push(Audience::One(id), ClientEvent::BlockChangedAck(sequence));
    }

    fn handle_dig(
        &mut self,
        id: PlayerId,
        action: DigAction,
        pos: BlockPos,
        _face: Direction,
        sequence: i32,
    ) {
        let Some(mode) = self.players.get(&id).map(|p| p.game_mode) else {
            return;
        };
        match action {
            DigAction::Start | DigAction::Finish => {
                let breaks = match action {
                    DigAction::Start => mode == GameMode::Creative,
                    _ => mode == GameMode::Survival,
                };
                if breaks {
                    if !self.within_reach(id, pos) || self.is_protected(id, pos) {
                        self.refuse_block(id, pos, sequence);
                        return;
                    }
                    self.set_block(pos, BlockStateId::AIR);
                }
                self.push(Audience::One(id), ClientEvent::BlockChangedAck(sequence));
            }
            DigAction::Cancel => {
                self.push(Audience::One(id), ClientEvent::BlockChangedAck(sequence))
            }
            DigAction::SwapHands => {
                if let Some(player) = self.players.get_mut(&id) {
                    let main = HOTBAR_START + usize::from(player.inventory.selected_hotbar);
                    let main_stack = player.inventory.get(main).cloned();
                    let off_stack = player.inventory.get(OFFHAND_SLOT).cloned();
                    player.inventory.set(main, off_stack);
                    player.inventory.set(OFFHAND_SLOT, main_stack);
                    self.send_inventory(id);
                    self.broadcast_held_item(id);
                }
            }
            DigAction::DropItem
            | DigAction::DropStack
            | DigAction::ReleaseUseItem
            | DigAction::Stab => self.send_inventory(id),
        }
    }

    pub(crate) fn set_block(&mut self, pos: BlockPos, state: BlockStateId) -> bool {
        match self.world.chunks.set_block(pos, state) {
            Some(old) if old != state => {
                self.push(
                    Audience::ChunkViewers(pos.chunk()),
                    ClientEvent::BlockUpdate { pos, state },
                );
                true
            }
            _ => false,
        }
    }

    fn handle_use_item_on(
        &mut self,
        id: PlayerId,
        hand: Hand,
        pos: BlockPos,
        face: Direction,
        cursor: Vec3,
        sequence: i32,
    ) {
        let Some(player) = self.players.get(&id) else {
            return;
        };
        let slot = match hand {
            Hand::Main => HOTBAR_START + usize::from(player.inventory.selected_hotbar),
            Hand::Off => OFFHAND_SLOT,
        };
        let sneaking = player.sneaking;
        let game_mode = player.game_mode;
        let rotation = player.rotation;
        let stack = player.inventory.get(slot).cloned();
        let Some(clicked) = self.world.chunks.block(pos) else {
            self.push(Audience::One(id), ClientEvent::BlockChangedAck(sequence));
            return;
        };
        if !sneaking && self.toggle_openable(pos, clicked) {
            self.push(Audience::One(id), ClientEvent::BlockChangedAck(sequence));
            return;
        }
        let placeable = stack
            .as_ref()
            .and_then(|s| rspigot_data::items::block_state_for(s.item.0))
            .map(BlockStateId);
        let (Some(stack), Some(default)) = (stack, placeable) else {
            self.push(Audience::One(id), ClientEvent::BlockChangedAck(sequence));
            return;
        };
        if game_mode == GameMode::Spectator || game_mode == GameMode::Adventure {
            self.refuse_block(id, pos, sequence);
            return;
        }
        let (dx, dy, dz) = face.offset();
        let target = if is_replaceable(clicked) {
            pos
        } else {
            pos.offset(dx, dy, dz)
        };
        let target_ok = self.world.chunks.block(target).is_some_and(is_replaceable)
            && self.world.dimension.height.contains(target.y);
        if !target_ok
            || !self.within_reach(id, target)
            || self.is_protected(id, target)
            || self.intersects_player(target)
        {
            self.refuse_block(id, target, sequence);
            return;
        }
        let state = placed_state(default, face, rotation.yaw, rotation.pitch, cursor);
        self.set_block(target, state);
        if let Some(name) = rspigot_data::blocks::block_name(state.0)
            && name.ends_with("_door")
            && let Some(upper) = rspigot_data::blocks::with_property(state.0, "half", "upper")
        {
            let above = target.offset(0, 1, 0);
            if self.world.chunks.block(above).is_some_and(is_replaceable) {
                self.set_block(above, BlockStateId(upper));
            }
        }
        self.push(Audience::One(id), ClientEvent::BlockChangedAck(sequence));
        if game_mode == GameMode::Survival {
            self.consume_one(id, slot, stack);
        }
    }

    fn toggle_openable(&mut self, pos: BlockPos, state: BlockStateId) -> bool {
        let Some(name) = rspigot_data::blocks::block_name(state.0) else {
            return false;
        };
        if name.starts_with("minecraft:iron_") {
            return false;
        }
        let Some(open) = rspigot_data::blocks::property(state.0, "open") else {
            return false;
        };
        let next = if open == "true" { "false" } else { "true" };
        let Some(toggled) = rspigot_data::blocks::with_property(state.0, "open", next) else {
            return false;
        };
        self.set_block(pos, BlockStateId(toggled));
        if name.ends_with("_door") {
            let other = match rspigot_data::blocks::property(state.0, "half") {
                Some("lower") => pos.offset(0, 1, 0),
                _ => pos.offset(0, -1, 0),
            };
            if let Some(other_state) = self.world.chunks.block(other)
                && rspigot_data::blocks::block_name(other_state.0) == Some(name)
                && let Some(other_toggled) =
                    rspigot_data::blocks::with_property(other_state.0, "open", next)
            {
                self.set_block(other, BlockStateId(other_toggled));
            }
        }
        true
    }

    fn intersects_player(&self, pos: BlockPos) -> bool {
        self.players.values().any(|p| {
            p.game_mode != GameMode::Spectator && {
                let feet = p.position.block();
                pos.x == feet.x && pos.z == feet.z && (pos.y == feet.y || pos.y == feet.y + 1)
            }
        })
    }

    fn consume_one(&mut self, id: PlayerId, slot: usize, stack: ItemStack) {
        let Some(player) = self.players.get_mut(&id) else {
            return;
        };
        let remaining = stack.count.get() - 1;
        let new = NonZeroU8::new(remaining).map(|count| ItemStack { count, ..stack });
        player.inventory.set(slot, new.clone());
        let state_id = player.next_inventory_state();
        player.link.send(ClientEvent::InventorySlot {
            state_id,
            slot: slot as i16,
            stack: new,
        });
        self.broadcast_held_item(id);
    }

    fn handle_creative_slot(&mut self, id: PlayerId, slot: i16, stack: Option<ItemStack>) {
        let Some(player) = self.players.get_mut(&id) else {
            return;
        };
        if player.game_mode != GameMode::Creative {
            self.send_inventory(id);
            return;
        }
        let Ok(index) = usize::try_from(slot) else {
            return;
        };
        if player.inventory.set(index, stack)
            && index == HOTBAR_START + usize::from(player.inventory.selected_hotbar)
        {
            self.broadcast_held_item(id);
        }
    }

    fn handle_pick_block(&mut self, id: PlayerId, pos: BlockPos) {
        let Some(state) = self.world.chunks.block(pos) else {
            return;
        };
        let Some(name) = rspigot_data::blocks::block_name(state.0) else {
            return;
        };
        let Some(item) = rspigot_data::items::id(name) else {
            return;
        };
        let Some(player) = self.players.get_mut(&id) else {
            return;
        };
        if player.game_mode != GameMode::Creative {
            return;
        }
        let slot = HOTBAR_START + usize::from(player.inventory.selected_hotbar);
        player
            .inventory
            .set(slot, Some(ItemStack::new(ItemId(item), NonZeroU8::MIN)));
        let state_id = player.next_inventory_state();
        player.link.send(ClientEvent::InventorySlot {
            state_id,
            slot: slot as i16,
            stack: player.inventory.get(slot).cloned(),
        });
        self.broadcast_held_item(id);
    }

    fn handle_suggestion(&mut self, id: PlayerId, request: i32, text: &str) {
        let input = text.trim_start_matches('/');
        let offset = (text.len() - input.len()) as i32;
        let (start, prefix, candidates): (i32, &str, Vec<String>) = match input.rsplit_once(' ') {
            None => {
                let operator = self.players.get(&id).is_some_and(|p| p.operator);
                let tree = self.command_tree(operator);
                let names = tree.nodes[tree.root]
                    .children
                    .iter()
                    .filter_map(|c| match &tree.nodes[*c].kind {
                        crate::commands::CommandNodeKind::Literal(name) => Some(name.clone()),
                        _ => None,
                    })
                    .collect();
                (offset, input, names)
            }
            Some((head, last)) => (
                offset + head.len() as i32 + 1,
                last,
                self.players
                    .values()
                    .map(|p| p.profile.name.clone())
                    .collect(),
            ),
        };
        let lower = prefix.to_ascii_lowercase();
        let mut matches: Vec<String> = candidates
            .into_iter()
            .filter(|c| c.to_ascii_lowercase().starts_with(&lower))
            .collect();
        matches.sort();
        self.push(
            Audience::One(id),
            ClientEvent::CommandSuggestions {
                id: request,
                start,
                length: prefix.len() as i32,
                matches,
            },
        );
    }

    pub(crate) fn send_inventory(&mut self, id: PlayerId) {
        if let Some(player) = self.players.get_mut(&id) {
            let state_id = player.next_inventory_state();
            player.link.send(ClientEvent::InventoryContent {
                state_id,
                slots: player.inventory.slots().to_vec(),
                carried: None,
            });
        }
    }

    pub(crate) fn broadcast_held_item(&mut self, id: PlayerId) {
        let Some(player) = self.players.get(&id) else {
            return;
        };
        let entity = player.entity_id;
        let main = player.inventory.held().cloned();
        let off = player.inventory.get(OFFHAND_SLOT).cloned();
        self.push(
            Audience::Trackers(entity),
            ClientEvent::EntityEquipment {
                entity,
                slots: vec![
                    (EquipmentSlot::MainHand, main),
                    (EquipmentSlot::OffHand, off),
                ],
            },
        );
    }

    pub(crate) fn broadcast_metadata(&mut self, id: PlayerId) {
        let Some(player) = self.players.get(&id) else {
            return;
        };
        let entity = player.entity_id;
        let entries = Self::metadata(player);
        self.push(
            Audience::Trackers(entity),
            ClientEvent::EntityMetadata { entity, entries },
        );
    }

    pub(crate) fn apply_game_mode(&mut self, id: PlayerId, mode: GameMode) {
        let Some(player) = self.players.get_mut(&id) else {
            return;
        };
        if player.game_mode == mode {
            return;
        }
        player.previous_game_mode = Some(player.game_mode);
        player.game_mode = mode;
        if !mode.can_fly() {
            player.flying = false;
        }
        player.link.send(ClientEvent::GameEvent(
            crate::events::GameEvent::ChangeGameMode(mode),
        ));
        player.link.send(ClientEvent::Abilities(player.abilities()));
        self.push(
            Audience::All,
            ClientEvent::PlayerInfoGameMode {
                player: id,
                game_mode: mode,
            },
        );
    }
}

fn to_pitch(rotation: Option<Rotation>) -> f32 {
    rotation.map_or(0.0, |r| r.pitch.clamp(-90.0, 90.0))
}
