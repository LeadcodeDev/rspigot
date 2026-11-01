use rspigot_core::plugins::{Order, PlayerOrder, PlayerSnapshot, PlayerState};
use rspigot_domain::player::{GameMode, PlayerId};
use rspigot_plugin_protocol::v1::{self, player_order::Order as Wire};
use uuid::Uuid;

use super::Malformed;
use super::shared::{self, required, wire_rotation, wire_vec};

pub(super) fn order(order: v1::PlayerOrder) -> Result<Order, Malformed> {
    let player = id(order.player)?;
    let order = match required(order.order)? {
        Wire::SendMessage(text) => PlayerOrder::SendMessage(text),
        Wire::SendActionBar(text) => PlayerOrder::SendActionBar(text),
        Wire::SendTitle(title) => PlayerOrder::SendTitle(shared::title(title)),
        Wire::ResetTitle(_) => PlayerOrder::ResetTitle,
        Wire::SetTabList(v1::TabList { header, footer }) => {
            PlayerOrder::SetTabList { header, footer }
        }
        Wire::Teleport(position) => PlayerOrder::Teleport(shared::vec3(position)),
        Wire::TeleportFacing(facing) => PlayerOrder::TeleportFacing(
            shared::vec3(required(facing.position)?),
            shared::rotation(required(facing.rotation)?),
        ),
        Wire::SetGameMode(mode) => PlayerOrder::SetGameMode(game_mode(mode)?),
        Wire::Kick(reason) => PlayerOrder::Kick(reason),
        Wire::SetHealth(health) => PlayerOrder::SetHealth(health),
        Wire::SetAbsorption(amount) => PlayerOrder::SetAbsorption(amount),
        Wire::Damage(amount) => PlayerOrder::Damage(amount),
        Wire::Kill(_) => PlayerOrder::Kill,
        Wire::SetFoodLevel(food) => PlayerOrder::SetFoodLevel(food),
        Wire::SetSaturation(saturation) => PlayerOrder::SetSaturation(saturation),
        Wire::SetExhaustion(exhaustion) => PlayerOrder::SetExhaustion(exhaustion),
        Wire::GiveExp(points) => PlayerOrder::GiveExp(points),
        Wire::GiveLevels(levels) => PlayerOrder::GiveLevels(levels),
        Wire::SetLevel(level) => PlayerOrder::SetLevel(level),
        Wire::SetExp(progress) => PlayerOrder::SetExp(progress),
        Wire::AddEffect(effect) => PlayerOrder::AddEffect(shared::effect(effect)),
        Wire::RemoveEffect(key) => PlayerOrder::RemoveEffect(key),
        Wire::ClearEffects(_) => PlayerOrder::ClearEffects,
        Wire::SetFireTicks(ticks) => PlayerOrder::SetFireTicks(ticks),
        Wire::SetVelocity(velocity) => PlayerOrder::SetVelocity(shared::vec3(velocity)),
        Wire::SetAllowFlight(allowed) => PlayerOrder::SetAllowFlight(allowed),
        Wire::SetFlying(flying) => PlayerOrder::SetFlying(flying),
        Wire::SetFlySpeed(speed) => PlayerOrder::SetFlySpeed(speed),
        Wire::SetWalkSpeed(speed) => PlayerOrder::SetWalkSpeed(speed),
        Wire::SetOp(operator) => PlayerOrder::SetOp(operator),
        Wire::Respawn(_) => PlayerOrder::Respawn,
        Wire::PerformCommand(line) => PlayerOrder::PerformCommand(line),
        Wire::Chat(text) => PlayerOrder::Chat(text),
        Wire::PlaySound(sound) => PlayerOrder::PlaySound(shared::sound(sound)),
        Wire::SpawnParticle(particle) => PlayerOrder::SpawnParticle(shared::particle(particle)?),
        Wire::CloseInventory(_) => PlayerOrder::CloseInventory,
        Wire::UpdateInventory(_) => PlayerOrder::UpdateInventory,
        Wire::Ban(terms) => PlayerOrder::Ban {
            reason: terms.reason,
            expires: shared::expiry(terms.expires_ms)?,
        },
        Wire::HidePlayer(_) | Wire::ShowPlayer(_) | Wire::OpenInventory(_) => {
            return Err(Malformed::Unsupported("player order"));
        }
    };
    Ok(Order::Player { player, order })
}

pub(super) fn id(raw: String) -> Result<PlayerId, Malformed> {
    Uuid::parse_str(&raw)
        .map(PlayerId)
        .map_err(|_| Malformed::Player(raw))
}

pub(super) fn wire_state(state: PlayerState) -> v1::PlayerState {
    v1::PlayerState {
        player: Some(wire_player(state.player)),
        health: state.health,
        food: state.food,
        level: state.level,
        max_health: state.max_health,
        absorption: state.absorption,
        saturation: state.saturation,
        exhaustion: state.exhaustion,
        exp: state.exp,
        total_exp: state.total_exp,
        op: state.operator,
        allow_flight: state.allow_flight,
        flying: state.flying,
        fly_speed: state.fly_speed,
        walk_speed: state.walk_speed,
        sneaking: state.sneaking,
        sprinting: state.sprinting,
        on_ground: state.on_ground,
        fire_ticks: state.fire_ticks,
        dead: state.dead,
        effects: state.effects.into_iter().map(shared::wire_effect).collect(),
        ping: state.ping,
        locale: state.locale,
        address: state.address.to_string(),
        ..v1::PlayerState::default()
    }
}

pub(super) fn wire_player(player: PlayerSnapshot) -> v1::PlayerSnapshot {
    v1::PlayerSnapshot {
        uuid: player.id.0.hyphenated().to_string(),
        name: player.name,
        entity_id: player.entity_id,
        world: player.world,
        position: Some(wire_vec(player.position)),
        game_mode: wire_game_mode(player.game_mode) as i32,
        rotation: Some(wire_rotation(player.rotation)),
    }
}

fn game_mode(value: i32) -> Result<GameMode, Malformed> {
    match v1::GameMode::try_from(value) {
        Ok(v1::GameMode::Survival) => Ok(GameMode::Survival),
        Ok(v1::GameMode::Creative) => Ok(GameMode::Creative),
        Ok(v1::GameMode::Adventure) => Ok(GameMode::Adventure),
        Ok(v1::GameMode::Spectator) => Ok(GameMode::Spectator),
        Ok(v1::GameMode::Unspecified) | Err(_) => Err(Malformed::Enum {
            field: "game_mode",
            value,
        }),
    }
}

const fn wire_game_mode(mode: GameMode) -> v1::GameMode {
    match mode {
        GameMode::Survival => v1::GameMode::Survival,
        GameMode::Creative => v1::GameMode::Creative,
        GameMode::Adventure => v1::GameMode::Adventure,
        GameMode::Spectator => v1::GameMode::Spectator,
    }
}

#[cfg(test)]
mod tests {
    use std::net::IpAddr;
    use std::time::{Duration, UNIX_EPOCH};

    use rspigot_core::plugins::{
        EffectSpec, ParticleDataSpec, ParticleSpec, PlayerOrder, PlayerState, TitleSpec,
    };
    use rspigot_domain::{Rotation, Vec3};
    use rspigot_plugin_protocol::v1::{particle, player_order};

    use super::*;

    const STEVE: &str = "00000000-0000-0000-0000-000000000001";

    fn convert(order: player_order::Order) -> Result<PlayerOrder, Malformed> {
        let converted = super::order(v1::PlayerOrder {
            player: STEVE.into(),
            order: Some(order),
        })?;
        let Order::Player { player, order } = converted else {
            panic!("expected a player order");
        };
        assert_eq!(player, PlayerId(Uuid::from_u128(1)));
        Ok(order)
    }

    fn wire_vec3(x: f64, y: f64, z: f64) -> v1::Vec3 {
        v1::Vec3 { x, y, z }
    }

    fn snapshot() -> PlayerSnapshot {
        PlayerSnapshot {
            id: PlayerId(Uuid::from_u128(1)),
            name: "Steve".to_owned(),
            entity_id: 12,
            world: "minecraft:overworld".to_owned(),
            position: Vec3::new(1.0, 64.0, -3.5),
            rotation: Rotation::new(90.0, -30.0),
            game_mode: GameMode::Survival,
        }
    }

    #[test]
    fn player_orders_convert_with_their_values() {
        assert_eq!(
            convert(player_order::Order::SetHealth(5.5)).unwrap(),
            PlayerOrder::SetHealth(5.5)
        );
        assert_eq!(
            convert(player_order::Order::SendTitle(v1::Title {
                title: "§6Hi".into(),
                subtitle: None,
                fade_in: 10,
                stay: 70,
                fade_out: 20,
            }))
            .unwrap(),
            PlayerOrder::SendTitle(TitleSpec {
                title: "§6Hi".into(),
                subtitle: None,
                fade_in: 10,
                stay: 70,
                fade_out: 20,
            })
        );
        assert_eq!(
            convert(player_order::Order::TeleportFacing(v1::Facing {
                position: Some(wire_vec3(1.5, 70.0, -2.5)),
                rotation: Some(v1::Rotation {
                    yaw: 90.0,
                    pitch: 10.0
                }),
            }))
            .unwrap(),
            PlayerOrder::TeleportFacing(Vec3::new(1.5, 70.0, -2.5), Rotation::new(90.0, 10.0))
        );
        assert_eq!(
            convert(player_order::Order::SpawnParticle(v1::Particle {
                key: "minecraft:dust".into(),
                data: Some(particle::Data::Dust(v1::Dust {
                    color: 0xFF0000,
                    scale: 1.0
                })),
                position: Some(wire_vec3(0.5, 65.0, 0.5)),
                count: 4,
                spread: Some(wire_vec3(0.25, 0.0, 0.25)),
                speed: 0.1,
            }))
            .unwrap(),
            PlayerOrder::SpawnParticle(ParticleSpec {
                key: "minecraft:dust".into(),
                data: ParticleDataSpec::Dust {
                    color: 0xFF0000,
                    scale: 1.0
                },
                position: Vec3::new(0.5, 65.0, 0.5),
                count: 4,
                spread: Vec3::new(0.25, 0.0, 0.25),
                speed: 0.1,
            })
        );
        assert_eq!(
            convert(player_order::Order::AddEffect(v1::Effect {
                key: "minecraft:speed".into(),
                duration: -1,
                amplifier: 2,
                ambient: true,
                particles: false,
                icon: true,
            }))
            .unwrap(),
            PlayerOrder::AddEffect(EffectSpec {
                key: "minecraft:speed".into(),
                duration: -1,
                amplifier: 2,
                ambient: true,
                particles: false,
                icon: true,
            })
        );
        assert_eq!(
            convert(player_order::Order::SetGameMode(
                v1::GameMode::Adventure as i32
            ))
            .unwrap(),
            PlayerOrder::SetGameMode(GameMode::Adventure)
        );
        assert!(matches!(
            convert(player_order::Order::SetGameMode(0)),
            Err(Malformed::Enum {
                field: "game_mode",
                value: 0
            })
        ));
        assert!(matches!(
            convert(player_order::Order::TeleportFacing(v1::Facing {
                position: None,
                rotation: None,
            })),
            Err(Malformed::Empty)
        ));
    }

    fn state() -> PlayerState {
        PlayerState {
            player: snapshot(),
            health: 10.0,
            max_health: 20.0,
            absorption: 4.0,
            food: 5,
            saturation: 2.5,
            exhaustion: 1.5,
            level: 3,
            exp: 0.5,
            total_exp: 27,
            operator: true,
            allow_flight: true,
            flying: true,
            fly_speed: 0.2,
            walk_speed: 0.4,
            sneaking: true,
            sprinting: false,
            on_ground: true,
            fire_ticks: 100,
            dead: false,
            effects: vec![EffectSpec {
                key: "minecraft:speed".into(),
                duration: 200,
                amplifier: 1,
                ambient: false,
                particles: true,
                icon: true,
            }],
            ping: 43,
            locale: "fr_fr".into(),
            address: IpAddr::from([203, 0, 113, 7]),
        }
    }

    #[test]
    fn a_full_player_state_converts_to_the_wire() {
        assert_eq!(
            wire_state(state()),
            v1::PlayerState {
                player: Some(wire_player(snapshot())),
                health: 10.0,
                food: 5,
                level: 3,
                max_health: 20.0,
                absorption: 4.0,
                saturation: 2.5,
                exhaustion: 1.5,
                exp: 0.5,
                total_exp: 27,
                op: true,
                allow_flight: true,
                flying: true,
                fly_speed: 0.2,
                walk_speed: 0.4,
                sneaking: true,
                sprinting: false,
                on_ground: true,
                fire_ticks: 100,
                dead: false,
                effects: vec![v1::Effect {
                    key: "minecraft:speed".into(),
                    duration: 200,
                    amplifier: 1,
                    ambient: false,
                    particles: true,
                    icon: true,
                }],
                ping: 43,
                locale: "fr_fr".into(),
                address: "203.0.113.7".into(),
                hidden: Vec::new(),
            }
        );
    }

    #[test]
    fn player_ban_and_address_convert() {
        assert_eq!(
            convert(player_order::Order::Ban(v1::BanTerms {
                reason: None,
                expires_ms: None,
            }))
            .unwrap(),
            PlayerOrder::Ban {
                reason: None,
                expires: None,
            }
        );
        assert_eq!(
            convert(player_order::Order::Ban(v1::BanTerms {
                reason: Some("cheating".into()),
                expires_ms: Some(-1_500),
            }))
            .unwrap(),
            PlayerOrder::Ban {
                reason: Some("cheating".into()),
                expires: Some(UNIX_EPOCH - Duration::from_millis(1_500)),
            }
        );
        let ipv6 = PlayerState {
            address: "2001:db8::7".parse().expect("an address"),
            ..state()
        };
        assert_eq!(wire_state(ipv6).address, "2001:db8::7");
    }
}
