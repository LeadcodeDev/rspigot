use rspigot_plugin_protocol::v1::{self, event};

use crate::Error;
use crate::link::Link;
use crate::listener::Call;
use crate::player::{Player, Position};

#[derive(Debug, Clone)]
pub struct PlayerJoin {
    pub player: Player,
    id: u64,
}

impl PlayerJoin {
    pub const fn id(&self) -> u64 {
        self.id
    }
}

#[derive(Debug, Clone)]
pub struct PlayerLeave {
    pub player: Player,
    id: u64,
}

impl PlayerLeave {
    pub const fn id(&self) -> u64 {
        self.id
    }
}

#[derive(Debug)]
pub struct PlayerMove {
    pub player: Player,
    pub from: Position,
    pub to: Position,
    id: u64,
    cancelled: bool,
    rolled_back: bool,
}

impl PlayerMove {
    pub const fn id(&self) -> u64 {
        self.id
    }

    pub const fn cancel(&mut self) {
        self.cancelled = true;
    }

    pub const fn is_cancelled(&self) -> bool {
        self.cancelled
    }

    pub const fn is_rolled_back(&self) -> bool {
        self.rolled_back
    }

    pub(crate) fn copy(&self) -> Self {
        Self {
            player: self.player.clone(),
            from: self.from,
            to: self.to,
            id: self.id,
            cancelled: self.cancelled,
            rolled_back: self.rolled_back,
        }
    }
}

pub(crate) enum Received {
    Join(PlayerJoin),
    Leave(PlayerLeave),
    Move(PlayerMove),
}

impl Received {
    pub(crate) fn from_wire(event: v1::Event, link: &Link) -> Result<Self, Error> {
        let id = event.id;
        let cancelled = event.cancelled;
        let rolled_back = event.phase() == v1::Phase::RolledBack;
        let payload = event
            .payload
            .ok_or_else(|| Error::Malformed(format!("event {id} without a payload")))?;
        Ok(match payload {
            event::Payload::PlayerJoin(join) => Self::Join(PlayerJoin {
                player: Player::from_wire(join.player, link)?,
                id,
            }),
            event::Payload::PlayerLeave(leave) => Self::Leave(PlayerLeave {
                player: Player::from_wire(leave.player, link)?,
                id,
            }),
            event::Payload::PlayerMove(moved) => Self::Move(PlayerMove {
                player: Player::from_wire(moved.player, link)?,
                from: Position::from_wire(moved.from, "from")?,
                to: Position::from_wire(moved.to, "to")?,
                id,
                cancelled,
                rolled_back,
            }),
            event::Payload::InventoryClick(_) => return Err(not_handled_yet("an inventory click")),
            event::Payload::InventoryClose(_) => return Err(not_handled_yet("an inventory close")),
        })
    }

    pub(crate) fn call(&self, handler: u32) -> Call {
        match self {
            Self::Join(event) => Call::PlayerJoin {
                handler,
                event: event.clone(),
            },
            Self::Leave(event) => Call::PlayerLeave {
                handler,
                event: event.clone(),
            },
            Self::Move(event) => Call::PlayerMove {
                handler,
                event: event.copy(),
            },
        }
    }
}

fn not_handled_yet(event: &str) -> Error {
    Error::Malformed(format!("{event}, which this plugin cannot handle yet"))
}
