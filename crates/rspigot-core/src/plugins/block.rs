use rspigot_domain::{BlockPos, BlockStateId};

use crate::plugins::resolve::{block_kind, within};
use crate::plugins::{BlockOrder, BlockState};
use crate::ports::ClientLink;
use crate::server::Server;

impl<L: ClientLink> Server<L> {
    pub(super) fn apply_block_order(&mut self, connection: u64, pos: BlockPos, order: BlockOrder) {
        let name = order.name();
        if let Err(reason) = self.block_order(pos, order) {
            self.warn_plugin(connection, name, reason);
        }
    }

    fn block_order(&mut self, pos: BlockPos, order: BlockOrder) -> Result<(), String> {
        let height = self.world.dimension.height;
        within("y", pos.y, height.min_y..=height.max_y())?;
        if self.world.chunks.block(pos).is_none() {
            return Err("the chunk is not loaded".to_owned());
        }
        match order {
            BlockOrder::Set(key) => {
                let state = block_kind(&key)?.default_state();
                self.set_block(pos, state);
            }
            BlockOrder::SetWith { kind, properties } => {
                let state = state_with(&kind, &properties)?;
                self.set_block(pos, state);
            }
            BlockOrder::BreakNaturally => {
                self.break_block_naturally(pos);
            }
        }
        Ok(())
    }

    pub(super) fn block_state(&self, pos: BlockPos) -> Option<BlockState> {
        let state = self.world.chunks.block(pos)?;
        Some(BlockState {
            kind: state.kind()?.name().to_owned(),
            properties: state
                .properties()
                .map(|(name, value)| (name.to_owned(), value.to_owned()))
                .collect(),
        })
    }
}

fn state_with(key: &str, properties: &[(String, String)]) -> Result<BlockStateId, String> {
    let kind = block_kind(key)?;
    let pairs: Vec<(&str, &str)> = properties
        .iter()
        .map(|(name, value)| (name.as_str(), value.as_str()))
        .collect();
    kind.state_with(&pairs).ok_or_else(|| {
        let asked: Vec<String> = properties
            .iter()
            .map(|(name, value)| format!("{name}={value}"))
            .collect();
        format!("{} does not take {}", kind.name(), asked.join(","))
    })
}
