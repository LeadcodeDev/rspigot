use crate::codec::{Decode, DecodeError, Encode, MAX_LIST_LEN, decode_len, encode_len, take};
use crate::play::wire::{FieldCodec, read_var, write_var};

pub const AIR_ITEM_ID: i32 = 0;
pub const MAX_HASHED_COMPONENTS: usize = 256;
pub const MAX_COMPONENT_VALUE_BYTES: usize = 1 << 21;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentValue {
    pub type_id: i32,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DataComponentPatch {
    pub added: Vec<ComponentValue>,
    pub removed: Vec<i32>,
}

impl DataComponentPatch {
    pub const EMPTY: Self = Self {
        added: Vec::new(),
        removed: Vec::new(),
    };

    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty()
    }

    fn write_header(&self, out: &mut Vec<u8>) {
        encode_len(self.added.len(), out);
        encode_len(self.removed.len(), out);
    }

    fn write_removed(&self, out: &mut Vec<u8>) {
        for type_id in &self.removed {
            write_var(*type_id, out);
        }
    }

    pub fn encode_trusted(&self, out: &mut Vec<u8>) {
        self.write_header(out);
        for value in &self.added {
            write_var(value.type_id, out);
            out.extend_from_slice(&value.data);
        }
        self.write_removed(out);
    }

    pub fn encode_untrusted(&self, out: &mut Vec<u8>) {
        self.write_header(out);
        for value in &self.added {
            write_var(value.type_id, out);
            encode_len(value.data.len(), out);
            out.extend_from_slice(&value.data);
        }
        self.write_removed(out);
    }

    fn read_counts(input: &mut &[u8]) -> Result<(usize, usize), DecodeError> {
        let added = decode_len(input, MAX_LIST_LEN)?;
        let removed = decode_len(input, MAX_LIST_LEN)?;
        Ok((added, removed))
    }

    fn read_removed(input: &mut &[u8], count: usize) -> Result<Vec<i32>, DecodeError> {
        let mut removed = Vec::with_capacity(count.min(input.len()));
        for _ in 0..count {
            removed.push(read_var(input)?);
        }
        Ok(removed)
    }

    pub fn decode_untrusted(input: &mut &[u8]) -> Result<Self, DecodeError> {
        let (added_count, removed_count) = Self::read_counts(input)?;
        let mut added = Vec::with_capacity(added_count.min(input.len()));
        for _ in 0..added_count {
            let type_id = read_var(input)?;
            let len = decode_len(input, MAX_COMPONENT_VALUE_BYTES)?;
            added.push(ComponentValue {
                type_id,
                data: take(input, len)?.to_vec(),
            });
        }
        let removed = Self::read_removed(input, removed_count)?;
        Ok(Self { added, removed })
    }

    pub fn decode_trusted(input: &mut &[u8]) -> Result<Self, DecodeError> {
        let (added_count, removed_count) = Self::read_counts(input)?;
        if added_count > 0 {
            let type_id = read_var(input)?;
            return Err(DecodeError::InvalidValue {
                field: "trusted data component without length prefix",
                value: i64::from(type_id),
            });
        }
        let removed = Self::read_removed(input, removed_count)?;
        Ok(Self {
            added: Vec::new(),
            removed,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemStack {
    pub item: i32,
    pub count: i32,
    pub components: DataComponentPatch,
}

impl ItemStack {
    pub const EMPTY: Self = Self {
        item: AIR_ITEM_ID,
        count: 0,
        components: DataComponentPatch::EMPTY,
    };

    pub fn new(item: i32, count: i32) -> Self {
        Self {
            item,
            count,
            components: DataComponentPatch::EMPTY,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.count <= 0 || self.item == AIR_ITEM_ID
    }

    fn write_with(&self, out: &mut Vec<u8>, patch: impl FnOnce(&DataComponentPatch, &mut Vec<u8>)) {
        if self.is_empty() {
            write_var(0, out);
            return;
        }
        write_var(self.count, out);
        write_var(self.item, out);
        patch(&self.components, out);
    }

    fn read_with(
        input: &mut &[u8],
        patch: impl FnOnce(&mut &[u8]) -> Result<DataComponentPatch, DecodeError>,
    ) -> Result<Self, DecodeError> {
        let count = read_var(input)?;
        if count <= 0 {
            return Ok(Self::EMPTY);
        }
        let item = read_var(input)?;
        let components = patch(input)?;
        Ok(Self {
            item,
            count,
            components,
        })
    }

    pub fn encode_untrusted(&self, out: &mut Vec<u8>) {
        self.write_with(out, DataComponentPatch::encode_untrusted);
    }

    pub fn decode_untrusted(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Self::read_with(input, DataComponentPatch::decode_untrusted)
    }
}

impl Encode for ItemStack {
    fn encode(&self, out: &mut Vec<u8>) {
        self.write_with(out, DataComponentPatch::encode_trusted);
    }
}

impl Decode for ItemStack {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Self::read_with(input, DataComponentPatch::decode_trusted)
    }
}

pub struct Untrusted;

impl FieldCodec<ItemStack> for Untrusted {
    fn write(value: &ItemStack, out: &mut Vec<u8>) {
        value.encode_untrusted(out);
    }

    fn read(input: &mut &[u8]) -> Result<ItemStack, DecodeError> {
        ItemStack::decode_untrusted(input)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStackTemplate {
    pub item: i32,
    pub count: i32,
    pub components: DataComponentPatch,
}

impl Encode for ItemStackTemplate {
    fn encode(&self, out: &mut Vec<u8>) {
        write_var(self.item, out);
        write_var(self.count, out);
        self.components.encode_trusted(out);
    }
}

impl Decode for ItemStackTemplate {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            item: read_var(input)?,
            count: read_var(input)?,
            components: DataComponentPatch::decode_trusted(input)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashedComponent {
    pub type_id: i32,
    pub hash: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashedItem {
    pub item: i32,
    pub count: i32,
    pub added: Vec<HashedComponent>,
    pub removed: Vec<i32>,
}

impl Encode for HashedItem {
    fn encode(&self, out: &mut Vec<u8>) {
        write_var(self.item, out);
        write_var(self.count, out);
        encode_len(self.added.len(), out);
        for component in &self.added {
            write_var(component.type_id, out);
            component.hash.encode(out);
        }
        encode_len(self.removed.len(), out);
        for type_id in &self.removed {
            write_var(*type_id, out);
        }
    }
}

impl Decode for HashedItem {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        let item = read_var(input)?;
        let count = read_var(input)?;
        let added_count = decode_len(input, MAX_HASHED_COMPONENTS)?;
        let mut added = Vec::with_capacity(added_count);
        for _ in 0..added_count {
            added.push(HashedComponent {
                type_id: read_var(input)?,
                hash: i32::decode(input)?,
            });
        }
        let removed_count = decode_len(input, MAX_HASHED_COMPONENTS)?;
        let mut removed = Vec::with_capacity(removed_count);
        for _ in 0..removed_count {
            removed.push(read_var(input)?);
        }
        Ok(Self {
            item,
            count,
            added,
            removed,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HashedStack(pub Option<HashedItem>);

impl HashedStack {
    pub const EMPTY: Self = Self(None);
}

impl Encode for HashedStack {
    fn encode(&self, out: &mut Vec<u8>) {
        self.0.encode(out);
    }
}

impl Decode for HashedStack {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Option::<HashedItem>::decode(input).map(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::decode_exact;
    use crate::play::wire::testing::{encoded, roundtrip};

    #[test]
    fn empty_stack_is_a_single_zero() {
        assert_eq!(roundtrip(&ItemStack::EMPTY), vec![0]);
        assert_eq!(encoded(&ItemStack::new(1, 0)), vec![0]);
        assert_eq!(encoded(&ItemStack::new(AIR_ITEM_ID, 5)), vec![0]);
        assert_eq!(
            decode_exact::<ItemStack>(&[0xFF, 0xFF, 0xFF, 0xFF, 0x0F]).unwrap(),
            ItemStack::EMPTY
        );
    }

    #[test]
    fn stack_with_empty_patch_is_count_item_zero_zero() {
        let sword = ItemStack::new(1050, 1);
        assert_eq!(roundtrip(&sword), vec![0x01, 0x9A, 0x08, 0x00, 0x00]);
        let mut untrusted = Vec::new();
        sword.encode_untrusted(&mut untrusted);
        assert_eq!(untrusted, vec![0x01, 0x9A, 0x08, 0x00, 0x00]);
    }

    #[test]
    fn trusted_patch_writes_values_without_length() {
        let stack = ItemStack {
            item: 1,
            count: 64,
            components: DataComponentPatch {
                added: vec![ComponentValue {
                    type_id: 1,
                    data: vec![0x10],
                }],
                removed: vec![3],
            },
        };
        assert_eq!(encoded(&stack), vec![64, 1, 1, 1, 1, 0x10, 3]);
        assert!(matches!(
            decode_exact::<ItemStack>(&encoded(&stack)),
            Err(DecodeError::InvalidValue { value: 1, .. })
        ));
        let mut untrusted = Vec::new();
        stack.encode_untrusted(&mut untrusted);
        assert_eq!(untrusted, vec![64, 1, 1, 1, 1, 1, 0x10, 3]);
        let mut input = untrusted.as_slice();
        assert_eq!(ItemStack::decode_untrusted(&mut input).unwrap(), stack);
        assert!(input.is_empty());
    }

    #[test]
    fn trusted_decode_accepts_removed_only_patch() {
        let stack = ItemStack {
            item: 7,
            count: 2,
            components: DataComponentPatch {
                added: Vec::new(),
                removed: vec![4, 9],
            },
        };
        assert_eq!(roundtrip(&stack), vec![2, 7, 0, 2, 4, 9]);
    }

    #[test]
    fn hashed_stack_layout() {
        assert_eq!(roundtrip(&HashedStack::EMPTY), vec![0]);
        let hashed = HashedStack(Some(HashedItem {
            item: 1,
            count: 3,
            added: vec![HashedComponent {
                type_id: 6,
                hash: -1,
            }],
            removed: vec![2],
        }));
        assert_eq!(
            roundtrip(&hashed),
            vec![1, 1, 3, 1, 6, 0xFF, 0xFF, 0xFF, 0xFF, 1, 2]
        );
        let mut too_many = vec![1, 1, 3];
        encode_len(MAX_HASHED_COMPONENTS + 1, &mut too_many);
        assert!(matches!(
            decode_exact::<HashedStack>(&too_many),
            Err(DecodeError::TooLong { .. })
        ));
    }

    #[test]
    fn template_writes_item_before_count() {
        let template = ItemStackTemplate {
            item: 1050,
            count: 1,
            components: DataComponentPatch::EMPTY,
        };
        assert_eq!(roundtrip(&template), vec![0x9A, 0x08, 0x01, 0x00, 0x00]);
    }
}
