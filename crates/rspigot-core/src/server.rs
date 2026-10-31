use std::sync::Arc;
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, Sender, select};
use rspigot_domain::item::HOTBAR_START;
use rspigot_domain::player::{ClientSettings, GameProfile, PlayerId};
use rspigot_domain::text::{Component, NamedColor};
use rspigot_domain::world::{Difficulty, Dimension};
use rspigot_domain::{ChunkPos, EntityId, Identifier, Rotation, Vec3};
use rustc_hash::{FxHashMap, FxHashSet};

use crate::admin::AdminLists;
use crate::chunks::{ChunkJob, ChunkManager, ChunkWorkers, MobGeneration, Priority};
use crate::commands::{CommandTree, build_tree};
use crate::config::{GameConfig, LevelData, hashed_seed};
use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::entities::EntityWorld;
use crate::events::{
    ClientEvent, EntityKind, EquipmentSlot, GameEvent, Inbound, JoinInfo, MetadataEntry,
    MetadataValue, PlayerListEntry, Pose, SpawnInfo,
};
use crate::falling::ScheduledFalls;
use crate::player::Player;
use crate::player_io::PlayerIo;
use crate::plugins::{PluginIntro, Registry, ToPlugin};
use crate::ports::{
    ChunkGenerator, ChunkStore, ClientLink, EntityStore, PlayerStore, PluginOutbox,
};
use crate::spawning::Spawning;
use crate::status::{PlayerSample, StatusBoard};
use crate::streaming::ChunkStreamer;

pub(crate) enum Audience {
    One(PlayerId),
    All,
    Trackers(EntityId),
    TrackersExcept(EntityId, PlayerId),
    ChunkViewers(ChunkPos),
    Nearby {
        position: Vec3,
        range: f64,
        except: Option<PlayerId>,
    },
}

pub struct World {
    pub dimension: Arc<Dimension>,
    pub chunks: ChunkManager,
    pub level: LevelData,
    pub spawn: Vec3,
    pub difficulty: Difficulty,
}

struct TpsMeter {
    last_sample: Instant,
    recent: [f64; 3],
}

impl TpsMeter {
    const SAMPLE_TICKS: u64 = 100;
    const DECAY: [f64; 3] = [0.92, 0.9835, 0.9945];

    fn new() -> Self {
        Self {
            last_sample: Instant::now(),
            recent: [20.0; 3],
        }
    }

    fn record(&mut self, tick: u64) {
        if !tick.is_multiple_of(Self::SAMPLE_TICKS) {
            return;
        }
        let now = Instant::now();
        let elapsed = now
            .duration_since(self.last_sample)
            .as_secs_f64()
            .max(f64::EPSILON);
        self.last_sample = now;
        let current = (Self::SAMPLE_TICKS as f64 / elapsed).min(20.0);
        for (avg, decay) in self.recent.iter_mut().zip(Self::DECAY) {
            *avg = *avg * decay + current * (1.0 - decay);
        }
    }
}

pub struct Server<L> {
    pub(crate) config: GameConfig,
    pub(crate) world: World,
    pub(crate) workers: ChunkWorkers,
    pub(crate) players: FxHashMap<PlayerId, Player<L>>,
    pub(crate) outbox: Vec<(Audience, ClientEvent)>,
    pub(crate) admin: AdminLists,
    pub(crate) admin_sink: Option<Sender<AdminLists>>,
    pub(crate) player_io: Option<PlayerIo>,
    pub(crate) running: bool,
    pub(crate) entities: EntityWorld,
    pub(crate) falls: ScheduledFalls,
    pub(crate) rng: StdRng,
    pub(crate) spawning: Spawning,
    pub(crate) plugins: Registry,
    next_entity_id: i32,
    status: StatusBoard,
    operator_tree: Arc<CommandTree>,
    player_tree: Arc<CommandTree>,
    tick: u64,
    tps: TpsMeter,
}

impl<L: ClientLink> Server<L> {
    pub fn new<S: ChunkStore + EntityStore, G: ChunkGenerator>(
        config: GameConfig,
        mut level: LevelData,
        store: Arc<S>,
        generator: Arc<G>,
        status: StatusBoard,
    ) -> Self {
        let dimension = Arc::new(Dimension::overworld());
        let spawn = match level.spawn {
            Some(pos) => pos.center(),
            None => generator.spawn_point(&dimension),
        };
        level.spawn = Some(spawn.block());
        let workers = ChunkWorkers::spawn(config.chunk_workers, store, generator);
        let mut chunks = ChunkManager::new(Arc::clone(&dimension), config.chunks);
        chunks.set_mob_generation(level.spawn_mobs.then_some(MobGeneration {
            seed: level.seed,
            zoom_seed: hashed_seed(level.seed),
            difficulty: config.difficulty,
        }));
        let center = spawn.chunk();
        for dx in -config.spawn_chunk_radius..=config.spawn_chunk_radius {
            for dz in -config.spawn_chunk_radius..=config.spawn_chunk_radius {
                chunks.add_ticket(
                    ChunkPos::new(center.x + dx, center.z + dz),
                    Priority::URGENT,
                    &workers,
                );
            }
        }
        let admin = config.admin.clone();
        status.update(|s| s.max_players = config.max_players);
        let spawning = Spawning::new(level.seed, &dimension);
        Self {
            world: World {
                dimension,
                chunks,
                level,
                spawn,
                difficulty: config.difficulty,
            },
            workers,
            players: FxHashMap::default(),
            outbox: Vec::new(),
            admin,
            admin_sink: None,
            player_io: None,
            running: true,
            entities: EntityWorld::default(),
            falls: ScheduledFalls::default(),
            rng: StdRng::from_os_rng(),
            spawning,
            plugins: Registry::default(),
            next_entity_id: 1,
            status,
            operator_tree: Arc::new(build_tree(true)),
            player_tree: Arc::new(build_tree(false)),
            tick: 0,
            tps: TpsMeter::new(),
            config,
        }
    }

    pub fn with_player_store<S: PlayerStore>(mut self, store: Arc<S>) -> Self {
        self.player_io = Some(PlayerIo::spawn(store));
        self
    }

    pub fn with_admin_sink(mut self, sink: Sender<AdminLists>) -> Self {
        self.admin_sink = Some(sink);
        self
    }

    pub(crate) fn publish_admin(&self) {
        if let Some(sink) = &self.admin_sink {
            let _ = sink.send(self.admin.clone());
        }
    }

    pub fn wait_for_spawn(&mut self, timeout: Duration) -> usize {
        let deadline = Instant::now() + timeout;
        while self.world.chunks.stats().pending > 0 && Instant::now() < deadline {
            if let Ok(result) = self
                .workers
                .results()
                .recv_timeout(Duration::from_millis(50))
            {
                self.world.chunks.accept(result);
            }
        }
        self.world.chunks.stats().loaded
    }

    pub fn run(mut self, inbound: &Receiver<Inbound<L>>) -> LevelData {
        let tick_length = Duration::from_millis(50);
        let results = self.workers.results().clone();
        let mut next_tick = Instant::now();
        while self.running {
            let now = Instant::now();
            if now >= next_tick {
                self.tick();
                next_tick += tick_length;
                let behind = Instant::now().saturating_duration_since(next_tick);
                if behind > Duration::from_secs(2) {
                    tracing::warn!(
                        "Can't keep up! Is the server overloaded? Running {}ms or {} ticks behind",
                        behind.as_millis(),
                        behind.as_millis() / 50
                    );
                    next_tick = Instant::now();
                }
                continue;
            }
            select! {
                recv(inbound) -> message => match message {
                    Ok(message) => {
                        self.handle(message);
                        self.deliver();
                    }
                    Err(_) => self.running = false,
                },
                recv(results) -> result => {
                    if let Ok(result) = result {
                        self.world.chunks.accept(result);
                    }
                },
                default(next_tick.saturating_duration_since(now)) => {}
            }
        }
        self.shutdown()
    }

    pub fn handle(&mut self, message: Inbound<L>) {
        match message {
            Inbound::Join {
                profile,
                settings,
                link,
            } => self.join(profile, settings, link),
            Inbound::Input { player, input } => self.handle_input(player, input),
            Inbound::Leave { player } => self.remove_player(player, true),
            Inbound::Console(line) => {
                crate::commands::execute(self, crate::commands::CommandSender::Console, &line);
            }
            Inbound::PluginConnected {
                connection,
                intro,
                outbox,
            } => self.plugin_connected(connection, intro, outbox),
            Inbound::PluginMessage { .. } => {}
            Inbound::PluginDisconnected { connection } => {
                self.plugins.disconnect(connection);
            }
            Inbound::Shutdown => self.running = false,
        }
    }

    fn plugin_connected(
        &mut self,
        connection: u64,
        intro: PluginIntro,
        outbox: Box<dyn PluginOutbox>,
    ) {
        let tick = self.current_tick();
        let replaced = self.plugins.connect(connection, intro, outbox, tick);
        if let Some(old) = replaced {
            old.send(ToPlugin::Replaced);
        }
        self.plugins.send(connection, ToPlugin::Accepted);
    }

    pub(crate) fn current_tick(&self) -> u64 {
        self.tick
    }

    pub(crate) fn allocate_entity_id(&mut self) -> EntityId {
        let id = EntityId(self.next_entity_id);
        self.next_entity_id += 1;
        id
    }

    pub(crate) fn push(&mut self, audience: Audience, event: ClientEvent) {
        self.outbox.push((audience, event));
    }

    pub fn deliver(&mut self) {
        for (audience, event) in self.outbox.drain(..) {
            match audience {
                Audience::One(id) => {
                    if let Some(player) = self.players.get(&id) {
                        player.link.send(event);
                    }
                }
                Audience::All => {
                    for player in self.players.values() {
                        player.link.send(event.clone());
                    }
                }
                Audience::Trackers(entity) => {
                    for player in self
                        .players
                        .values()
                        .filter(|p| p.tracked.contains(&entity))
                    {
                        player.link.send(event.clone());
                    }
                }
                Audience::TrackersExcept(entity, excluded) => {
                    for player in self
                        .players
                        .values()
                        .filter(|p| p.id != excluded && p.tracked.contains(&entity))
                    {
                        player.link.send(event.clone());
                    }
                }
                Audience::ChunkViewers(pos) => {
                    for player in self.players.values().filter(|p| p.streamer.has_sent(pos)) {
                        player.link.send(event.clone());
                    }
                }
                Audience::Nearby {
                    position,
                    range,
                    except,
                } => {
                    for player in self.players.values().filter(|p| {
                        Some(p.id) != except
                            && p.position.distance_squared(position) < range * range
                    }) {
                        player.link.send(event.clone());
                    }
                }
            }
        }
    }

    pub(crate) fn list_entry(player: &Player<L>) -> PlayerListEntry {
        PlayerListEntry {
            profile: player.profile.clone(),
            game_mode: player.game_mode,
            latency_ms: 0,
            listed: true,
            display_name: None,
        }
    }

    pub(crate) fn spawn_info(&self, player: &Player<L>) -> SpawnInfo {
        SpawnInfo {
            dimension: (*self.world.dimension).clone(),
            hashed_seed: hashed_seed(self.world.level.seed),
            game_mode: player.game_mode,
            previous_game_mode: player.previous_game_mode,
            is_debug: false,
            is_flat: self.config.is_flat,
            death_location: None,
            portal_cooldown: 0,
        }
    }

    pub(crate) fn command_tree(&self, operator: bool) -> Arc<CommandTree> {
        if operator {
            Arc::clone(&self.operator_tree)
        } else {
            Arc::clone(&self.player_tree)
        }
    }

    fn join(&mut self, profile: GameProfile, settings: ClientSettings, link: L) {
        if self.players.contains_key(&profile.id) {
            if let Some(existing) = self.players.get(&profile.id) {
                existing.link.disconnect(Component::translatable(
                    "multiplayer.disconnect.duplicate_login",
                    Vec::new(),
                ));
            }
            self.remove_player(profile.id, true);
        }
        let operator = self.admin.is_operator(&profile);
        if !self.admin.may_join(&profile) {
            tracing::info!(
                "Disconnecting {} ({}): {}",
                profile.name,
                profile.id,
                self.config.whitelist_message
            );
            link.disconnect(Component::text(self.config.whitelist_message.clone()));
            return;
        }
        if self.players.len() as u32 >= self.config.max_players && !operator {
            link.disconnect(Component::translatable(
                "multiplayer.disconnect.server_full",
                Vec::new(),
            ));
            return;
        }
        let saved = self.load_saved_player(&profile);
        let entity_id = self.allocate_entity_id();
        let position = saved
            .as_ref()
            .filter(|save| !save.is_dead())
            .map_or(self.world.spawn, |save| save.position);
        let view_distance = self.config.view_distance.min(settings.view_distance.max(2));
        let mut player = Player {
            id: profile.id,
            profile,
            link,
            entity_id,
            position,
            rotation: Rotation::new(self.world.level.spawn_angle, 0.0),
            on_ground: false,
            game_mode: self.config.default_game_mode,
            previous_game_mode: None,
            inventory: Default::default(),
            inventory_state: 0,
            menu: Default::default(),
            drops: Default::default(),
            streamer: ChunkStreamer::new(self.config.stream, position.chunk(), view_distance),
            settings,
            pending_teleport: None,
            teleport_counter: 0,
            sneaking: false,
            sprinting: false,
            flying: false,
            tracked: FxHashSet::default(),
            loaded: false,
            operator,
            effects: Default::default(),
            vitals: Default::default(),
            known_movement: Vec3::ZERO,
            moved_this_tick: false,
            using_item: None,
            open_menu: None,
            window_counter: 0,
            ender_items: vec![None; 27],
        };
        if let Some(save) = &saved {
            save.restore(&mut player, self.world.spawn, self.world.level.spawn_angle);
        }
        tracing::info!(
            "{}[/{}] logged in with entity id {} at ([{}]{:.1}, {:.1}, {:.1})",
            player.name(),
            player.id,
            entity_id.0,
            self.world.level.name,
            position.x,
            position.y,
            position.z
        );
        self.send_join_sequence(&mut player);
        let entries: Vec<PlayerListEntry> = self
            .players
            .values()
            .map(Self::list_entry)
            .chain(std::iter::once(Self::list_entry(&player)))
            .collect();
        player.link.send(ClientEvent::PlayerInfo(entries));
        self.push(
            Audience::All,
            ClientEvent::PlayerInfo(vec![Self::list_entry(&player)]),
        );
        player
            .streamer
            .start(&mut self.world.chunks, &self.workers, &player.link);
        let name = player.profile.name.clone();
        let id = player.id;
        self.players.insert(id, player);
        self.update_player_view(id);
        if let Some(save) = &saved {
            self.announce_restored(id, save);
        }
        let joined =
            Component::translatable("multiplayer.player.joined", vec![Component::text(name)])
                .color(NamedColor::Yellow);
        tracing::info!("{}", crate::console::console_text(&joined));
        self.push(Audience::All, ClientEvent::system(joined));
        self.refresh_status();
    }

    fn send_join_sequence(&self, player: &mut Player<L>) {
        let info = JoinInfo {
            entity_id: player.entity_id,
            hardcore: self.config.hardcore,
            dimension_names: vec![
                Identifier::minecraft("overworld"),
                Identifier::minecraft("the_nether"),
                Identifier::minecraft("the_end"),
            ],
            max_players: self.config.max_players,
            view_distance: self.config.view_distance,
            simulation_distance: self.config.simulation_distance,
            reduced_debug_info: false,
            enable_respawn_screen: !self.config.rules.immediate_respawn,
            do_limited_crafting: false,
            spawn: self.spawn_info(player),
            enforces_secure_chat: false,
        };
        let link = &player.link;
        link.send(ClientEvent::Join(Box::new(info)));
        link.send(ClientEvent::Difficulty {
            difficulty: self.world.difficulty,
            locked: false,
        });
        link.send(ClientEvent::Abilities(player.abilities()));
        link.send(ClientEvent::HeldSlot(player.inventory.selected_hotbar));
        link.send(ClientEvent::RecipeSync);
        link.send(ClientEvent::OperatorLevel {
            entity: player.entity_id,
            level: if player.operator { 4 } else { 0 },
        });
        link.send(ClientEvent::Commands(self.command_tree(player.operator)));
        link.send(ClientEvent::SpawnPosition {
            dimension: self.world.dimension.key.clone(),
            pos: self.world.spawn.block(),
            yaw: self.world.level.spawn_angle,
            pitch: 0.0,
        });
        let teleport = player.next_teleport_id();
        player.link.send(ClientEvent::Teleport {
            id: teleport,
            position: player.position,
            rotation: player.rotation,
        });
        let link = &player.link;
        link.send(self.time_event());
        if self.world.level.raining {
            link.send(ClientEvent::GameEvent(GameEvent::StartRaining));
            link.send(ClientEvent::GameEvent(GameEvent::RainLevelChange(1.0)));
            if self.world.level.thundering {
                link.send(ClientEvent::GameEvent(GameEvent::ThunderLevelChange(1.0)));
            }
        }
        link.send(ClientEvent::GameEvent(GameEvent::LevelChunksLoadStart));
        let (slots, carried) = player.menu.sync_all(&player.inventory);
        link.send(ClientEvent::InventoryContent {
            state_id: 0,
            slots,
            carried,
        });
        link.send(ClientEvent::Health {
            health: player.vitals.health,
            food: player.vitals.food.food,
            saturation: player.vitals.food.saturation,
        });
        link.send(ClientEvent::Experience {
            progress: 0.0,
            level: 0,
            total: 0,
        });
        player.vitals.mark_sent();
    }

    pub(crate) fn time_event(&self) -> ClientEvent {
        ClientEvent::Time {
            game_time: self.world.level.game_time,
            day_time: self.world.level.day_time,
            advance_time: self.world.level.daylight_cycle,
        }
    }

    pub(crate) fn remove_player(&mut self, id: PlayerId, announce: bool) {
        self.close_open_menu(id, false);
        let Some(mut player) = self.players.remove(&id) else {
            return;
        };
        self.persist_player(&player);
        player.streamer.release_all(&mut self.world.chunks);
        for other in self.players.values_mut() {
            if other.tracked.remove(&player.entity_id) {
                other
                    .link
                    .send(ClientEvent::RemoveEntities(vec![player.entity_id]));
            }
        }
        self.push(Audience::All, ClientEvent::PlayerInfoRemove(vec![id]));
        if announce {
            let left = Component::translatable(
                "multiplayer.player.left",
                vec![Component::text(player.profile.name.clone())],
            )
            .color(NamedColor::Yellow);
            tracing::info!("{}", crate::console::console_text(&left));
            self.push(Audience::All, ClientEvent::system(left));
        }
        self.refresh_status();
    }

    fn refresh_status(&self) {
        let sample: Vec<PlayerSample> = self
            .players
            .values()
            .take(12)
            .map(|p| PlayerSample {
                id: p.id,
                name: p.profile.name.clone(),
            })
            .collect();
        let online = self.players.len() as u32;
        self.status.update(|s| {
            s.online = online;
            s.sample = sample;
        });
    }

    pub fn tick(&mut self) {
        self.tick += 1;
        self.tps.record(self.tick);
        self.world.level.game_time += 1;
        if self.world.level.daylight_cycle {
            self.world.level.day_time += 1;
        }
        self.world.chunks.tick(&self.workers);
        self.handle_chunk_events();
        for player in self.players.values_mut() {
            player.streamer.tick(&self.world.chunks, &player.link);
        }
        self.world.chunks.relight();
        self.send_entity_changes();
        self.tick_health();
        self.tick_spawning();
        self.tick_entities();
        self.collect_items();
        self.collect_arrows();
        self.collect_orbs();
        self.tick_furnaces();
        self.tick_campfires();
        self.tick_inventories();
        self.tick_open_menus();
        self.tick_falling_blocks();
        self.update_tracking();
        self.sync_vitals();
        if self.tick.is_multiple_of(20) {
            let time = self.time_event();
            self.push(Audience::All, time);
        }
        let disconnected: Vec<PlayerId> = self
            .players
            .values()
            .filter(|p| !p.link.is_connected())
            .map(|p| p.id)
            .collect();
        for id in disconnected {
            self.remove_player(id, true);
        }
        self.deliver();
    }

    pub(crate) fn metadata(player: &Player<L>) -> Vec<MetadataEntry> {
        vec![
            MetadataEntry {
                index: 0,
                value: MetadataValue::Byte(player.shared_flags()),
            },
            MetadataEntry {
                index: 6,
                value: MetadataValue::Pose(if player.sneaking {
                    Pose::Crouching
                } else {
                    Pose::Standing
                }),
            },
        ]
        .into_iter()
        .chain((player.vitals.arrow_count > 0).then(|| MetadataEntry {
            index: 12,
            value: MetadataValue::VarInt(player.vitals.arrow_count),
        }))
        .collect()
    }

    fn update_tracking(&mut self) {
        struct Target {
            id: PlayerId,
            entity: EntityId,
            position: Vec3,
            rotation: Rotation,
            chunk: ChunkPos,
            metadata: Vec<MetadataEntry>,
            held: Option<rspigot_domain::item::ItemStack>,
        }
        let targets: Vec<Target> = self
            .players
            .values()
            .filter(|p| !p.vitals.removed)
            .map(|p| Target {
                id: p.id,
                entity: p.entity_id,
                position: p.position,
                rotation: p.rotation,
                chunk: p.position.chunk(),
                metadata: Self::metadata(p)
                    .into_iter()
                    .chain(Self::health_metadata(p))
                    .collect(),
                held: p
                    .inventory
                    .get(HOTBAR_START + usize::from(p.inventory.selected_hotbar))
                    .cloned(),
            })
            .collect();
        let range = self.config.player_tracking_range * self.config.player_tracking_range;
        for viewer in self.players.values_mut() {
            for target in &targets {
                if target.id == viewer.id {
                    continue;
                }
                let visible = viewer.position.distance_squared(target.position) <= range
                    && viewer.streamer.has_sent(target.chunk);
                let tracking = viewer.tracked.contains(&target.entity);
                if visible && !tracking {
                    viewer.tracked.insert(target.entity);
                    viewer.link.send(ClientEvent::SpawnEntity {
                        entity: target.entity,
                        uuid: target.id.0,
                        kind: EntityKind::Player,
                        position: target.position,
                        rotation: target.rotation,
                        head_yaw: target.rotation.yaw,
                        data: 0,
                        velocity: Vec3::ZERO,
                    });
                    viewer.link.send(ClientEvent::EntityHeadRotation {
                        entity: target.entity,
                        yaw: target.rotation.yaw,
                    });
                    viewer.link.send(ClientEvent::EntityMetadata {
                        entity: target.entity,
                        entries: target.metadata.clone(),
                    });
                    if target.held.is_some() {
                        viewer.link.send(ClientEvent::EntityEquipment {
                            entity: target.entity,
                            slots: vec![(EquipmentSlot::MainHand, target.held.clone())],
                        });
                    }
                } else if !visible && tracking {
                    viewer.tracked.remove(&target.entity);
                    viewer
                        .link
                        .send(ClientEvent::RemoveEntities(vec![target.entity]));
                }
            }
        }
    }

    pub(crate) fn tps(&self) -> [f64; 3] {
        self.tps.recent
    }

    pub fn players_online(&self) -> usize {
        self.players.len()
    }

    pub fn chunk_stats(&self) -> crate::chunks::ChunkManagerStats {
        self.world.chunks.stats()
    }

    fn shutdown(mut self) -> LevelData {
        tracing::info!("Stopping server");
        let ids: Vec<PlayerId> = self.players.keys().copied().collect();
        for id in ids {
            if let Some(player) = self.players.get(&id) {
                player.link.disconnect(Component::translatable(
                    "multiplayer.disconnect.server_shutdown",
                    Vec::new(),
                ));
            }
            self.remove_player(id, false);
        }
        self.outbox.clear();
        tracing::info!(
            "Saving chunks for level '{}'/{}",
            self.world.level.name,
            self.world.dimension.key
        );
        self.handle_chunk_events();
        self.save_all_entities();
        let saved = self.world.chunks.save_dirty(&self.workers);
        self.workers.submit(Priority::BACKGROUND, ChunkJob::Flush);
        self.workers.shutdown();
        if let Some(io) = self.player_io.take() {
            io.shutdown();
        }
        let stats = self.world.chunks.stats();
        tracing::info!(
            "Saved {saved} chunks ({} loaded from disk, {} generated, {} unloaded, {} cancelled, average load {:.2} ms)",
            stats.loaded_from_disk,
            stats.generated,
            stats.unloaded,
            stats.cancelled,
            stats.average_load_time().as_secs_f64() * 1000.0
        );
        self.world.level
    }
}
