use rspigot_nbt::{NbtError, Tag};
use uuid::Uuid;

pub const MAX_STRING_CHARS: usize = 32767;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DecodeError {
    #[error("unexpected end of packet")]
    UnexpectedEof,
    #[error("varint too long")]
    VarIntTooLong,
    #[error("string of {0} bytes exceeds the limit")]
    StringTooLong(usize),
    #[error("invalid utf-8 string")]
    InvalidUtf8,
    #[error("negative length {0}")]
    NegativeLength(i32),
    #[error("length {len} exceeds the limit {max}")]
    TooLong { len: usize, max: usize },
    #[error("invalid value {value} for {field}")]
    InvalidValue { field: &'static str, value: i64 },
    #[error("unknown packet id {0:#x}")]
    UnknownPacket(i32),
    #[error("{0} trailing bytes after packet")]
    TrailingBytes(usize),
    #[error("nbt: {0}")]
    Nbt(#[from] NbtError),
}

pub trait Encode {
    fn encode(&self, out: &mut Vec<u8>);
}

pub trait Decode: Sized {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError>;
}

pub fn take<'a>(input: &mut &'a [u8], n: usize) -> Result<&'a [u8], DecodeError> {
    if input.len() < n {
        return Err(DecodeError::UnexpectedEof);
    }
    let (head, tail) = input.split_at(n);
    *input = tail;
    Ok(head)
}

fn array<const N: usize>(input: &mut &[u8]) -> Result<[u8; N], DecodeError> {
    let bytes = take(input, N)?;
    let mut out = [0u8; N];
    out.copy_from_slice(bytes);
    Ok(out)
}

pub fn decode_exact<T: Decode>(mut body: &[u8]) -> Result<T, DecodeError> {
    let value = T::decode(&mut body)?;
    if body.is_empty() {
        Ok(value)
    } else {
        Err(DecodeError::TrailingBytes(body.len()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct VarInt(pub i32);

impl VarInt {
    pub const MAX_LEN: usize = 5;

    pub const fn written_len(self) -> usize {
        let v = self.0 as u32;
        match v {
            0..=0x7F => 1,
            0x80..=0x3FFF => 2,
            0x4000..=0x1F_FFFF => 3,
            0x20_0000..=0x0FFF_FFFF => 4,
            _ => 5,
        }
    }

    pub fn decode_partial(input: &[u8]) -> Result<Option<(i32, usize)>, DecodeError> {
        let mut value: u32 = 0;
        for (i, byte) in input.iter().take(Self::MAX_LEN).enumerate() {
            value |= u32::from(byte & 0x7F) << (7 * i);
            if byte & 0x80 == 0 {
                return Ok(Some((value as i32, i + 1)));
            }
        }
        if input.len() >= Self::MAX_LEN {
            Err(DecodeError::VarIntTooLong)
        } else {
            Ok(None)
        }
    }
}

impl Encode for VarInt {
    fn encode(&self, out: &mut Vec<u8>) {
        let mut v = self.0 as u32;
        loop {
            if v & !0x7F == 0 {
                out.push(v as u8);
                return;
            }
            out.push((v as u8 & 0x7F) | 0x80);
            v >>= 7;
        }
    }
}

impl Decode for VarInt {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        match Self::decode_partial(input)? {
            Some((value, len)) => {
                *input = &input[len..];
                Ok(Self(value))
            }
            None => Err(DecodeError::UnexpectedEof),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct VarLong(pub i64);

impl Encode for VarLong {
    fn encode(&self, out: &mut Vec<u8>) {
        let mut v = self.0 as u64;
        loop {
            if v & !0x7F == 0 {
                out.push(v as u8);
                return;
            }
            out.push((v as u8 & 0x7F) | 0x80);
            v >>= 7;
        }
    }
}

impl Decode for VarLong {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        let mut value: u64 = 0;
        for i in 0..10 {
            let byte = array::<1>(input)?[0];
            value |= u64::from(byte & 0x7F) << (7 * i);
            if byte & 0x80 == 0 {
                return Ok(Self(value as i64));
            }
        }
        Err(DecodeError::VarIntTooLong)
    }
}

pub fn decode_len(input: &mut &[u8], max: usize) -> Result<usize, DecodeError> {
    let len = VarInt::decode(input)?.0;
    let len = usize::try_from(len).map_err(|_| DecodeError::NegativeLength(len))?;
    if len > max {
        return Err(DecodeError::TooLong { len, max });
    }
    Ok(len)
}

pub fn encode_len(len: usize, out: &mut Vec<u8>) {
    VarInt(len as i32).encode(out);
}

macro_rules! be_number {
    ($($t:ty),*) => {$(
        impl Encode for $t {
            fn encode(&self, out: &mut Vec<u8>) {
                out.extend_from_slice(&self.to_be_bytes());
            }
        }

        impl Decode for $t {
            fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
                Ok(<$t>::from_be_bytes(array(input)?))
            }
        }
    )*};
}

be_number!(u8, i8, u16, i16, i32, u32, i64, u64, i128, u128, f32, f64);

impl Encode for bool {
    fn encode(&self, out: &mut Vec<u8>) {
        out.push(u8::from(*self));
    }
}

impl Decode for bool {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        match array::<1>(input)?[0] {
            0 => Ok(false),
            1 => Ok(true),
            other => Err(DecodeError::InvalidValue {
                field: "bool",
                value: i64::from(other),
            }),
        }
    }
}

impl Encode for str {
    fn encode(&self, out: &mut Vec<u8>) {
        encode_len(self.len(), out);
        out.extend_from_slice(self.as_bytes());
    }
}

impl Encode for String {
    fn encode(&self, out: &mut Vec<u8>) {
        self.as_str().encode(out);
    }
}

pub fn decode_string(input: &mut &[u8], max_chars: usize) -> Result<String, DecodeError> {
    let len = VarInt::decode(input)?.0;
    let len = usize::try_from(len).map_err(|_| DecodeError::NegativeLength(len))?;
    if len > max_chars * 3 {
        return Err(DecodeError::StringTooLong(len));
    }
    let bytes = take(input, len)?;
    let s = std::str::from_utf8(bytes).map_err(|_| DecodeError::InvalidUtf8)?;
    if s.encode_utf16().count() > max_chars {
        return Err(DecodeError::StringTooLong(len));
    }
    Ok(s.to_owned())
}

impl Decode for String {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        decode_string(input, MAX_STRING_CHARS)
    }
}

impl Encode for Uuid {
    fn encode(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(self.as_bytes());
    }
}

impl Decode for Uuid {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Uuid::from_bytes(array(input)?))
    }
}

impl<T: Encode> Encode for Option<T> {
    fn encode(&self, out: &mut Vec<u8>) {
        match self {
            Some(v) => {
                true.encode(out);
                v.encode(out);
            }
            None => false.encode(out),
        }
    }
}

impl<T: Decode> Decode for Option<T> {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        if bool::decode(input)? {
            T::decode(input).map(Some)
        } else {
            Ok(None)
        }
    }
}

pub const MAX_LIST_LEN: usize = 1 << 20;

impl<T: Encode> Encode for [T] {
    fn encode(&self, out: &mut Vec<u8>) {
        encode_len(self.len(), out);
        for item in self {
            item.encode(out);
        }
    }
}

impl<T: Encode> Encode for Vec<T> {
    fn encode(&self, out: &mut Vec<u8>) {
        self.as_slice().encode(out);
    }
}

impl<T: Decode> Decode for Vec<T> {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        let len = decode_len(input, MAX_LIST_LEN)?;
        let mut items = Vec::with_capacity(len.min(input.len()));
        for _ in 0..len {
            items.push(T::decode(input)?);
        }
        Ok(items)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ByteArray(pub Vec<u8>);

impl Encode for ByteArray {
    fn encode(&self, out: &mut Vec<u8>) {
        encode_len(self.0.len(), out);
        out.extend_from_slice(&self.0);
    }
}

impl Decode for ByteArray {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        let len = decode_len(input, 1 << 21)?;
        Ok(Self(take(input, len)?.to_vec()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RemainingBytes(pub Vec<u8>);

impl Encode for RemainingBytes {
    fn encode(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.0);
    }
}

impl Decode for RemainingBytes {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        let rest = std::mem::take(input);
        Ok(Self(rest.to_vec()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Encode for Position {
    fn encode(&self, out: &mut Vec<u8>) {
        let packed = ((i64::from(self.x) & 0x3FF_FFFF) << 38)
            | ((i64::from(self.z) & 0x3FF_FFFF) << 12)
            | (i64::from(self.y) & 0xFFF);
        packed.encode(out);
    }
}

impl Decode for Position {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        let packed = i64::decode(input)?;
        Ok(Self {
            x: (packed >> 38) as i32,
            y: ((packed << 52) >> 52) as i32,
            z: ((packed << 26) >> 38) as i32,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Angle(pub u8);

impl Angle {
    pub fn from_degrees(degrees: f32) -> Self {
        Self((degrees * 256.0 / 360.0).floor() as i32 as u8)
    }

    pub fn degrees(self) -> f32 {
        f32::from(self.0) * 360.0 / 256.0
    }
}

impl Encode for Angle {
    fn encode(&self, out: &mut Vec<u8>) {
        out.push(self.0);
    }
}

impl Decode for Angle {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self(array::<1>(input)?[0]))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NetworkNbt(pub Option<Tag>);

impl Encode for NetworkNbt {
    fn encode(&self, out: &mut Vec<u8>) {
        match &self.0 {
            Some(tag) => rspigot_nbt::write_network(out, tag),
            None => out.push(0),
        }
    }
}

impl Decode for NetworkNbt {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self(rspigot_nbt::read_network(input)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip<T: Encode + Decode + PartialEq + std::fmt::Debug>(value: T) -> Vec<u8> {
        let mut out = Vec::new();
        value.encode(&mut out);
        assert_eq!(decode_exact::<T>(&out).unwrap(), value);
        out
    }

    #[test]
    fn varint_known_vectors() {
        assert_eq!(roundtrip(VarInt(0)), vec![0x00]);
        assert_eq!(roundtrip(VarInt(127)), vec![0x7F]);
        assert_eq!(roundtrip(VarInt(128)), vec![0x80, 0x01]);
        assert_eq!(roundtrip(VarInt(25565)), vec![0xDD, 0xC7, 0x01]);
        assert_eq!(roundtrip(VarInt(-1)), vec![0xFF, 0xFF, 0xFF, 0xFF, 0x0F]);
        assert_eq!(
            roundtrip(VarInt(i32::MIN)),
            vec![0x80, 0x80, 0x80, 0x80, 0x08]
        );
        assert_eq!(VarInt(-1).written_len(), 5);
        assert_eq!(VarInt(25565).written_len(), 3);
    }

    #[test]
    fn varlong_known_vectors() {
        assert_eq!(
            roundtrip(VarLong(-1)),
            vec![0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01]
        );
        assert_eq!(
            roundtrip(VarLong(2147483648)),
            vec![0x80, 0x80, 0x80, 0x80, 0x08]
        );
    }

    #[test]
    fn partial_varint_needs_more_bytes() {
        assert_eq!(VarInt::decode_partial(&[0x80]).unwrap(), None);
        assert_eq!(
            VarInt::decode_partial(&[0x80, 0x80, 0x80, 0x80, 0x80]),
            Err(DecodeError::VarIntTooLong)
        );
    }

    #[test]
    fn position_packing_matches_vanilla() {
        let mut out = Vec::new();
        Position {
            x: 18357644,
            y: 831,
            z: -20882616,
        }
        .encode(&mut out);
        assert_eq!(out, vec![0x46, 0x07, 0x63, 0x2C, 0x15, 0xB4, 0x83, 0x3F]);
        roundtrip(Position {
            x: -1,
            y: -64,
            z: -33_554_432,
        });
    }

    #[test]
    fn string_and_option_roundtrip() {
        roundtrip(String::from("héllo"));
        roundtrip(Some(VarInt(3)));
        roundtrip(Option::<VarInt>::None);
        roundtrip(vec![VarInt(1), VarInt(300)]);
    }

    #[test]
    fn invalid_bool_rejected() {
        assert!(matches!(
            decode_exact::<bool>(&[2]),
            Err(DecodeError::InvalidValue { .. })
        ));
    }
}
