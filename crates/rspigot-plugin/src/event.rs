use rspigot_plugin_protocol::v1::{self, event, patch};

use crate::custom_inventory::{ClickKind, ClickedSlot, CustomInventory};
use crate::link::Link;
use crate::listener::{Call, Verdict};
use crate::player::{Player, Position};
use crate::{Error, ItemStack};

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
    to: Position,
    id: u64,
    cancelled: bool,
    rolled_back: bool,
    moved_to: Option<Position>,
}

impl PlayerMove {
    pub const fn id(&self) -> u64 {
        self.id
    }

    pub const fn to(&self) -> Position {
        self.to
    }

    pub const fn set_to(&mut self, to: Position) {
        self.to = to;
        self.moved_to = Some(to);
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
            moved_to: self.moved_to,
        }
    }
}

impl From<PlayerMove> for Verdict {
    fn from(event: PlayerMove) -> Self {
        Self {
            cancel: event.cancelled,
            patch: event.moved_to.map(|to| v1::Patch {
                change: Some(patch::Change::PlayerMove(v1::PlayerMovePatch {
                    to: Some(to.to_wire()),
                })),
            }),
        }
    }
}

#[derive(Debug, Clone)]
pub struct InventoryClick {
    pub player: Player,
    pub inventory: CustomInventory,
    pub slot: ClickedSlot,
    pub kind: ClickKind,
    pub item: Option<ItemStack>,
    id: u64,
}

impl InventoryClick {
    pub const fn id(&self) -> u64 {
        self.id
    }
}

#[derive(Debug, Clone)]
pub struct InventoryClose {
    pub player: Player,
    pub inventory: CustomInventory,
    pub by_player: bool,
    id: u64,
}

impl InventoryClose {
    pub const fn id(&self) -> u64 {
        self.id
    }
}

pub(crate) enum Received {
    Join(PlayerJoin),
    Leave(PlayerLeave),
    Move(PlayerMove),
    Click(InventoryClick),
    Close(InventoryClose),
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
                moved_to: None,
            }),
            event::Payload::InventoryClick(click) => Self::Click(InventoryClick {
                player: Player::from_wire(click.player, link)?,
                inventory: CustomInventory::new(click.inventory, link),
                slot: ClickedSlot::from_wire(click.slot)?,
                kind: ClickKind::from_wire(click.kind, click.hotbar_key)?,
                item: click.item.map(ItemStack::from_wire).transpose()?,
                id,
            }),
            event::Payload::InventoryClose(close) => Self::Close(InventoryClose {
                player: Player::from_wire(close.player, link)?,
                inventory: CustomInventory::new(close.inventory, link),
                by_player: close.by_player,
                id,
            }),
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
            Self::Click(event) => Call::InventoryClick {
                handler,
                event: event.clone(),
            },
            Self::Close(event) => Call::InventoryClose {
                handler,
                event: event.clone(),
            },
        }
    }
}
