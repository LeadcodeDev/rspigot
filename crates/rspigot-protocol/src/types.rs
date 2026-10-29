use rspigot_nbt::Tag;
use uuid::Uuid;

use crate::codec::{Decode, DecodeError, Encode, NetworkNbt, decode_len, decode_string};

#[derive(Debug, Clone, PartialEq)]
pub struct Text(pub Tag);

impl Encode for Text {
    fn encode(&self, out: &mut Vec<u8>) {
        rspigot_nbt::write_network(out, &self.0);
    }
}

impl Decode for Text {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        match NetworkNbt::decode(input)?.0 {
            Some(tag) => Ok(Self(tag)),
            None => Err(DecodeError::InvalidValue {
                field: "text",
                value: 0,
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonText(pub String);

impl Encode for JsonText {
    fn encode(&self, out: &mut Vec<u8>) {
        self.0.encode(out);
    }
}

impl Decode for JsonText {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        decode_string(input, 262_144).map(Self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileProperty {
    pub name: String,
    pub value: String,
    pub signature: Option<String>,
}

impl Encode for ProfileProperty {
    fn encode(&self, out: &mut Vec<u8>) {
        self.name.encode(out);
        self.value.encode(out);
        self.signature.encode(out);
    }
}

impl Decode for ProfileProperty {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            name: decode_string(input, 64)?,
            value: decode_string(input, 32767)?,
            signature: if bool::decode(input)? {
                Some(decode_string(input, 1024)?)
            } else {
                None
            },
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameProfile {
    pub id: Uuid,
    pub name: String,
    pub properties: Vec<ProfileProperty>,
}

impl Encode for GameProfile {
    fn encode(&self, out: &mut Vec<u8>) {
        self.id.encode(out);
        self.name.encode(out);
        self.properties.encode(out);
    }
}

impl Decode for GameProfile {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        let id = Uuid::decode(input)?;
        let name = decode_string(input, 16)?;
        let count = decode_len(input, 16)?;
        let mut properties = Vec::with_capacity(count);
        for _ in 0..count {
            properties.push(ProfileProperty::decode(input)?);
        }
        Ok(Self {
            id,
            name,
            properties,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnownPack {
    pub namespace: String,
    pub id: String,
    pub version: String,
}

impl Encode for KnownPack {
    fn encode(&self, out: &mut Vec<u8>) {
        self.namespace.encode(out);
        self.id.encode(out);
        self.version.encode(out);
    }
}

impl Decode for KnownPack {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            namespace: String::decode(input)?,
            id: String::decode(input)?,
            version: String::decode(input)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BitSet(pub Vec<i64>);

impl BitSet {
    pub fn set(&mut self, bit: usize) {
        let word = bit / 64;
        if self.0.len() <= word {
            self.0.resize(word + 1, 0);
        }
        self.0[word] |= 1 << (bit % 64);
    }

    pub fn get(&self, bit: usize) -> bool {
        self.0
            .get(bit / 64)
            .is_some_and(|w| w & (1 << (bit % 64)) != 0)
    }
}

impl Encode for BitSet {
    fn encode(&self, out: &mut Vec<u8>) {
        self.0.encode(out);
    }
}

impl Decode for BitSet {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Vec::<i64>::decode(input).map(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitset_sets_bits_across_words() {
        let mut bits = BitSet::default();
        bits.set(0);
        bits.set(65);
        assert_eq!(bits.0, vec![1, 2]);
        assert!(bits.get(65));
        assert!(!bits.get(64));
    }
}
