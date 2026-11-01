use rspigot_domain::damage::DamageType;

use crate::commands::{self, CommandContext, CommandSender};
use crate::config::GameRules;
use crate::events::{ClientEvent, GameEvent, SoundSource};
use crate::explosion::BlockInteraction;
use crate::health::DamageSource;
use crate::plugins::resolve::{self, finite_not_negative, not_negative};
use crate::plugins::{GameRule, ServerOrder, ServerState, WorldOrder, WorldState};
use crate::ports::ClientLink;
use crate::server::{Audience, Server};

const PARTICLE_RANGE: f64 = 32.0;

impl<L: ClientLink> Server<L> {
    pub(super) fn apply_server_order(&mut self, order: ServerOrder) {
        match order {
            ServerOrder::Broadcast(message) => {
                CommandContext::broadcast(self, resolve::text(&message));
            }
            ServerOrder::DispatchCommand(line) => {
                commands::execute(self, CommandSender::Console, &line);
            }
            ServerOrder::Shutdown => CommandContext::stop(self),
            ServerOrder::Save => {
                CommandContext::save_all(self);
            }
            ServerOrder::SetWhitelistEnabled(enabled) => {
                CommandContext::set_whitelist_enabled(self, enabled);
            }
            ServerOrder::WhitelistAdd(name) => {
                CommandContext::whitelist_add(self, &name);
            }
            ServerOrder::WhitelistRemove(name) => {
                CommandContext::whitelist_remove(self, &name);
            }
            ServerOrder::OperatorAdd(name) => {
                CommandContext::set_operator(self, &name, true);
            }
            ServerOrder::OperatorRemove(name) => {
                CommandContext::set_operator(self, &name, false);
            }
        }
    }

    pub(super) fn apply_world_order(&mut self, connection: u64, order: WorldOrder) {
        let name = order.name();
        if let Err(reason) = self.world_order(order) {
            self.warn_plugin(connection, name, reason);
        }
    }

    fn world_order(&mut self, order: WorldOrder) -> Result<(), String> {
        match order {
            WorldOrder::SetTime(time) => {
                not_negative("day time", time)?;
                CommandContext::set_day_time(self, time);
            }
            WorldOrder::SetWeather(weather) => CommandContext::set_weather(self, weather),
            WorldOrder::SetDifficulty(difficulty) => {
                CommandContext::set_difficulty(self, difficulty);
            }
            WorldOrder::SetSpawn(pos) => CommandContext::set_world_spawn(self, pos),
            WorldOrder::SetPvp(pvp) => self.config.pvp = pvp,
            WorldOrder::SetGameRule(rule, value) => self.set_game_rule(rule, value),
            WorldOrder::Save => {
                CommandContext::save_all(self);
            }
            WorldOrder::PlaySound { sound, position } => {
                let key = resolve::sound_of(&sound)?;
                self.play_sound(
                    None,
                    position,
                    key,
                    SoundSource::Master,
                    sound.volume,
                    sound.pitch,
                );
            }
            WorldOrder::SpawnParticle(spec) => {
                let particle = resolve::particle_of(&spec)?;
                self.push(
                    Audience::Nearby {
                        position: spec.position,
                        range: PARTICLE_RANGE,
                        except: None,
                    },
                    ClientEvent::Particle(particle),
                );
            }
            WorldOrder::Explode {
                position,
                power,
                breaks_blocks,
            } => {
                let position = resolve::position_of(position)?;
                finite_not_negative("power", f64::from(power))?;
                let interaction = if breaks_blocks {
                    BlockInteraction::Destroy
                } else {
                    BlockInteraction::Keep
                };
                let damage = DamageSource::new(DamageType::Explosion);
                self.explode(None, &damage, position, power, interaction);
            }
        }
        Ok(())
    }

    fn set_game_rule(&mut self, rule: GameRule, value: bool) {
        *rule_flag(
            &mut self.config.rules,
            &mut self.world.level.daylight_cycle,
            rule,
        ) = value;
        match rule {
            GameRule::DaylightCycle => {
                let event = self.time_event();
                self.push(Audience::All, event);
            }
            GameRule::ImmediateRespawn => self.push(
                Audience::All,
                ClientEvent::GameEvent(GameEvent::ImmediateRespawn(value)),
            ),
            _ => {}
        }
    }

    pub(super) fn server_state(&self) -> ServerState {
        ServerState {
            max_players: self.config.max_players,
            online: u32::try_from(self.players.len()).unwrap_or(u32::MAX),
            tps: self.tps(),
            whitelist_enabled: self.admin.whitelist_enabled,
        }
    }

    pub(super) fn world_state(&self) -> WorldState {
        let level = &self.world.level;
        let (mut rules, mut daylight) = (self.config.rules, level.daylight_cycle);
        WorldState {
            name: level.name.clone(),
            uuid: level.uuid,
            seed: level.seed,
            day_time: level.day_time,
            game_time: level.game_time,
            raining: level.raining,
            thundering: level.thundering,
            difficulty: self.world.difficulty,
            spawn: self.world.spawn.block(),
            pvp: self.config.pvp,
            rules: GameRule::ALL
                .iter()
                .map(|&rule| (rule, *rule_flag(&mut rules, &mut daylight, rule)))
                .collect(),
        }
    }
}

const fn rule_flag<'a>(
    rules: &'a mut GameRules,
    daylight: &'a mut bool,
    rule: GameRule,
) -> &'a mut bool {
    match rule {
        GameRule::DaylightCycle => daylight,
        GameRule::NaturalRegeneration => &mut rules.natural_regeneration,
        GameRule::KeepInventory => &mut rules.keep_inventory,
        GameRule::ShowDeathMessages => &mut rules.show_death_messages,
        GameRule::ImmediateRespawn => &mut rules.immediate_respawn,
        GameRule::FallDamage => &mut rules.fall_damage,
        GameRule::MobDrops => &mut rules.mob_drops,
        GameRule::MobGriefing => &mut rules.mob_griefing,
        GameRule::MobExplosionDropDecay => &mut rules.mob_explosion_drop_decay,
        GameRule::BlockDrops => &mut rules.block_drops,
        GameRule::UniversalAnger => &mut rules.universal_anger,
    }
}
