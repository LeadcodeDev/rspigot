use std::sync::Arc;

use parking_lot::RwLock;
use rspigot_domain::player::PlayerId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerSample {
    pub id: PlayerId,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StatusSnapshot {
    pub online: u32,
    pub max_players: u32,
    pub sample: Vec<PlayerSample>,
}

#[derive(Debug, Clone, Default)]
pub struct StatusBoard(Arc<RwLock<StatusSnapshot>>);

impl StatusBoard {
    pub fn new(max_players: u32) -> Self {
        Self(Arc::new(RwLock::new(StatusSnapshot {
            max_players,
            ..StatusSnapshot::default()
        })))
    }

    pub fn snapshot(&self) -> StatusSnapshot {
        self.0.read().clone()
    }

    pub fn update(&self, update: impl FnOnce(&mut StatusSnapshot)) {
        update(&mut self.0.write());
    }
}
