use std::borrow::Cow;

use bytes::Bytes;
use rspigot_core::commands::{
    ArgumentParser as CoreParser, CommandNodeKind as CoreNodeKind, CommandTree,
};
use rspigot_core::events::{
    Abilities, Animation, ClientEvent, EntityKind, EquipmentSlot as CoreSlot,
    GameEvent as CoreGameEvent, MetadataValue, PlayerListEntry, Pose as CorePose, SpawnInfo,
};
use rspigot_domain::chunk::{
    ChunkColumn, HeightmapKind as DomainHeightmap, LightArray as DomainLight,
};
use rspigot_domain::effect::{AttributeOperation, AttributeState, EffectInstance};
use rspigot_domain::item::{ItemStack, Slot};
use rspigot_domain::player::GameMode as DomainGameMode;
use rspigot_domain::world::Difficulty as DomainDifficulty;
use rspigot_domain::{BlockPos, ChunkPos, Rotation, Vec3};
use rspigot_protocol::codec::{Angle, Encode, Position};
use rspigot_protocol::frame::{Compression, build_frame};
use rspigot_protocol::play::chat::ChatTypeBound;
use rspigot_protocol::play::chunk::{
    BlockEntityInfo, ChunkSection as WireSection, Heightmap as WireHeightmap,
    HeightmapKind as WireHeightmap2, LevelChunkData, LightData, PaletteStrategy,
    PalettedContainer as WireContainer, write_sections,
};
use rspigot_protocol::play::clientbound as cb;
use rspigot_protocol::play::commands::{
    ArgumentParser as WireParser, CommandNode as WireNode, CommandNodeKind as WireNodeKind,
    SimpleParser, StringKind, Suggestion, parser,
};
use rspigot_protocol::play::effect::{FLAG_AMBIENT, FLAG_BLEND, FLAG_SHOW_ICON, FLAG_VISIBLE};
use rspigot_protocol::play::entity::{
    AttributeModifier as WireModifier, AttributeOperation as WireOperation, AttributeSnapshot,
    EquipmentSlot as WireSlot, PositionPath, SwingAnimationStyle, SwingAnimationType, VecDelta,
};
use rspigot_protocol::play::metadata::{EntityDataItem, EntityDataValue};
use rspigot_protocol::play::player::{CommonPlayerSpawnInfo, PlayerInfoActions, PlayerInfoEntry};
use rspigot_protocol::play::slot::ItemStack as WireStack;
use rspigot_protocol::play::types::{
    ByteBitSet, ChunkPos as WireChunkPos, Difficulty as WireDifficulty, GameMode as WireGameMode,
    GlobalPos, Hand as WireHand, Holder, LpVec3, Pose as WirePose, PositionMoveRotation, Relatives,
    Vec3 as WireVec3,
};
use rspigot_protocol::play::world::{ClockState, ClockUpdate, GameEventKind};
use rspigot_protocol::types::{ProfileProperty, Text};
use rspigot_protocol::{Packet, ids, write_packet};

use crate::cache::ChunkPacketCache;
use crate::connection::{ConnectionError, PacketWriter};
use crate::text::to_nbt;

struct EntityEvent {
    entity: i32,
    status: i8,
}

impl Encode for EntityEvent {
    fn encode(&self, out: &mut Vec<u8>) {
        self.entity.encode(out);
        self.status.encode(out);
    }
}

pub struct Encoder {
    online_mode: bool,
}

fn position(pos: BlockPos) -> Position {
    Position {
        x: pos.x,
        y: pos.y,
        z: pos.z,
    }
}

fn wire_vec(v: Vec3) -> WireVec3 {
    WireVec3 {
        x: v.x,
        y: v.y,
        z: v.z,
    }
}

fn text(component: &rspigot_domain::text::Component) -> Text {
    Text(to_nbt(component))
}

fn game_mode(mode: DomainGameMode) -> WireGameMode {
    match mode {
        DomainGameMode::Survival => WireGameMode::Survival,
        DomainGameMode::Creative => WireGameMode::Creative,
        DomainGameMode::Adventure => WireGameMode::Adventure,
        DomainGameMode::Spectator => WireGameMode::Spectator,
    }
}

fn difficulty(d: DomainDifficulty) -> WireDifficulty {
    match d {
        DomainDifficulty::Peaceful => WireDifficulty::Peaceful,
        DomainDifficulty::Easy => WireDifficulty::Easy,
        DomainDifficulty::Normal => WireDifficulty::Normal,
        DomainDifficulty::Hard => WireDifficulty::Hard,
    }
}

pub fn stack(slot: &Slot) -> WireStack {
    match slot {
        Some(ItemStack { item, count, .. }) => {
            WireStack::new(i32::from(item.0), i32::from(count.get()))
        }
        None => WireStack::EMPTY,
    }
}

fn spawn_info(spawn: &SpawnInfo) -> CommonPlayerSpawnInfo {
    CommonPlayerSpawnInfo {
        dimension_type: rspigot_data::registries::dimension_type_id(
            spawn.dimension.dimension_type.as_str(),
        )
        .map_or(0, |id| id as i32),
        dimension: spawn.dimension.key.to_string(),
        seed: spawn.hashed_seed,
        game_mode: game_mode(spawn.game_mode),
        previous_game_mode: spawn.previous_game_mode.map(game_mode),
        is_debug: spawn.is_debug,
        is_flat: spawn.is_flat,
        last_death_location: spawn
            .death_location
            .as_ref()
            .map(|(dimension, pos)| GlobalPos {
                dimension: dimension.to_string(),
                pos: position(*pos),
            }),
        portal_cooldown: spawn.portal_cooldown,
        sea_level: spawn.dimension.sea_level,
    }
}

fn info_entry(entry: &PlayerListEntry) -> PlayerInfoEntry {
    PlayerInfoEntry {
        profile_id: entry.profile.id.0,
        name: entry.profile.name.clone(),
        properties: entry
            .profile
            .properties
            .iter()
            .map(|p| ProfileProperty {
                name: p.name.clone(),
                value: p.value.clone(),
                signature: p.signature.clone(),
            })
            .collect(),
        chat_session: None,
        game_mode: game_mode(entry.game_mode),
        listed: entry.listed,
        latency: entry.latency_ms,
        display_name: entry.display_name.as_ref().map(text),
        list_order: 0,
        show_hat: true,
    }
}

fn abilities(a: &Abilities) -> cb::PlayerAbilities {
    cb::PlayerAbilities {
        invulnerable: a.invulnerable,
        flying: a.flying,
        can_fly: a.may_fly,
        instabuild: a.instant_build,
        flying_speed: a.fly_speed,
        walking_speed: a.walk_speed,
    }
}

fn entity_type(kind: EntityKind) -> i32 {
    let name = match kind {
        EntityKind::Player => "minecraft:player",
        EntityKind::Item => "minecraft:item",
        EntityKind::Other(id) => return i32::from(id),
    };
    rspigot_data::registries::static_index("minecraft:entity_type", name).map_or(0, |id| id as i32)
}

fn pose(p: CorePose) -> WirePose {
    match p {
        CorePose::Standing => WirePose::Standing,
        CorePose::FallFlying => WirePose::FallFlying,
        CorePose::Sleeping => WirePose::Sleeping,
        CorePose::Swimming => WirePose::Swimming,
        CorePose::SpinAttack => WirePose::SpinAttack,
        CorePose::Crouching => WirePose::Crouching,
        CorePose::LongJumping => WirePose::LongJumping,
        CorePose::Dying => WirePose::Dying,
    }
}

fn equipment_slot(slot: CoreSlot) -> WireSlot {
    match slot {
        CoreSlot::MainHand => WireSlot::MainHand,
        CoreSlot::OffHand => WireSlot::OffHand,
        CoreSlot::Feet => WireSlot::Feet,
        CoreSlot::Legs => WireSlot::Legs,
        CoreSlot::Chest => WireSlot::Chest,
        CoreSlot::Head => WireSlot::Head,
        CoreSlot::Body => WireSlot::Body,
        CoreSlot::Saddle => WireSlot::Saddle,
    }
}

fn simple(id: i32) -> WireParser {
    SimpleParser::new(id).map_or(
        WireParser::String(StringKind::SingleWord),
        WireParser::Simple,
    )
}

fn parser(p: &CoreParser) -> WireParser {
    match p {
        CoreParser::Bool => simple(parser::BOOL),
        CoreParser::Integer { min, max } => WireParser::Integer {
            min: *min,
            max: *max,
        },
        CoreParser::Float { min, max } => WireParser::Float {
            min: *min,
            max: *max,
        },
        CoreParser::Double { min, max } => WireParser::Double {
            min: *min,
            max: *max,
        },
        CoreParser::Word => WireParser::String(StringKind::SingleWord),
        CoreParser::QuotedString => WireParser::String(StringKind::QuotablePhrase),
        CoreParser::GreedyString => WireParser::String(StringKind::GreedyPhrase),
        CoreParser::Entity {
            single,
            players_only,
        } => WireParser::Entity {
            single: *single,
            players_only: *players_only,
        },
        CoreParser::GameProfile => simple(parser::GAME_PROFILE),
        CoreParser::BlockPos => simple(parser::BLOCK_POS),
        CoreParser::Vec3 => simple(parser::VEC3),
        CoreParser::BlockState => simple(parser::BLOCK_STATE),
        CoreParser::ItemStack => simple(parser::ITEM_STACK),
        CoreParser::Message => simple(parser::MESSAGE),
        CoreParser::Time { min } => WireParser::Time { min: *min },
        CoreParser::GameMode => simple(parser::GAMEMODE),
        CoreParser::Dimension => simple(parser::DIMENSION),
        CoreParser::ResourceLocation => simple(parser::RESOURCE_LOCATION),
        CoreParser::Resource { registry } => WireParser::Resource {
            registry: registry.clone(),
        },
    }
}

fn commands(tree: &CommandTree) -> cb::Commands {
    cb::Commands {
        nodes: tree
            .nodes
            .iter()
            .map(|node| WireNode {
                kind: match &node.kind {
                    CoreNodeKind::Root => WireNodeKind::Root,
                    CoreNodeKind::Literal(name) => WireNodeKind::Literal { name: name.clone() },
                    CoreNodeKind::Argument {
                        name,
                        parser: p,
                        suggests,
                    } => WireNodeKind::Argument {
                        name: name.clone(),
                        parser: parser(p),
                        suggestions: suggests.clone(),
                    },
                },
                executable: node.executable,
                restricted: false,
                children: node.children.iter().map(|c| *c as i32).collect(),
                redirect: node.redirect.map(|r| r as i32),
            })
            .collect(),
        root_index: tree.root as i32,
    }
}

fn game_event(event: CoreGameEvent) -> cb::GameEvent {
    let (kind, param) = match event {
        CoreGameEvent::NoRespawnBlockAvailable => (GameEventKind::NoRespawnBlockAvailable, 0.0),
        CoreGameEvent::StartRaining => (GameEventKind::StartRaining, 0.0),
        CoreGameEvent::StopRaining => (GameEventKind::StopRaining, 0.0),
        CoreGameEvent::ChangeGameMode(mode) => {
            (GameEventKind::ChangeGameMode, f32::from(mode.id()))
        }
        CoreGameEvent::WinGame => (GameEventKind::WinGame, 0.0),
        CoreGameEvent::DemoEvent => (GameEventKind::DemoEvent, 0.0),
        CoreGameEvent::ArrowHitPlayer => (GameEventKind::PlayArrowHitSound, 0.0),
        CoreGameEvent::RainLevelChange(v) => (GameEventKind::RainLevelChange, v),
        CoreGameEvent::ThunderLevelChange(v) => (GameEventKind::ThunderLevelChange, v),
        CoreGameEvent::PufferFishSting => (GameEventKind::PufferFishSting, 0.0),
        CoreGameEvent::GuardianElderEffect => (GameEventKind::GuardianElderEffect, 0.0),
        CoreGameEvent::ImmediateRespawn(on) => {
            (GameEventKind::ImmediateRespawn, if on { 1.0 } else { 0.0 })
        }
        CoreGameEvent::LimitedCrafting(on) => {
            (GameEventKind::LimitedCrafting, if on { 1.0 } else { 0.0 })
        }
        CoreGameEvent::LevelChunksLoadStart => (GameEventKind::LevelChunksLoadStart, 0.0),
    };
    cb::GameEvent { event: kind, param }
}

fn fluid_count(section: &rspigot_domain::chunk::ChunkSection) -> i16 {
    let blocks = section.blocks();
    let fluid_slots: Vec<bool> = blocks
        .palette()
        .iter()
        .map(|state| state.flags().has_fluid())
        .collect();
    if !fluid_slots.iter().any(|f| *f) {
        return 0;
    }
    if blocks.single_value().is_some() {
        return 4096;
    }
    (0..4096)
        .filter(|i| fluid_slots[blocks.palette_index(*i)])
        .count() as i16
}

fn container_for<T: Copy + Eq, S: rspigot_domain::chunk::PaletteStrategy>(
    container: &rspigot_domain::chunk::PalettedContainer<T, S>,
    strategy: &PaletteStrategy,
    id: impl Fn(T) -> u32,
    palette: &mut Vec<u32>,
) -> WireContainer<'static> {
    if let Some(value) = container.single_value() {
        return WireContainer::single(id(value));
    }
    let bits = container.bits();
    if bits <= strategy.max_indirect_bits && bits >= strategy.min_indirect_bits {
        palette.clear();
        palette.extend(container.palette().iter().map(|v| id(*v)));
        return WireContainer::Indirect {
            bits,
            palette: Cow::Owned(std::mem::take(palette)),
            packed: Cow::Owned(container.packed().to_vec()),
        };
    }
    let entries: Vec<u32> = container.values().map(id).collect();
    WireContainer::from_entries(strategy, &entries)
}

fn light_data(light: &[Option<DomainLight>]) -> (ByteBitSet, ByteBitSet, Vec<Cow<'_, [u8; 2048]>>) {
    let mut present = Vec::new();
    let mut empty = Vec::new();
    let mut arrays = Vec::new();
    for (index, layer) in light.iter().enumerate() {
        match layer {
            Some(array) if !array.is_uniform(0) => {
                present.push(index);
                arrays.push(Cow::Borrowed(array.as_bytes()));
            }
            _ => empty.push(index),
        }
    }
    (
        ByteBitSet::from_bits(present),
        ByteBitSet::from_bits(empty),
        arrays,
    )
}

fn heightmap_kind(kind: DomainHeightmap) -> WireHeightmap2 {
    match kind {
        DomainHeightmap::WorldSurfaceWg => WireHeightmap2::WorldSurfaceWorldgen,
        DomainHeightmap::WorldSurface => WireHeightmap2::WorldSurface,
        DomainHeightmap::OceanFloorWg => WireHeightmap2::OceanFloorWorldgen,
        DomainHeightmap::OceanFloor => WireHeightmap2::OceanFloor,
        DomainHeightmap::MotionBlocking => WireHeightmap2::MotionBlocking,
        DomainHeightmap::MotionBlockingNoLeaves => WireHeightmap2::MotionBlockingNoLeaves,
    }
}

pub fn chunk_packet(chunk: &ChunkColumn, out: &mut Vec<u8>) {
    let blocks = PaletteStrategy::blocks(rspigot_blocks::STATE_COUNT as u32);
    let biome_count = rspigot_data::registries::synced_entries("minecraft:worldgen/biome")
        .map_or(1, <[&str]>::len) as u32;
    let biomes = PaletteStrategy::biomes(biome_count);
    let mut scratch = Vec::new();
    let sections: Vec<WireSection<'static>> = chunk
        .sections()
        .iter()
        .map(|section| WireSection {
            non_empty_block_count: section.non_air_count() as i16,
            fluid_count: fluid_count(section),
            blocks: container_for(section.blocks(), &blocks, |s| u32::from(s.0), &mut scratch),
            biomes: container_for(section.biomes(), &biomes, |b| u32::from(b.0), &mut scratch),
        })
        .collect();
    let mut data = Vec::with_capacity(16 * 1024);
    write_sections(&sections, &mut data);
    let height = chunk.height().height;
    let heightmaps = chunk
        .heightmaps
        .maps
        .iter()
        .filter(|(kind, _)| DomainHeightmap::CLIENT.contains(kind))
        .map(|(kind, map)| WireHeightmap {
            kind: heightmap_kind(*kind),
            data: Cow::Owned(map.pack(height).into_iter().map(|w| w as i64).collect()),
        })
        .collect();
    let block_entities: Vec<BlockEntityInfo> = chunk
        .block_entities
        .iter()
        .filter_map(|entity| {
            let kind = rspigot_data::registries::static_index(
                "minecraft:block_entity_type",
                entity.kind.as_str(),
            )?;
            Some(BlockEntityInfo {
                packed_xz: (((entity.pos.x & 15) << 4) | (entity.pos.z & 15)) as u8,
                y: entity.pos.y as i16,
                kind: kind as i32,
                data: Some(rspigot_nbt::Tag::Compound(entity.data.clone())),
            })
        })
        .collect();
    let (sky_y_mask, empty_sky_y_mask, sky_updates) = light_data(&chunk.light.sky);
    let (block_y_mask, empty_block_y_mask, block_updates) = light_data(&chunk.light.block);
    let packet = cb::LevelChunkWithLight {
        x: chunk.pos().x,
        z: chunk.pos().z,
        chunk: LevelChunkData {
            heightmaps,
            data: Cow::Owned(data),
            block_entities: Cow::Owned(block_entities),
        },
        light: LightData {
            sky_y_mask,
            block_y_mask,
            empty_sky_y_mask,
            empty_block_y_mask,
            sky_updates,
            block_updates,
        },
    };
    write_packet(&packet, out);
}

fn effect_flags(effect: &EffectInstance, blend: bool) -> u8 {
    let mut flags = 0;
    if effect.ambient {
        flags |= FLAG_AMBIENT;
    }
    if effect.visible {
        flags |= FLAG_VISIBLE;
    }
    if effect.show_icon {
        flags |= FLAG_SHOW_ICON;
    }
    if blend {
        flags |= FLAG_BLEND;
    }
    flags
}

fn attribute_snapshot(state: &AttributeState) -> Option<AttributeSnapshot> {
    let attribute = rspigot_data::registries::static_index("minecraft:attribute", state.attribute)?;
    Some(AttributeSnapshot {
        attribute: attribute as i32,
        base: state.base,
        modifiers: state
            .modifiers
            .iter()
            .map(|m| WireModifier {
                id: m.id.to_owned(),
                amount: m.amount,
                operation: match m.operation {
                    AttributeOperation::AddValue => WireOperation::AddValue,
                    AttributeOperation::AddMultipliedBase => WireOperation::AddMultipliedBase,
                    AttributeOperation::AddMultipliedTotal => WireOperation::AddMultipliedTotal,
                },
            })
            .collect(),
    })
}

fn fixed(value: f64) -> i64 {
    (value * 4096.0).round() as i64
}

fn delta(from: Vec3, to: Vec3) -> Option<VecDelta> {
    let dx = i16::try_from(fixed(to.x) - fixed(from.x)).ok()?;
    let dy = i16::try_from(fixed(to.y) - fixed(from.y)).ok()?;
    let dz = i16::try_from(fixed(to.z) - fixed(from.z)).ok()?;
    Some(VecDelta::Linear {
        xa: dx,
        ya: dy,
        za: dz,
    })
}

impl Encoder {
    pub fn new(online_mode: bool) -> Self {
        Self { online_mode }
    }

    fn queue<P: Packet>(writer: &mut PacketWriter, packet: &P) -> Result<(), ConnectionError> {
        writer.queue(P::ID, packet)
    }

    fn chunk(
        writer: &mut PacketWriter,
        dimension: &rspigot_domain::Identifier,
        chunk: &ChunkColumn,
        cache: &ChunkPacketCache,
        compression: Option<Compression>,
    ) -> Result<(), ConnectionError> {
        let key = (dimension.clone(), chunk.pos(), chunk.revision());
        let frame = cache.get_or_build(key, || {
            let mut body = Vec::with_capacity(32 * 1024);
            chunk_packet(chunk, &mut body);
            let mut frame = Vec::with_capacity(body.len() / 2 + 8);
            build_frame(&body, compression, &mut frame).map(|()| Bytes::from(frame))
        })?;
        writer.queue_prebuilt(&frame);
        Ok(())
    }

    pub fn encode(
        &mut self,
        event: &ClientEvent,
        writer: &mut PacketWriter,
        cache: &ChunkPacketCache,
        threshold: Option<usize>,
        level: u32,
    ) -> Result<(), ConnectionError> {
        match event {
            ClientEvent::Chunk { dimension, chunk } => {
                let compression = threshold.map(|threshold| Compression { threshold, level });
                Self::chunk(writer, dimension, chunk, cache, compression)
            }
            ClientEvent::Join(info) => Self::queue(
                writer,
                &cb::Login {
                    player_id: info.entity_id.0,
                    hardcore: info.hardcore,
                    levels: info
                        .dimension_names
                        .iter()
                        .map(ToString::to_string)
                        .collect(),
                    max_players: info.max_players as i32,
                    chunk_radius: i32::from(info.view_distance),
                    simulation_distance: i32::from(info.simulation_distance),
                    reduced_debug_info: info.reduced_debug_info,
                    show_death_screen: info.enable_respawn_screen,
                    do_limited_crafting: info.do_limited_crafting,
                    spawn_info: spawn_info(&info.spawn),
                    online_mode: self.online_mode,
                    enforces_secure_chat: info.enforces_secure_chat,
                },
            ),
            ClientEvent::Respawn { spawn, keep_data } => Self::queue(
                writer,
                &cb::Respawn {
                    spawn_info: spawn_info(spawn),
                    data_to_keep: *keep_data,
                },
            ),
            ClientEvent::Difficulty {
                difficulty: d,
                locked,
            } => Self::queue(
                writer,
                &cb::ChangeDifficulty {
                    difficulty: difficulty(*d),
                    locked: *locked,
                },
            ),
            ClientEvent::Abilities(a) => Self::queue(writer, &abilities(a)),
            ClientEvent::HeldSlot(slot) => Self::queue(
                writer,
                &cb::SetHeldSlot {
                    slot: i32::from(*slot),
                },
            ),
            ClientEvent::SpawnPosition {
                dimension,
                pos,
                yaw,
                pitch,
            } => Self::queue(
                writer,
                &cb::SetDefaultSpawnPosition {
                    pos: GlobalPos {
                        dimension: dimension.to_string(),
                        pos: position(*pos),
                    },
                    yaw: *yaw,
                    pitch: *pitch,
                },
            ),
            ClientEvent::Teleport {
                id,
                position: target,
                rotation,
            } => Self::queue(
                writer,
                &cb::PlayerPosition {
                    teleport_id: *id,
                    change: PositionMoveRotation {
                        position: wire_vec(*target),
                        delta: WireVec3::ZERO,
                        y_rot: rotation.yaw,
                        x_rot: rotation.pitch,
                    },
                    relatives: Relatives::NONE,
                },
            ),
            ClientEvent::ChunkBatchStart => Self::queue(writer, &cb::ChunkBatchStart),
            ClientEvent::ChunkBatchFinished { count } => Self::queue(
                writer,
                &cb::ChunkBatchFinished {
                    batch_size: *count as i32,
                },
            ),
            ClientEvent::ForgetChunk(pos) => Self::queue(
                writer,
                &cb::ForgetLevelChunk {
                    pos: WireChunkPos { x: pos.x, z: pos.z },
                },
            ),
            ClientEvent::CenterChunk(ChunkPos { x, z }) => {
                Self::queue(writer, &cb::SetChunkCacheCenter { x: *x, z: *z })
            }
            ClientEvent::ViewDistance(radius) => Self::queue(
                writer,
                &cb::SetChunkCacheRadius {
                    radius: i32::from(*radius),
                },
            ),
            ClientEvent::SimulationDistance(distance) => Self::queue(
                writer,
                &cb::SetSimulationDistance {
                    simulation_distance: i32::from(*distance),
                },
            ),
            ClientEvent::BlockUpdate { pos, state } => Self::queue(
                writer,
                &cb::BlockUpdate {
                    pos: position(*pos),
                    state: i32::from(state.0),
                },
            ),
            ClientEvent::BlockChangedAck(sequence) => Self::queue(
                writer,
                &cb::BlockChangedAck {
                    sequence: *sequence,
                },
            ),
            ClientEvent::BlockDestruction { entity, pos, stage } => Self::queue(
                writer,
                &cb::BlockDestruction {
                    id: entity.0,
                    pos: position(*pos),
                    progress: *stage as u8,
                },
            ),
            ClientEvent::GameEvent(event) => Self::queue(writer, &game_event(*event)),
            ClientEvent::Time {
                game_time,
                day_time,
                advance_time,
            } => {
                let clock = rspigot_data::registries::synced_index(
                    "minecraft:world_clock",
                    "minecraft:overworld",
                )
                .map_or(0, |id| id as i32);
                Self::queue(
                    writer,
                    &cb::SetTime {
                        game_time: *game_time,
                        clocks: vec![ClockUpdate {
                            clock,
                            state: ClockState {
                                total_ticks: *day_time,
                                partial_tick: 0.0,
                                rate: if *advance_time { 1.0 } else { 0.0 },
                            },
                        }],
                    },
                )
            }
            ClientEvent::SystemChat { message, overlay } => Self::queue(
                writer,
                &cb::SystemChat {
                    content: text(message),
                    overlay: *overlay,
                },
            ),
            ClientEvent::PlayerChat {
                sender_name,
                message,
                ..
            } => {
                let chat =
                    rspigot_data::registries::synced_index("minecraft:chat_type", "minecraft:chat")
                        .map_or(0, |id| id as i32);
                Self::queue(
                    writer,
                    &cb::DisguisedChat {
                        message: Text(rspigot_nbt::Tag::String(message.clone())),
                        chat_type: ChatTypeBound {
                            chat_type: Holder::Reference(chat),
                            name: text(sender_name),
                            target_name: None,
                        },
                    },
                )
            }
            ClientEvent::PlayerInfo(entries) => Self::queue(
                writer,
                &cb::PlayerInfoUpdate {
                    actions: PlayerInfoActions::ADD_PLAYER
                        .union(PlayerInfoActions::UPDATE_GAME_MODE)
                        .union(PlayerInfoActions::UPDATE_LISTED)
                        .union(PlayerInfoActions::UPDATE_LATENCY)
                        .union(PlayerInfoActions::UPDATE_HAT),
                    entries: entries.iter().map(info_entry).collect(),
                },
            ),
            ClientEvent::PlayerInfoGameMode {
                player,
                game_mode: mode,
            } => Self::queue(
                writer,
                &cb::PlayerInfoUpdate {
                    actions: PlayerInfoActions::UPDATE_GAME_MODE,
                    entries: vec![PlayerInfoEntry {
                        profile_id: player.0,
                        name: String::new(),
                        properties: Vec::new(),
                        chat_session: None,
                        game_mode: game_mode(*mode),
                        listed: true,
                        latency: 0,
                        display_name: None,
                        list_order: 0,
                        show_hat: true,
                    }],
                },
            ),
            ClientEvent::PlayerInfoRemove(ids) => Self::queue(
                writer,
                &cb::PlayerInfoRemove {
                    profile_ids: ids.iter().map(|id| id.0).collect(),
                },
            ),
            ClientEvent::TabList { header, footer } => Self::queue(
                writer,
                &cb::TabList {
                    header: text(header),
                    footer: text(footer),
                },
            ),
            ClientEvent::SpawnEntity {
                entity,
                uuid,
                kind,
                position: at,
                rotation,
                head_yaw,
                data,
                velocity,
            } => Self::queue(
                writer,
                &cb::AddEntity {
                    entity_id: entity.0,
                    uuid: *uuid,
                    kind: entity_type(*kind),
                    x: at.x,
                    y: at.y,
                    z: at.z,
                    movement: LpVec3(wire_vec(*velocity)),
                    x_rot: Angle::from_degrees(rotation.pitch),
                    y_rot: Angle::from_degrees(rotation.yaw),
                    y_head_rot: Angle::from_degrees(*head_yaw),
                    data: *data,
                },
            ),
            ClientEvent::EntityMoved {
                entity,
                from,
                to,
                rotation,
                on_ground,
            } => Self::entity_moved(writer, entity.0, *from, *to, *rotation, *on_ground),
            ClientEvent::EntityVelocity { entity, velocity } => Self::queue(
                writer,
                &cb::SetEntityMotion {
                    entity_id: entity.0,
                    movement: LpVec3(wire_vec(*velocity)),
                },
            ),
            ClientEvent::EntityHeadRotation { entity, yaw } => Self::queue(
                writer,
                &cb::RotateHead {
                    entity_id: entity.0,
                    y_head_rot: Angle::from_degrees(*yaw),
                },
            ),
            ClientEvent::EntityAnimation { entity, animation } => match animation {
                Animation::SwingMainArm | Animation::SwingOffHand => Self::queue(
                    writer,
                    &cb::SwingAnimation {
                        entity_id: entity.0,
                        hand: if *animation == Animation::SwingMainArm {
                            WireHand::MainHand
                        } else {
                            WireHand::OffHand
                        },
                        animation: SwingAnimationStyle {
                            kind: SwingAnimationType::Whack,
                            duration: 6,
                        },
                    },
                ),
                other => Self::queue(
                    writer,
                    &cb::Animate {
                        entity_id: entity.0,
                        action: match other {
                            Animation::WakeUp => 0,
                            Animation::CriticalHit => 1,
                            _ => 2,
                        },
                    },
                ),
            },
            ClientEvent::EntityMetadata { entity, entries } => Self::queue(
                writer,
                &cb::SetEntityData {
                    entity_id: entity.0,
                    items: entries
                        .iter()
                        .map(|entry| EntityDataItem {
                            index: entry.index,
                            value: match &entry.value {
                                MetadataValue::Byte(v) => EntityDataValue::Byte(*v),
                                MetadataValue::VarInt(v) => EntityDataValue::Int(*v),
                                MetadataValue::Float(v) => EntityDataValue::Float(*v),
                                MetadataValue::Pose(p) => EntityDataValue::Pose(pose(*p)),
                                MetadataValue::OptionalComponent(c) => {
                                    EntityDataValue::OptionalComponent(c.as_ref().map(text))
                                }
                                MetadataValue::Boolean(b) => EntityDataValue::Boolean(*b),
                                MetadataValue::Item(item) => {
                                    EntityDataValue::ItemStack(stack(item))
                                }
                            },
                        })
                        .collect(),
                },
            ),
            ClientEvent::EntityEquipment { entity, slots } => Self::queue(
                writer,
                &cb::SetEquipment {
                    entity_id: entity.0,
                    slots: slots
                        .iter()
                        .map(|(slot, item)| (equipment_slot(*slot), stack(item)))
                        .collect(),
                },
            ),
            ClientEvent::RemoveEntities(entities) => Self::queue(
                writer,
                &cb::RemoveEntities {
                    entity_ids: entities.iter().map(|e| e.0).collect(),
                },
            ),
            ClientEvent::InventoryContent {
                state_id,
                slots,
                carried,
            } => Self::queue(
                writer,
                &cb::ContainerSetContent {
                    container_id: 0,
                    state_id: *state_id,
                    items: slots.iter().map(stack).collect(),
                    carried: stack(carried),
                },
            ),
            ClientEvent::InventorySlot {
                state_id,
                slot,
                stack: item,
            } => Self::queue(
                writer,
                &cb::ContainerSetSlot {
                    container_id: 0,
                    state_id: *state_id,
                    slot: *slot,
                    item: stack(item),
                },
            ),
            ClientEvent::Health {
                health,
                food,
                saturation,
            } => Self::queue(
                writer,
                &cb::SetHealth {
                    health: *health,
                    food: *food,
                    saturation: *saturation,
                },
            ),
            ClientEvent::Experience {
                progress,
                level,
                total,
            } => Self::queue(
                writer,
                &cb::SetExperience {
                    progress: *progress,
                    level: *level,
                    total: *total,
                },
            ),
            ClientEvent::Commands(tree) => Self::queue(writer, &commands(tree)),
            ClientEvent::CommandSuggestions {
                id,
                start,
                length,
                matches,
            } => Self::queue(
                writer,
                &cb::CommandSuggestions {
                    id: *id,
                    start: *start,
                    length: *length,
                    suggestions: matches
                        .iter()
                        .map(|m| Suggestion {
                            text: m.clone(),
                            tooltip: None,
                        })
                        .collect(),
                },
            ),
            ClientEvent::ItemPickup {
                collected,
                collector,
                amount,
            } => Self::queue(
                writer,
                &cb::TakeItemEntity {
                    item_id: collected.0,
                    player_id: collector.0,
                    amount: *amount,
                },
            ),
            ClientEvent::OperatorLevel { entity, level } => writer.queue(
                ids::play::clientbound::ENTITY_EVENT,
                &EntityEvent {
                    entity: entity.0,
                    status: 24 + (*level).min(4) as i8,
                },
            ),
            ClientEvent::EffectApplied {
                entity,
                effect,
                blend,
            } => Self::queue(
                writer,
                &cb::UpdateMobEffect {
                    entity_id: entity.0,
                    effect: i32::from(effect.effect.0),
                    amplifier: i32::from(effect.amplifier),
                    duration: effect.duration.wire_ticks(),
                    flags: effect_flags(effect, *blend),
                },
            ),
            ClientEvent::EffectRemoved { entity, effect } => Self::queue(
                writer,
                &cb::RemoveMobEffect {
                    entity_id: entity.0,
                    effect: i32::from(effect.0),
                },
            ),
            ClientEvent::Attributes { entity, attributes } => Self::queue(
                writer,
                &cb::UpdateAttributes {
                    entity_id: entity.0,
                    attributes: attributes.iter().filter_map(attribute_snapshot).collect(),
                },
            ),
            ClientEvent::Disconnect(_) => Ok(()),
        }
    }

    fn entity_moved(
        writer: &mut PacketWriter,
        entity: i32,
        from: Vec3,
        to: Vec3,
        rotation: Option<Rotation>,
        on_ground: bool,
    ) -> Result<(), ConnectionError> {
        let moved = fixed(from.x) != fixed(to.x)
            || fixed(from.y) != fixed(to.y)
            || fixed(from.z) != fixed(to.z);
        match (moved, delta(from, to), rotation) {
            (false, _, Some(rotation)) => Self::queue(
                writer,
                &cb::MoveEntityRot {
                    entity_id: entity,
                    on_ground,
                    y_rot: Angle::from_degrees(rotation.yaw),
                    x_rot: Angle::from_degrees(rotation.pitch),
                },
            ),
            (false, _, None) => Ok(()),
            (true, Some(step), None) => Self::queue(
                writer,
                &cb::MoveEntityPos {
                    entity_id: entity,
                    on_ground,
                    delta: step,
                },
            ),
            (true, Some(step), Some(rotation)) => Self::queue(
                writer,
                &cb::MoveEntityPosRot {
                    entity_id: entity,
                    on_ground,
                    delta: step,
                    y_rot: Angle::from_degrees(rotation.yaw),
                    x_rot: Angle::from_degrees(rotation.pitch),
                },
            ),
            (true, None, rotation) => {
                let rotation = rotation.unwrap_or_default();
                Self::queue(
                    writer,
                    &cb::EntityPositionSync {
                        entity_id: entity,
                        position: PositionPath::Linear(wire_vec(to)),
                        y_rot: rotation.yaw,
                        x_rot: rotation.pitch,
                        on_ground,
                    },
                )
            }
        }
    }
}
