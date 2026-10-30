use std::marker::PhantomData;

use crate::codec::{
    Decode, DecodeError, Encode, MAX_LIST_LEN, VarInt, VarLong, decode_len, decode_string,
    encode_len,
};

pub const UNBOUNDED: usize = MAX_LIST_LEN;

pub trait FieldCodec<T> {
    fn write(value: &T, out: &mut Vec<u8>);
    fn read(input: &mut &[u8]) -> Result<T, DecodeError>;
}

pub struct Plain;

impl<T: Encode + Decode> FieldCodec<T> for Plain {
    fn write(value: &T, out: &mut Vec<u8>) {
        value.encode(out);
    }

    fn read(input: &mut &[u8]) -> Result<T, DecodeError> {
        T::decode(input)
    }
}

pub struct Var;

impl FieldCodec<i32> for Var {
    fn write(value: &i32, out: &mut Vec<u8>) {
        VarInt(*value).encode(out);
    }

    fn read(input: &mut &[u8]) -> Result<i32, DecodeError> {
        VarInt::decode(input).map(|v| v.0)
    }
}

pub struct VarL;

impl FieldCodec<i64> for VarL {
    fn write(value: &i64, out: &mut Vec<u8>) {
        VarLong(*value).encode(out);
    }

    fn read(input: &mut &[u8]) -> Result<i64, DecodeError> {
        VarLong::decode(input).map(|v| v.0)
    }
}

pub struct Utf<const MAX_CHARS: usize>;

impl<const MAX_CHARS: usize> FieldCodec<String> for Utf<MAX_CHARS> {
    fn write(value: &String, out: &mut Vec<u8>) {
        value.encode(out);
    }

    fn read(input: &mut &[u8]) -> Result<String, DecodeError> {
        decode_string(input, MAX_CHARS)
    }
}

pub struct Opt<C>(PhantomData<C>);

impl<T, C: FieldCodec<T>> FieldCodec<Option<T>> for Opt<C> {
    fn write(value: &Option<T>, out: &mut Vec<u8>) {
        match value {
            Some(inner) => {
                true.encode(out);
                C::write(inner, out);
            }
            None => false.encode(out),
        }
    }

    fn read(input: &mut &[u8]) -> Result<Option<T>, DecodeError> {
        if bool::decode(input)? {
            C::read(input).map(Some)
        } else {
            Ok(None)
        }
    }
}

pub struct List<C, const MAX: usize>(PhantomData<C>);

impl<T, C: FieldCodec<T>, const MAX: usize> FieldCodec<Vec<T>> for List<C, MAX> {
    fn write(value: &Vec<T>, out: &mut Vec<u8>) {
        encode_len(value.len(), out);
        for item in value {
            C::write(item, out);
        }
    }

    fn read(input: &mut &[u8]) -> Result<Vec<T>, DecodeError> {
        let len = decode_len(input, MAX)?;
        let mut items = Vec::with_capacity(len.min(input.len()));
        for _ in 0..len {
            items.push(C::read(input)?);
        }
        Ok(items)
    }
}

pub struct Bytes<const MAX: usize>;

impl<const MAX: usize> FieldCodec<Vec<u8>> for Bytes<MAX> {
    fn write(value: &Vec<u8>, out: &mut Vec<u8>) {
        encode_len(value.len(), out);
        out.extend_from_slice(value);
    }

    fn read(input: &mut &[u8]) -> Result<Vec<u8>, DecodeError> {
        let len = decode_len(input, MAX)?;
        Ok(crate::codec::take(input, len)?.to_vec())
    }
}

pub fn write_var(value: i32, out: &mut Vec<u8>) {
    VarInt(value).encode(out);
}

pub fn read_var(input: &mut &[u8]) -> Result<i32, DecodeError> {
    VarInt::decode(input).map(|v| v.0)
}

pub fn read_count(input: &mut &[u8], max: usize) -> Result<usize, DecodeError> {
    decode_len(input, max)
}

pub fn read_array<const N: usize>(input: &mut &[u8]) -> Result<[u8; N], DecodeError> {
    let bytes = crate::codec::take(input, N)?;
    let mut out = [0u8; N];
    out.copy_from_slice(bytes);
    Ok(out)
}

macro_rules! field_codec {
    () => {
        $crate::play::wire::Plain
    };
    ($codec:ty) => {
        $codec
    };
}

pub(crate) use field_codec;

macro_rules! wire_struct {
    (
        $(#[$meta:meta])*
        pub struct $name:ident $(= $id:path)? {
            $( $(#[$fmeta:meta])* pub $field:ident : $ty:ty $(as $codec:ty)? ),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        pub struct $name {
            $( $(#[$fmeta])* pub $field: $ty, )+
        }

        impl $crate::codec::Encode for $name {
            fn encode(&self, out: &mut Vec<u8>) {
                $(
                    <$crate::play::wire::field_codec!($($codec)?) as $crate::play::wire::FieldCodec<$ty>>::write(&self.$field, out);
                )+
            }
        }

        impl $crate::codec::Decode for $name {
            fn decode(input: &mut &[u8]) -> Result<Self, $crate::codec::DecodeError> {
                Ok(Self {
                    $(
                        $field: <$crate::play::wire::field_codec!($($codec)?) as $crate::play::wire::FieldCodec<$ty>>::read(input)?,
                    )+
                })
            }
        }

        $(
            impl $crate::Packet for $name {
                const ID: i32 = $id;
            }
        )?
    };
}

pub(crate) use wire_struct;

macro_rules! unit_packet {
    ($(#[$meta:meta])* pub struct $name:ident = $id:path;) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
        pub struct $name;

        impl $crate::codec::Encode for $name {
            fn encode(&self, _out: &mut Vec<u8>) {}
        }

        impl $crate::codec::Decode for $name {
            fn decode(_input: &mut &[u8]) -> Result<Self, $crate::codec::DecodeError> {
                Ok(Self)
            }
        }

        impl $crate::Packet for $name {
            const ID: i32 = $id;
        }
    };
}

pub(crate) use unit_packet;

macro_rules! wire_enum {
    (@fallback Strict, $name:ident, $id:ident) => {
        Self::from_id($id).ok_or($crate::codec::DecodeError::InvalidValue {
            field: stringify!($name),
            value: i64::from($id),
        })
    };
    (@fallback Zero, $name:ident, $id:ident) => {
        Ok(Self::from_id($id).unwrap_or(Self::ALL[0]))
    };
    (@fallback Wrap, $name:ident, $id:ident) => {
        Ok(Self::from_id($id.rem_euclid(Self::ALL.len() as i32)).unwrap_or(Self::ALL[0]))
    };
    (
        $(#[$meta:meta])*
        pub enum $name:ident : $fallback:ident {
            $( $(#[$vmeta:meta])* $variant:ident ),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $name {
            $( $(#[$vmeta])* $variant, )+
        }

        impl $name {
            pub const ALL: &'static [Self] = &[$( Self::$variant, )+];

            pub const fn id(self) -> i32 {
                self as i32
            }

            pub fn from_id(id: i32) -> Option<Self> {
                usize::try_from(id).ok().and_then(|index| Self::ALL.get(index).copied())
            }
        }

        impl $crate::codec::Encode for $name {
            fn encode(&self, out: &mut Vec<u8>) {
                $crate::codec::VarInt(self.id()).encode(out);
            }
        }

        impl $crate::codec::Decode for $name {
            fn decode(input: &mut &[u8]) -> Result<Self, $crate::codec::DecodeError> {
                let id = $crate::codec::VarInt::decode(input)?.0;
                $crate::play::wire::wire_enum!(@fallback $fallback, $name, id)
            }
        }
    };
}

pub(crate) use wire_enum;

#[cfg(test)]
pub(crate) mod testing {
    use crate::codec::{Decode, Encode, decode_exact};

    pub fn roundtrip<T: Encode + Decode + PartialEq + std::fmt::Debug>(value: &T) -> Vec<u8> {
        let mut out = Vec::new();
        value.encode(&mut out);
        assert_eq!(&decode_exact::<T>(&out).unwrap(), value);
        out
    }

    pub fn encoded<T: Encode>(value: &T) -> Vec<u8> {
        let mut out = Vec::new();
        value.encode(&mut out);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::testing::roundtrip;
    use super::*;
    use crate::codec::decode_exact;

    wire_struct! {
        #[derive(Debug, Clone, PartialEq)]
        pub struct Sample {
            pub id: i32 as Var,
            pub name: String as Utf<4>,
            pub flags: u8,
            pub tags: Vec<i32> as List<Var, 3>,
            pub extra: Option<i64> as Opt<VarL>,
        }
    }

    wire_enum! {
        pub enum Mode: Zero { First, Second }
    }

    wire_enum! {
        pub enum Turn: Wrap { North, East, South, West }
    }

    wire_enum! {
        pub enum Strict: Strict { Only }
    }

    #[test]
    fn struct_fields_encode_in_declaration_order() {
        let sample = Sample {
            id: 300,
            name: "ab".into(),
            flags: 7,
            tags: vec![1, 2],
            extra: Some(-1),
        };
        let bytes = roundtrip(&sample);
        assert_eq!(&bytes[..6], &[0xAC, 0x02, 0x02, b'a', b'b', 0x07]);
        assert_eq!(&bytes[6..9], &[0x02, 0x01, 0x02]);
        assert_eq!(bytes[9], 0x01);
        assert_eq!(bytes.len(), 20);
    }

    #[test]
    fn bounded_fields_reject_oversized_input() {
        let mut long_name = Vec::new();
        Sample {
            id: 0,
            name: "abcde".into(),
            flags: 0,
            tags: vec![],
            extra: None,
        }
        .encode(&mut long_name);
        assert!(decode_exact::<Sample>(&long_name).is_err());
        let mut many_tags = Vec::new();
        Sample {
            id: 0,
            name: String::new(),
            flags: 0,
            tags: vec![0; 4],
            extra: None,
        }
        .encode(&mut many_tags);
        assert!(matches!(
            decode_exact::<Sample>(&many_tags),
            Err(DecodeError::TooLong { len: 4, max: 3 })
        ));
    }

    #[test]
    fn enum_fallback_strategies_match_vanilla_by_id_map() {
        assert_eq!(decode_exact::<Mode>(&[1]).unwrap(), Mode::Second);
        assert_eq!(decode_exact::<Mode>(&[9]).unwrap(), Mode::First);
        assert_eq!(decode_exact::<Turn>(&[6]).unwrap(), Turn::South);
        assert_eq!(
            decode_exact::<Turn>(&[0xFF, 0xFF, 0xFF, 0xFF, 0x0F]).unwrap(),
            Turn::West
        );
        assert!(decode_exact::<Strict>(&[1]).is_err());
        assert_eq!(roundtrip(&Turn::West), vec![3]);
    }
}
