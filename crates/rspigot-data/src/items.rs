use crate::generated::items::MAX_STACK;
use crate::generated::static_registries::{ITEM, STATIC};

const DEFAULT_STACK_SIZE: u8 = 64;

pub fn id(name: &str) -> Option<u16> {
    STATIC[ITEM].index_of(name)
}

pub fn name(id: u16) -> Option<&'static str> {
    STATIC[ITEM].name_of(id)
}

pub fn count() -> usize {
    STATIC[ITEM].entries.len()
}

pub fn max_stack_size(id: u16) -> u8 {
    MAX_STACK
        .get(usize::from(id))
        .copied()
        .unwrap_or(DEFAULT_STACK_SIZE)
}
