use crate::generated::blocks::{AIR, BLOCKS, BY_NAME, CAVE_AIR, PROP_REFS, PROPS, VOID_AIR};

pub const STATE_COUNT: usize = crate::generated::blocks::STATE_COUNT;

pub(crate) struct Prop {
    pub(crate) name: &'static str,
    pub(crate) values: &'static [&'static str],
}

pub(crate) struct Block {
    pub(crate) name: &'static str,
    pub(crate) first: u16,
    pub(crate) default: u16,
    pub(crate) ref_start: u16,
    pub(crate) ref_len: u8,
}

impl Block {
    fn prop_refs(&self) -> &'static [u8] {
        let start = usize::from(self.ref_start);
        &PROP_REFS[start..start + usize::from(self.ref_len)]
    }

    fn locate(&self, key: &str) -> Option<(&'static Prop, usize)> {
        let mut stride = 1;
        for &prop_ref in self.prop_refs().iter().rev() {
            let prop = &PROPS[usize::from(prop_ref)];
            if prop.name == key {
                return Some((prop, stride));
            }
            stride *= prop.values.len();
        }
        None
    }

    fn replaced(&self, state: u16, key: &str, value: &str) -> Option<u16> {
        let (prop, stride) = self.locate(key)?;
        let new = prop
            .values
            .iter()
            .position(|candidate| *candidate == value)?;
        let old = usize::from(state - self.first) / stride % prop.values.len();
        u16::try_from(usize::from(state) - old * stride + new * stride).ok()
    }
}

fn block_index(state: u16) -> Option<usize> {
    if usize::from(state) >= STATE_COUNT {
        return None;
    }
    BLOCKS
        .partition_point(|block| block.first <= state)
        .checked_sub(1)
}

fn block_by_name(name: &str) -> Option<&'static Block> {
    let position = BY_NAME
        .binary_search_by(|&id| BLOCKS[usize::from(id)].name.cmp(name))
        .ok()?;
    Some(&BLOCKS[usize::from(BY_NAME[position])])
}

pub fn is_air(state: u16) -> bool {
    matches!(state, AIR | VOID_AIR | CAVE_AIR)
}

pub fn default_state(block: &str) -> Option<u16> {
    block_by_name(block).map(|block| block.default)
}

pub fn state_with(block: &str, properties: &[(&str, &str)]) -> Option<u16> {
    let block = block_by_name(block)?;
    properties
        .iter()
        .try_fold(block.default, |state, &(key, value)| {
            block.replaced(state, key, value)
        })
}

pub fn with_property(state: u16, key: &str, value: &str) -> Option<u16> {
    BLOCKS[block_index(state)?].replaced(state, key, value)
}

pub fn property(state: u16, key: &str) -> Option<&'static str> {
    let block = &BLOCKS[block_index(state)?];
    let (prop, stride) = block.locate(key)?;
    let index = usize::from(state - block.first) / stride % prop.values.len();
    prop.values.get(index).copied()
}

pub fn properties(state: u16) -> Vec<(&'static str, &'static str)> {
    let Some(index) = block_index(state) else {
        return Vec::new();
    };
    let block = &BLOCKS[index];
    let refs = block.prop_refs();
    let offset = usize::from(state - block.first);
    let mut stride: usize = refs
        .iter()
        .map(|&prop_ref| PROPS[usize::from(prop_ref)].values.len())
        .product();
    let mut result = Vec::with_capacity(refs.len());
    for &prop_ref in refs {
        let prop = &PROPS[usize::from(prop_ref)];
        stride /= prop.values.len();
        result.push((prop.name, prop.values[offset / stride % prop.values.len()]));
    }
    result
}

pub fn block_name(state: u16) -> Option<&'static str> {
    block_index(state).map(|index| BLOCKS[index].name)
}

pub fn block_id(state: u16) -> Option<u16> {
    block_index(state).and_then(|index| u16::try_from(index).ok())
}

pub fn block_count() -> usize {
    BLOCKS.len()
}
