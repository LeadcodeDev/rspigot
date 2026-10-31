use rspigot_plugin_protocol::v1::{self, answer, from_plugin};
use uuid::Uuid;

use crate::Error;
use crate::link::Link;
use crate::server::Server;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Position {
    pub(crate) fn from_wire(vec: Option<v1::Vec3>, field: &str) -> Result<Self, Error> {
        let vec = vec.ok_or_else(|| Error::Malformed(format!("a message without `{field}`")))?;
        Ok(Self {
            x: vec.x,
            y: vec.y,
            z: vec.z,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameMode {
    Survival,
    Creative,
    Adventure,
    Spectator,
}

impl GameMode {
    fn from_wire(mode: i32) -> Result<Self, Error> {
        match v1::GameMode::try_from(mode) {
            Ok(v1::GameMode::Survival) => Ok(Self::Survival),
            Ok(v1::GameMode::Creative) => Ok(Self::Creative),
            Ok(v1::GameMode::Adventure) => Ok(Self::Adventure),
            Ok(v1::GameMode::Spectator) => Ok(Self::Spectator),
            Ok(v1::GameMode::Unspecified) | Err(_) => {
                Err(Error::Malformed(format!("an unknown game mode {mode}")))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Player {
    pub uuid: Uuid,
    pub name: String,
    pub entity_id: i32,
    pub world: String,
    pub position: Position,
    pub game_mode: GameMode,
    link: Link,
}

impl Player {
    pub(crate) fn from_wire(
        snapshot: Option<v1::PlayerSnapshot>,
        link: &Link,
    ) -> Result<Self, Error> {
        let snapshot =
            snapshot.ok_or_else(|| Error::Malformed("a message without its player".to_owned()))?;
        let uuid = Uuid::parse_str(&snapshot.uuid).map_err(|_| {
            Error::Malformed(format!("`{}`, which is not a player UUID", snapshot.uuid))
        })?;
        Ok(Self {
            uuid,
            game_mode: GameMode::from_wire(snapshot.game_mode)?,
            name: snapshot.name,
            entity_id: snapshot.entity_id,
            world: snapshot.world,
            position: Position::from_wire(snapshot.position, "position")?,
            link: link.clone(),
        })
    }

    pub fn send_message(&self, text: impl Into<String>) {
        self.link
            .order(from_plugin::Body::SendMessage(v1::SendMessage {
                player: self.uuid.to_string(),
                text: text.into(),
            }));
    }

    pub async fn resolve(&self) -> Result<PlayerState, Error> {
        let player = self.uuid.to_string();
        let answer = self
            .link
            .ask(|request| from_plugin::Body::ResolvePlayer(v1::ResolvePlayer { request, player }))
            .await?;
        match answer {
            answer::Body::Player(state) => PlayerState::from_wire(state, &self.link),
            answer::Body::NotFound(_) => Err(Error::NotFound),
            answer::Body::Players(_) => Err(Error::Malformed(
                "a player list where one player was expected".to_owned(),
            )),
        }
    }

    pub fn server(&self) -> Server {
        Server::new(self.link.clone())
    }
}

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub player: Player,
    pub health: f32,
    pub food: i32,
    pub level: i32,
}

impl PlayerState {
    fn from_wire(state: v1::PlayerState, link: &Link) -> Result<Self, Error> {
        Ok(Self {
            player: Player::from_wire(state.player, link)?,
            health: state.health,
            food: state.food,
            level: state.level,
        })
    }
}
