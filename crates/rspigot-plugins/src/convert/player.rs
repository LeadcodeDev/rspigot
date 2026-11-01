use rspigot_core::plugins::{Order, PlayerOrder, PlayerSnapshot, PlayerState};
use rspigot_domain::Rotation;
use rspigot_domain::player::{GameMode, PlayerId};
use rspigot_plugin_protocol::v1::{self, player_order};
use uuid::Uuid;

use super::{Malformed, wire_vec};

pub(super) fn order(order: v1::PlayerOrder) -> Result<Order, Malformed> {
    let player = id(order.player)?;
    let order = match order.order.ok_or(Malformed::Empty)? {
        player_order::Order::SendMessage(text) => PlayerOrder::SendMessage(text),
        _ => return Err(Malformed::Unsupported("player order")),
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

const fn wire_rotation(rotation: Rotation) -> v1::Rotation {
    v1::Rotation {
        yaw: rotation.yaw,
        pitch: rotation.pitch,
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
