use std::future::Future;

use rspigot_plugin_protocol::v1;

use crate::event::{InventoryClick, InventoryClose, PlayerJoin, PlayerLeave, PlayerMove};
use crate::server::Server;

pub trait Plugin: Listeners + Send + Sync + 'static {
    const NAME: &'static str;
    const VERSION: &'static str;
}

pub trait Listeners {
    fn handlers() -> Vec<HandlerSpec>;

    fn dispatch(&self, call: Call) -> impl Future<Output = Outcome> + Send;

    fn enable(&self, server: &Server) -> impl Future<Output = ()> + Send;

    fn disable(&self) -> impl Future<Output = ()> + Send;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventKind {
    PlayerJoin,
    PlayerLeave,
    PlayerMove,
    InventoryClick,
    InventoryClose,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Access {
    Observe,
    Decide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum Priority {
    Lowest,
    Low,
    #[default]
    Normal,
    High,
    Highest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HandlerSpec {
    pub id: u32,
    pub event: EventKind,
    pub access: Access,
    pub priority: Priority,
    pub ignore_cancelled: bool,
}

impl HandlerSpec {
    pub(crate) fn to_wire(self) -> v1::Handler {
        let event = match self.event {
            EventKind::PlayerJoin => v1::EventKind::PlayerJoin,
            EventKind::PlayerLeave => v1::EventKind::PlayerLeave,
            EventKind::PlayerMove => v1::EventKind::PlayerMove,
            EventKind::InventoryClick => v1::EventKind::InventoryClick,
            EventKind::InventoryClose => v1::EventKind::InventoryClose,
        };
        let access = match self.access {
            Access::Observe => v1::Access::Observe,
            Access::Decide => v1::Access::Decide,
        };
        let priority = match self.priority {
            Priority::Lowest => v1::Priority::Lowest,
            Priority::Low => v1::Priority::Low,
            Priority::Normal => v1::Priority::Normal,
            Priority::High => v1::Priority::High,
            Priority::Highest => v1::Priority::Highest,
        };
        v1::Handler {
            id: self.id,
            event: event as i32,
            access: access as i32,
            priority: priority as i32,
            ignore_cancelled: self.ignore_cancelled,
        }
    }
}

#[derive(Debug)]
pub enum Call {
    PlayerJoin { handler: u32, event: PlayerJoin },
    PlayerLeave { handler: u32, event: PlayerLeave },
    PlayerMove { handler: u32, event: PlayerMove },
    InventoryClick { handler: u32, event: InventoryClick },
    InventoryClose { handler: u32, event: InventoryClose },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Observed,
    Decided { cancel: bool },
}
