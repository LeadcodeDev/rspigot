use rspigot_blocks::BlockKind;

use crate::{Item, ItemBehavior};

pub struct BlockItem;

impl ItemBehavior for BlockItem {
    fn placed_block<I: Item>() -> Option<BlockKind> {
        I::BLOCK.and_then(BlockKind::from_id)
    }
}
