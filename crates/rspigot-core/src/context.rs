use std::hash::{BuildHasher, Hasher};

use rspigot_domain::effect::{EffectId, EffectInstance};
use rspigot_domain::item::PLAYER_INVENTORY_SIZE;
use rspigot_domain::player::{GameMode, PlayerId};
use rspigot_domain::text::Component;
use rspigot_domain::world::Difficulty;
use rspigot_domain::{BlockPos, ItemId, Vec3};

use crate::admin::{AdminLists, ListEntry};
use crate::chunks::{ChunkJob, Priority};
use crate::commands::Weather;
use crate::commands::{CommandContext, CommandSender};
use crate::events::{ClientEvent, GameEvent};
use crate::ports::ClientLink;
use crate::server::{Audience, Server};

impl<L: ClientLink> CommandContext for Server<L> {
    fn reply(&mut self, sender: CommandSender, message: Component) {
        match sender {
            CommandSender::Console => tracing::info!("{}", crate::console::console_text(&message)),
            CommandSender::Player(id) => self.push(Audience::One(id), ClientEvent::system(message)),
        }
    }

    fn broadcast(&mut self, message: Component) {
        tracing::info!("{}", crate::console::console_text(&message));
        self.push(Audience::All, ClientEvent::system(message));
    }

    fn is_operator(&self, sender: CommandSender) -> bool {
        match sender {
            CommandSender::Console => true,
            CommandSender::Player(id) => self.players.get(&id).is_some_and(|p| p.operator),
        }
    }

    fn sender_name(&self, sender: CommandSender) -> String {
        match sender {
            CommandSender::Console => "Server".to_owned(),
            CommandSender::Player(id) => self.player_name(id).unwrap_or_default(),
        }
    }

    fn find_player(&self, name: &str) -> Option<PlayerId> {
        self.players
            .values()
            .find(|p| p.profile.name.eq_ignore_ascii_case(name))
            .map(|p| p.id)
    }

    fn player_name(&self, player: PlayerId) -> Option<String> {
        self.players.get(&player).map(|p| p.profile.name.clone())
    }

    fn player_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self
            .players
            .values()
            .map(|p| p.profile.name.clone())
            .collect();
        names.sort_by_key(|n| n.to_ascii_lowercase());
        names
    }

    fn max_players(&self) -> u32 {
        self.config.max_players
    }

    fn player_position(&self, player: PlayerId) -> Option<Vec3> {
        self.players.get(&player).map(|p| p.position)
    }

    fn set_game_mode(&mut self, player: PlayerId, mode: GameMode) {
        self.apply_game_mode(player, mode);
    }

    fn teleport(&mut self, player: PlayerId, position: Vec3) {
        let Some(p) = self.players.get_mut(&player) else {
            return;
        };
        let from = p.position;
        p.position = position;
        let id = p.next_teleport_id();
        p.link.send(ClientEvent::Teleport {
            id,
            position,
            rotation: p.rotation,
        });
        let chunk = position.chunk();
        p.streamer
            .move_to(chunk, &mut self.world.chunks, &self.workers, &p.link);
        let entity = p.entity_id;
        let on_ground = p.on_ground;
        self.push(
            Audience::Trackers(entity),
            ClientEvent::EntityMoved {
                entity,
                from,
                to: position,
                rotation: None,
                on_ground,
            },
        );
    }

    fn day_time(&self) -> i64 {
        self.world.level.day_time
    }

    fn game_time(&self) -> i64 {
        self.world.level.game_time
    }

    fn set_day_time(&mut self, time: i64) {
        self.world.level.day_time = time;
        let event = self.time_event();
        self.push(Audience::All, event);
    }

    fn give(&mut self, player: PlayerId, item: ItemId, count: u32) -> bool {
        let Some(p) = self.players.get_mut(&player) else {
            return false;
        };
        let outcome = p.inventory.insert(item, count, item.max_stack_size());
        let remaining = outcome.remaining;
        self.send_inventory(player);
        self.broadcast_held_item(player);
        remaining < count
    }

    fn clear_inventory(&mut self, player: PlayerId) -> u32 {
        let Some(p) = self.players.get_mut(&player) else {
            return 0;
        };
        let mut removed = 0u32;
        for slot in 0..PLAYER_INVENTORY_SIZE {
            if let Some(stack) = p.inventory.get(slot) {
                removed += u32::from(stack.count.get());
                p.inventory.set(slot, None);
            }
        }
        self.send_inventory(player);
        self.broadcast_held_item(player);
        removed
    }

    fn kick(&mut self, player: PlayerId, reason: Component) {
        if let Some(p) = self.players.get(&player) {
            tracing::info!(
                "Kicked {}: {}",
                p.profile.name,
                crate::console::console_text(&reason)
            );
            p.link.disconnect(reason);
        }
        self.remove_player(player, true);
    }

    fn set_operator(&mut self, name: &str, operator: bool) -> bool {
        let online = self.find_player(name);
        let entry = online
            .and_then(|id| self.players.get(&id))
            .map_or_else(|| ListEntry::named(name), |p| ListEntry::of(&p.profile));
        let changed = if operator {
            AdminLists::add(&mut self.admin.operators, entry)
        } else {
            AdminLists::remove(&mut self.admin.operators, name)
        };
        if changed {
            self.publish_admin();
        }
        if let Some(id) = online {
            let tree = self.command_tree(operator);
            if let Some(p) = self.players.get_mut(&id) {
                p.operator = operator;
                p.link.send(ClientEvent::OperatorLevel {
                    entity: p.entity_id,
                    level: if operator { 4 } else { 0 },
                });
                p.link.send(ClientEvent::Commands(tree));
            }
        }
        changed
    }

    fn set_whitelist_enabled(&mut self, enabled: bool) -> bool {
        if self.admin.whitelist_enabled == enabled {
            return false;
        }
        self.admin.whitelist_enabled = enabled;
        self.publish_admin();
        true
    }

    fn whitelist_add(&mut self, name: &str) -> bool {
        let entry = self
            .find_player(name)
            .and_then(|id| self.players.get(&id))
            .map_or_else(|| ListEntry::named(name), |p| ListEntry::of(&p.profile));
        let changed = AdminLists::add(&mut self.admin.whitelist, entry);
        if changed {
            self.publish_admin();
        }
        changed
    }

    fn whitelist_remove(&mut self, name: &str) -> bool {
        let changed = AdminLists::remove(&mut self.admin.whitelist, name);
        if changed {
            self.publish_admin();
        }
        changed
    }

    fn whitelist_names(&self) -> Vec<String> {
        self.admin
            .whitelist
            .iter()
            .map(|e| e.name.clone())
            .collect()
    }

    fn select_players(
        &self,
        sender: CommandSender,
        selector: &str,
    ) -> Result<Vec<PlayerId>, Component> {
        let Some(kind) = selector.strip_prefix('@') else {
            return Ok(self.find_player(selector).into_iter().collect());
        };
        if let Some((_, options)) = kind.split_once('[') {
            let option = options.split(['=', ']']).next().unwrap_or_default();
            return Err(Component::translatable(
                "argument.entity.options.unknown",
                vec![Component::text(option)],
            ));
        }
        let origin = match sender {
            CommandSender::Player(id) => self
                .players
                .get(&id)
                .map_or(self.world.spawn, |p| p.position),
            CommandSender::Console => self.world.spawn,
        };
        let mut nearest: Vec<(f64, PlayerId)> = self
            .players
            .values()
            .map(|p| (p.position.distance_squared(origin), p.id))
            .collect();
        nearest.sort_by(|a, b| a.0.total_cmp(&b.0));
        match kind {
            "s" => Ok(match sender {
                CommandSender::Player(id) => vec![id],
                CommandSender::Console => Vec::new(),
            }),
            "p" => Ok(nearest.first().map(|(_, id)| *id).into_iter().collect()),
            "a" | "e" => Ok(nearest.into_iter().map(|(_, id)| id).collect()),
            "r" => {
                let mut hasher = std::collections::hash_map::RandomState::new().build_hasher();
                hasher.write_u64(self.world.level.game_time as u64);
                let index = (hasher.finish() as usize).checked_rem(nearest.len());
                Ok(index.map(|i| nearest[i].1).into_iter().collect())
            }
            other => Err(Component::translatable(
                "argument.entity.selector.unknown",
                vec![Component::text(format!("@{other}"))],
            )),
        }
    }

    fn give_effect(&mut self, player: PlayerId, effect: EffectInstance) -> bool {
        self.apply_effect(player, effect)
    }

    fn clear_effect(&mut self, player: PlayerId, effect: Option<EffectId>) -> bool {
        self.remove_effects(player, effect)
    }

    fn kill(&mut self, player: PlayerId) -> bool {
        Server::kill(self, player)
    }

    fn seed(&self) -> i64 {
        self.world.level.seed
    }

    fn difficulty(&self) -> Difficulty {
        self.world.difficulty
    }

    fn set_difficulty(&mut self, difficulty: Difficulty) {
        self.world.difficulty = difficulty;
        self.push(
            Audience::All,
            ClientEvent::Difficulty {
                difficulty,
                locked: false,
            },
        );
    }

    fn set_weather(&mut self, weather: Weather) {
        let was_raining = self.world.level.raining;
        self.world.level.raining = weather != Weather::Clear;
        self.world.level.thundering = weather == Weather::Thunder;
        if was_raining != self.world.level.raining {
            let event = if self.world.level.raining {
                GameEvent::StartRaining
            } else {
                GameEvent::StopRaining
            };
            self.push(Audience::All, ClientEvent::GameEvent(event));
        }
        let rain = if self.world.level.raining { 1.0 } else { 0.0 };
        let thunder = if self.world.level.thundering {
            1.0
        } else {
            0.0
        };
        self.push(
            Audience::All,
            ClientEvent::GameEvent(GameEvent::RainLevelChange(rain)),
        );
        self.push(
            Audience::All,
            ClientEvent::GameEvent(GameEvent::ThunderLevelChange(thunder)),
        );
    }

    fn set_world_spawn(&mut self, pos: BlockPos) {
        self.world.level.spawn = Some(pos);
        self.world.spawn = pos.center();
        self.push(
            Audience::All,
            ClientEvent::SpawnPosition {
                dimension: self.world.dimension.key.clone(),
                pos,
                yaw: self.world.level.spawn_angle,
                pitch: 0.0,
            },
        );
    }

    fn save_all(&mut self) -> usize {
        let count = self.world.chunks.save_dirty(&self.workers);
        self.workers.submit(Priority::BACKGROUND, ChunkJob::Flush);
        count
    }

    fn tps(&self) -> [f64; 3] {
        Server::tps(self)
    }

    fn stop(&mut self) {
        self.running = false;
    }
}
