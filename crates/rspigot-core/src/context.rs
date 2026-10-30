use std::hash::{BuildHasher, Hasher};

use rspigot_domain::effect::{EffectId, EffectInstance};
use rspigot_domain::entity::{Aabb, EntityState, living, save};
use rspigot_domain::item::PLAYER_INVENTORY_SIZE;
use rspigot_domain::player::{GameMode, PlayerId};
use rspigot_domain::spawn::SpawnReason;
use rspigot_domain::text::Component;
use rspigot_domain::world::Difficulty;
use rspigot_domain::{BlockPos, EntityId, ItemId, Rotation, Vec3};
use rspigot_entities::EntityKind as Kind;
use rspigot_nbt::Compound;

use crate::admin::{AdminLists, ListEntry};
use crate::chunks::{ChunkJob, Priority};
use crate::commands::selector::{self, Selector, SelectorBase, Sort, TypeMatch};
use crate::commands::{CommandContext, CommandSender, Target, Weather};
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
        if selector.contains('[') {
            let parsed = selector::parse(selector)?;
            return Ok(self
                .evaluate_selector(sender, &parsed)
                .into_iter()
                .filter_map(|t| match t {
                    Target::Player(id) => Some(id),
                    Target::Entity(_) => None,
                })
                .collect());
        }
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
        self.save_all_entities();
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

    fn sender_rotation(&self, sender: CommandSender) -> Rotation {
        match sender {
            CommandSender::Player(id) => self
                .players
                .get(&id)
                .map_or_else(Rotation::default, |p| p.rotation),
            CommandSender::Console => Rotation::default(),
        }
    }

    fn sender_origin(&self, sender: CommandSender) -> Vec3 {
        match sender {
            CommandSender::Player(id) => self.players.get(&id).map_or(Vec3::ZERO, |p| p.position),
            CommandSender::Console => {
                let spawn = self.world.spawn.block();
                Vec3::new(f64::from(spawn.x), f64::from(spawn.y), f64::from(spawn.z))
            }
        }
    }

    fn select_entities(&self, sender: CommandSender, text: &str) -> Result<Vec<Target>, Component> {
        let selector = selector::parse(text)?;
        Ok(self.evaluate_selector(sender, &selector))
    }

    fn target_name(&self, target: Target) -> Component {
        match target {
            Target::Player(id) => Component::text(self.player_name(id).unwrap_or_default()),
            Target::Entity(id) => self.entity_name(id).unwrap_or_else(|| Component::text("")),
        }
    }

    fn target_position(&self, target: Target) -> Option<Vec3> {
        match target {
            Target::Player(id) => self.player_position(id),
            Target::Entity(id) => self.entities.store.get(id).map(|e| e.base.position),
        }
    }

    fn teleport_target(&mut self, target: Target, position: Vec3) {
        match target {
            Target::Player(id) => CommandContext::teleport(self, id, position),
            Target::Entity(id) => {
                if let Some(entity) = self.entities.store.get_mut(id) {
                    let base = &mut entity.base;
                    base.position = position;
                    base.old_position = position;
                    let living = matches!(&entity.state, EntityState::Living(l) if l.fall_flying);
                    if !living {
                        base.velocity = base.velocity.multiply(1.0, 0.0, 1.0);
                        base.on_ground = true;
                    }
                    base.needs_sync = true;
                }
                self.entities.store.reindex(id);
            }
        }
    }

    fn kill_target(&mut self, target: Target) -> bool {
        match target {
            Target::Player(id) => Server::kill(self, id),
            Target::Entity(id) => self.kill_entity(id),
        }
    }

    fn give_effect_to(&mut self, target: Target, effect: EffectInstance) -> bool {
        match target {
            Target::Player(id) => self.apply_effect(id, effect),
            Target::Entity(id) => self.apply_entity_effect(id, effect),
        }
    }

    fn clear_effect_of(&mut self, target: Target, effect: Option<EffectId>) -> bool {
        match target {
            Target::Player(id) => self.remove_effects(id, effect),
            Target::Entity(id) => self.remove_entity_effects(id, effect),
        }
    }

    fn summon(
        &mut self,
        _sender: CommandSender,
        kind: Kind,
        position: Vec3,
        nbt: Option<Compound>,
    ) -> Result<Component, Component> {
        let failed = |key: &str| Component::translatable(key, Vec::new());
        let block = position.block();
        if !(-20_000_000..20_000_000).contains(&block.y)
            || !(-30_000_000..30_000_000).contains(&block.x)
            || !(-30_000_000..30_000_000).contains(&block.z)
        {
            return Err(failed("commands.summon.invalidPosition"));
        }
        if self.world.difficulty == Difficulty::Peaceful && !kind.allowed_in_peaceful() {
            return Err(failed("commands.summon.failed.peaceful"));
        }
        if !save::is_simulated(kind) {
            return Err(failed("commands.summon.failed"));
        }
        let finalize = nbt.is_none();
        let mut compound = nbt.unwrap_or_default();
        compound.insert("id", kind.name());
        let id = self.allocate_entity_id();
        let uuid = self.random_uuid();
        let mut entity =
            save::load_entity(&compound, id, uuid).map_err(|_| failed("commands.summon.failed"))?;
        let (yaw, pitch) = (entity.base.yaw, entity.base.pitch);
        entity.base.snap_to(position, yaw, pitch);
        if finalize && let EntityState::Living(state) = &mut entity.state {
            let rng = &mut self.rng;
            let mut next_double = || rand::Rng::random::<f64>(rng);
            let mut doubles: Vec<f64> = (0..2).map(|_| next_double()).collect();
            let mut take_double = || doubles.remove(0);
            let float = rand::Rng::random::<f32>(&mut self.rng);
            let mut take_float = || float;
            living::finalize_spawn(
                &mut entity.base,
                state,
                SpawnReason::Command,
                None,
                &mut take_double,
                &mut take_float,
            );
        }
        let name = match entity.base.custom_name() {
            Some(name) => name.clone(),
            None => {
                Component::translatable(crate::entities::entity_translation_key(kind), Vec::new())
            }
        };
        self.add_entity(entity)
            .map(|_| name)
            .map_err(|_| failed("commands.summon.failed.uuid"))
    }
}

struct Candidate<'a> {
    target: Target,
    kind: Kind,
    position: Vec3,
    bounds: Aabb,
    name: String,
    tags: &'a [String],
    alive: bool,
}

fn english_name(kind: Kind) -> String {
    let name = kind.name();
    let path = name.strip_prefix("minecraft:").unwrap_or(name);
    path.split('_')
        .map(|word| {
            let mut chars = word.chars();
            chars.next().map_or_else(String::new, |first| {
                first.to_uppercase().collect::<String>() + chars.as_str()
            })
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn type_matches(kind: Kind, matcher: &TypeMatch) -> bool {
    match matcher {
        TypeMatch::Kind(expected) => kind == *expected,
        TypeMatch::Tag(tag) => {
            let tag = if tag.contains(':') {
                tag.clone()
            } else {
                format!("minecraft:{tag}")
            };
            rspigot_data::tags::tag("minecraft:entity_type", &tag)
                .is_some_and(|ids| ids.binary_search(&u32::from(kind.id().0)).is_ok())
        }
    }
}

fn selection_box(selector: &Selector, origin: Vec3) -> Option<Aabb> {
    if selector.dx.is_none() && selector.dy.is_none() && selector.dz.is_none() {
        return None;
    }
    let edge = |d: f64| if d < 0.0 { (d, 1.0) } else { (0.0, d + 1.0) };
    let (x0, x1) = edge(selector.dx.unwrap_or(0.0));
    let (y0, y1) = edge(selector.dy.unwrap_or(0.0));
    let (z0, z1) = edge(selector.dz.unwrap_or(0.0));
    Some(Aabb::new(x0, y0, z0, x1, y1, z1).move_by(origin))
}

impl<L: ClientLink> Server<L> {
    fn candidates(&self, include_entities: bool) -> Vec<Candidate<'_>> {
        let mut out: Vec<Candidate<'_>> = self
            .players
            .values()
            .map(|p| Candidate {
                target: Target::Player(p.id),
                kind: Kind::Player,
                position: p.position,
                bounds: Aabb::of_size(p.position, 0.6, p.height() as f32),
                name: p.profile.name.clone(),
                tags: &[],
                alive: !p.vitals.is_dead(),
            })
            .collect();
        if include_entities {
            out.extend(self.entities.store.iter().map(|e| {
                Candidate {
                    target: Target::Entity(e.base.id),
                    kind: e.base.kind,
                    position: e.base.position,
                    bounds: e.base.bounding_box(),
                    name: e
                        .base
                        .custom_name()
                        .map_or_else(|| english_name(e.base.kind), |n| n.plain_text()),
                    tags: &e.base.tags,
                    alive: !e.base.is_removed()
                        && e.living().is_none_or(|_| living::health(&e.base) > 0.0),
                }
            }));
        }
        out
    }

    fn evaluate_selector(&self, sender: CommandSender, selector: &Selector) -> Vec<Target> {
        let base = self.sender_origin(sender);
        let origin = Vec3::new(
            selector.x.unwrap_or(base.x),
            selector.y.unwrap_or(base.y),
            selector.z.unwrap_or(base.z),
        );
        let bounds = selection_box(selector, origin);
        let matches = |candidate: &Candidate<'_>| {
            if selector.alive_only && !candidate.alive {
                return false;
            }
            if !selector
                .types
                .iter()
                .all(|(negated, matcher)| type_matches(candidate.kind, matcher) != *negated)
            {
                return false;
            }
            if let Some((min, max)) = selector.distance {
                let distance = candidate.position.distance_squared(origin);
                if min.is_some_and(|m| distance < m * m) || max.is_some_and(|m| distance > m * m) {
                    return false;
                }
            }
            if let Some(bounds) = &bounds
                && !bounds.intersects(&candidate.bounds)
            {
                return false;
            }
            if !selector
                .names
                .iter()
                .all(|(negated, name)| (candidate.name == *name) != *negated)
            {
                return false;
            }
            selector.tags.iter().all(|(negated, tag)| {
                let has = if tag.is_empty() {
                    candidate.tags.is_empty()
                } else {
                    candidate.tags.iter().any(|t| t == tag)
                };
                has != *negated
            })
        };
        let mut found: Vec<Candidate<'_>> = match &selector.base {
            SelectorBase::Name(name) => self
                .candidates(false)
                .into_iter()
                .filter(|c| c.name.eq_ignore_ascii_case(name))
                .collect(),
            SelectorBase::Uuid(uuid) => self
                .candidates(true)
                .into_iter()
                .filter(|c| match c.target {
                    Target::Player(id) => id.0 == *uuid,
                    Target::Entity(id) => self
                        .entities
                        .store
                        .get(id)
                        .is_some_and(|e| e.base.uuid == *uuid),
                })
                .collect(),
            SelectorBase::Sender => match sender {
                CommandSender::Player(id) => self
                    .candidates(false)
                    .into_iter()
                    .filter(|c| c.target == Target::Player(id) && matches(c))
                    .collect(),
                CommandSender::Console => Vec::new(),
            },
            SelectorBase::Search => self
                .candidates(selector.includes_entities)
                .into_iter()
                .filter(|c| matches(c))
                .collect(),
        };
        match selector.sort {
            Sort::Arbitrary => {}
            Sort::Nearest => found.sort_by(|a, b| {
                a.position
                    .distance_squared(origin)
                    .total_cmp(&b.position.distance_squared(origin))
            }),
            Sort::Furthest => found.sort_by(|a, b| {
                b.position
                    .distance_squared(origin)
                    .total_cmp(&a.position.distance_squared(origin))
            }),
            Sort::Random => {
                let mut hasher = std::collections::hash_map::RandomState::new().build_hasher();
                hasher.write_u64(self.world.level.game_time as u64);
                let mut seed = hasher.finish();
                for index in (1..found.len()).rev() {
                    seed = seed
                        .wrapping_mul(6364136223846793005)
                        .wrapping_add(1442695040888963407);
                    let pick = (seed >> 33) as usize % (index + 1);
                    found.swap(index, pick);
                }
            }
        }
        found.truncate(selector.limit);
        found.into_iter().map(|c| c.target).collect()
    }

    fn apply_entity_effect(&mut self, id: EntityId, effect: EffectInstance) -> bool {
        let Some(state) = self.entities.store.get_mut(id).and_then(|e| e.living_mut()) else {
            return false;
        };
        if state.effects.apply(effect) == rspigot_domain::effect::ApplyOutcome::Unchanged {
            return false;
        }
        living::mark_effect_attributes(state, effect.effect);
        true
    }

    fn remove_entity_effects(&mut self, id: EntityId, effect: Option<EffectId>) -> bool {
        let Some(state) = self.entities.store.get_mut(id).and_then(|e| e.living_mut()) else {
            return false;
        };
        let removed: Vec<EffectId> = match effect {
            Some(effect) => state
                .effects
                .remove(effect)
                .map(|e| e.effect)
                .into_iter()
                .collect(),
            None => state
                .effects
                .clear()
                .into_iter()
                .map(|e| e.effect)
                .collect(),
        };
        for effect in &removed {
            living::mark_effect_attributes(state, *effect);
        }
        !removed.is_empty()
    }
}
