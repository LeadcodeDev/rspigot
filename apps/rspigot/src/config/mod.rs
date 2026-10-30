mod properties;
mod yaml;

use std::path::Path;
use std::time::Duration;

use rspigot_core::admin::AdminLists;
use rspigot_core::chunks::ChunkManagerConfig;
use rspigot_core::config::{ActivationRanges, GameConfig};
use rspigot_core::streaming::StreamConfig;
use rspigot_domain::entity::TrackingRanges;
use rspigot_domain::player::GameMode;
use rspigot_domain::world::Difficulty;
use rspigot_network::NetworkConfig;

pub use properties::Properties;
use yaml::{BUKKIT_DEFAULTS, SPIGOT_DEFAULTS, YamlConfig};

pub struct ServerFiles {
    pub properties: Properties,
    pub spigot: YamlConfig,
    pub bukkit: YamlConfig,
}

impl ServerFiles {
    pub fn load(root: &Path) -> anyhow::Result<Self> {
        Ok(Self {
            properties: Properties::load_or_create(&root.join("server.properties"))?,
            spigot: YamlConfig::load_or_create(&root.join("spigot.yml"), SPIGOT_DEFAULTS)?,
            bukkit: YamlConfig::load_or_create(&root.join("bukkit.yml"), BUKKIT_DEFAULTS)?,
        })
    }

    fn distance(&self, spigot_key: &str, property: &str) -> u8 {
        let base = self.properties.int(property, 10);
        let value = match self.spigot.string(spigot_key, "default").parse::<i64>() {
            Ok(v) if v > 0 => v,
            _ => base,
        };
        value.clamp(2, 32) as u8
    }

    pub fn game_config(&self, admin: AdminLists) -> GameConfig {
        let p = &self.properties;
        GameConfig {
            max_players: p.int("max-players", 20).clamp(0, i64::from(i32::MAX)) as u32,
            view_distance: self.distance("world-settings.default.view-distance", "view-distance"),
            simulation_distance: self.distance(
                "world-settings.default.simulation-distance",
                "simulation-distance",
            ),
            default_game_mode: GameMode::from_name(p.string("gamemode"))
                .unwrap_or(GameMode::Survival),
            force_game_mode: p.bool("force-gamemode"),
            difficulty: Difficulty::from_name(p.string("difficulty")).unwrap_or(Difficulty::Easy),
            hardcore: p.bool("hardcore"),
            pvp: true,
            allow_flight: p.bool("allow-flight"),
            spawn_protection: p.int("spawn-protection", 16).max(0) as u32,
            player_tracking_range: self.spigot.float(
                "world-settings.default.entity-tracking-range.players",
                128.0,
            ),
            tracking: self.tracking_ranges(),
            activation: self.activation_ranges(),
            stream: StreamConfig::default(),
            chunks: ChunkManagerConfig {
                autosave_interval_ticks: self.bukkit.int("ticks-per.autosave", 6000).max(0) as u64,
                ..ChunkManagerConfig::default()
            },
            admin,
            whitelist_message: self.spigot.string(
                "messages.whitelist",
                "You are not whitelisted on this server!",
            ),
            is_flat: p.string("level-type").ends_with("flat"),
            ..GameConfig::default()
        }
    }

    fn tracking_ranges(&self) -> TrackingRanges {
        let defaults = TrackingRanges::default();
        let range = |key: &str, fallback: i32| {
            self.spigot
                .int(
                    &format!("world-settings.default.{key}"),
                    i64::from(fallback),
                )
                .clamp(0, i64::from(i32::MAX)) as i32
        };
        TrackingRanges {
            players: range("entity-tracking-range.players", defaults.players),
            animals: range("entity-tracking-range.animals", defaults.animals),
            monsters: range("entity-tracking-range.monsters", defaults.monsters),
            misc: range("entity-tracking-range.misc", defaults.misc),
            display: range("entity-tracking-range.display", defaults.display),
            other: range("entity-tracking-range.other", defaults.other),
            monster_activation: range(
                "entity-activation-range.monsters",
                defaults.monster_activation,
            ),
        }
    }

    fn activation_ranges(&self) -> ActivationRanges {
        let defaults = ActivationRanges::default();
        let range = |key: &str, fallback: i32| {
            self.spigot
                .int(
                    &format!("world-settings.default.entity-activation-range.{key}"),
                    i64::from(fallback),
                )
                .clamp(0, i64::from(i32::MAX)) as i32
        };
        ActivationRanges {
            animals: range("animals", defaults.animals),
            monsters: range("monsters", defaults.monsters),
            raiders: range("raiders", defaults.raiders),
            misc: range("misc", defaults.misc),
            ignore_spectators: self.spigot.string(
                "world-settings.default.entity-activation-range.ignore-spectators",
                &defaults.ignore_spectators.to_string(),
            ) == "true",
        }
    }

    pub fn network_config(&self, favicon: Option<String>) -> anyhow::Result<NetworkConfig> {
        let p = &self.properties;
        let ip = p.string("server-ip");
        let port = p.int("server-port", 25565).clamp(1, 65535) as u16;
        let host = if ip.is_empty() { "0.0.0.0" } else { ip };
        let bind = format!("{host}:{port}")
            .parse()
            .or_else(|_| format!("[{host}]:{port}").parse())?;
        let threshold = p.int("network-compression-threshold", 256);
        Ok(NetworkConfig {
            bind,
            online_mode: p.bool("online-mode"),
            prevent_proxy_connections: p.bool("prevent-proxy-connections"),
            compression_threshold: usize::try_from(threshold).ok(),
            motd: p.string("motd").to_owned(),
            favicon,
            hide_online_players: p.bool("hide-online-players"),
            enable_status: p.bool("enable-status"),
            enforce_secure_profile: false,
            accepts_transfers: p.bool("accepts-transfers"),
            keep_alive_timeout: Duration::from_secs(30),
            connection_throttle: Duration::from_millis(
                self.bukkit.int("settings.connection-throttle", 4000).max(0) as u64,
            ),
            outdated_client_message: self.spigot.string(
                "messages.outdated-client",
                "Outdated client! Please use {0}",
            ),
            outdated_server_message: self.spigot.string(
                "messages.outdated-server",
                "Outdated server! I'm still on {0}",
            ),
            ..NetworkConfig::default()
        })
    }
}
