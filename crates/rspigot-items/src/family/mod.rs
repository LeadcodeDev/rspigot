mod block_item;

pub use block_item::BlockItem;

use rspigot_blocks::BlockKind;

use crate::Item;

pub trait ItemBehavior {
    fn placed_block<I: Item>() -> Option<BlockKind> {
        None
    }
}

pub struct Plain;

impl ItemBehavior for Plain {}
