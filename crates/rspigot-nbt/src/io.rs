use crate::compound::Compound;
use crate::mutf8;
use crate::tag::{Tag, TagType};

pub const MAX_DEPTH: usize = 512;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum NbtError {
    #[error("unexpected end of input")]
    UnexpectedEof,
    #[error("unknown tag type {0}")]
    UnknownTag(u8),
    #[error("root tag must be a compound, got {0:?}")]
    RootNotCompound(TagType),
    #[error("nesting deeper than {MAX_DEPTH}")]
    TooDeep,
    #[error("negative length {0}")]
    NegativeLength(i32),
    #[error("invalid modified utf-8 string")]
    InvalidString,
    #[error("string longer than 65535 bytes")]
    StringTooLong,
}

struct Reader<'a, 'b> {
    input: &'b mut &'a [u8],
}

impl<'a> Reader<'a, '_> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], NbtError> {
        if self.input.len() < n {
            return Err(NbtError::UnexpectedEof);
        }
        let (head, tail) = self.input.split_at(n);
        *self.input = tail;
        Ok(head)
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], NbtError> {
        let bytes = self.take(N)?;
        let mut out = [0u8; N];
        out.copy_from_slice(bytes);
        Ok(out)
    }

    fn u8(&mut self) -> Result<u8, NbtError> {
        Ok(self.array::<1>()?[0])
    }

    fn i16(&mut self) -> Result<i16, NbtError> {
        Ok(i16::from_be_bytes(self.array()?))
    }

    fn i32(&mut self) -> Result<i32, NbtError> {
        Ok(i32::from_be_bytes(self.array()?))
    }

    fn i64(&mut self) -> Result<i64, NbtError> {
        Ok(i64::from_be_bytes(self.array()?))
    }

    fn len(&mut self) -> Result<usize, NbtError> {
        let n = self.i32()?;
        usize::try_from(n).map_err(|_| NbtError::NegativeLength(n))
    }

    fn string(&mut self) -> Result<String, NbtError> {
        let len = u16::from_be_bytes(self.array()?) as usize;
        let bytes = self.take(len)?;
        mutf8::decode(bytes)
            .map(|s| s.into_owned())
            .ok_or(NbtError::InvalidString)
    }

    fn tag_type(&mut self) -> Result<TagType, NbtError> {
        let id = self.u8()?;
        TagType::from_id(id).ok_or(NbtError::UnknownTag(id))
    }

    fn payload(&mut self, ty: TagType, depth: usize) -> Result<Tag, NbtError> {
        if depth > MAX_DEPTH {
            return Err(NbtError::TooDeep);
        }
        Ok(match ty {
            TagType::End => return Err(NbtError::UnknownTag(0)),
            TagType::Byte => Tag::Byte(self.u8()? as i8),
            TagType::Short => Tag::Short(self.i16()?),
            TagType::Int => Tag::Int(self.i32()?),
            TagType::Long => Tag::Long(self.i64()?),
            TagType::Float => Tag::Float(f32::from_bits(self.i32()? as u32)),
            TagType::Double => Tag::Double(f64::from_bits(self.i64()? as u64)),
            TagType::ByteArray => {
                let len = self.len()?;
                Tag::ByteArray(self.take(len)?.to_vec())
            }
            TagType::String => Tag::String(self.string()?),
            TagType::List => {
                let element = self.tag_type()?;
                let len = self.i32()?;
                let len = usize::try_from(len).unwrap_or(0);
                if element == TagType::End {
                    Tag::List(Vec::new())
                } else {
                    let mut items = Vec::with_capacity(len.min(self.input.len()));
                    for _ in 0..len {
                        items.push(self.payload(element, depth + 1)?);
                    }
                    Tag::List(items)
                }
            }
            TagType::Compound => Tag::Compound(self.compound(depth + 1)?),
            TagType::IntArray => {
                let len = self.len()?;
                let bytes = self.take(len.checked_mul(4).ok_or(NbtError::UnexpectedEof)?)?;
                Tag::IntArray(
                    bytes
                        .as_chunks::<4>()
                        .0
                        .iter()
                        .map(|c| i32::from_be_bytes(*c))
                        .collect(),
                )
            }
            TagType::LongArray => {
                let len = self.len()?;
                let bytes = self.take(len.checked_mul(8).ok_or(NbtError::UnexpectedEof)?)?;
                Tag::LongArray(
                    bytes
                        .as_chunks::<8>()
                        .0
                        .iter()
                        .map(|c| i64::from_be_bytes(*c))
                        .collect(),
                )
            }
        })
    }

    fn compound(&mut self, depth: usize) -> Result<Compound, NbtError> {
        let mut compound = Compound::new();
        loop {
            let ty = self.tag_type()?;
            if ty == TagType::End {
                return Ok(compound);
            }
            let name = self.string()?;
            let value = self.payload(ty, depth)?;
            compound.insert(name, value);
        }
    }
}

pub fn read_named(input: &mut &[u8]) -> Result<(String, Compound), NbtError> {
    let mut reader = Reader { input };
    let ty = reader.tag_type()?;
    if ty != TagType::Compound {
        return Err(NbtError::RootNotCompound(ty));
    }
    let name = reader.string()?;
    let compound = reader.compound(1)?;
    Ok((name, compound))
}

pub fn read_network(input: &mut &[u8]) -> Result<Option<Tag>, NbtError> {
    let mut reader = Reader { input };
    let ty = reader.tag_type()?;
    if ty == TagType::End {
        return Ok(None);
    }
    reader.payload(ty, 0).map(Some)
}

pub fn read_network_compound(input: &mut &[u8]) -> Result<Option<Compound>, NbtError> {
    match read_network(input)? {
        None => Ok(None),
        Some(Tag::Compound(c)) => Ok(Some(c)),
        Some(other) => Err(NbtError::RootNotCompound(other.tag_type())),
    }
}

fn write_string(out: &mut Vec<u8>, s: &str) {
    let bytes = mutf8::encode(s);
    let len = bytes.len().min(u16::MAX as usize);
    out.extend_from_slice(&(len as u16).to_be_bytes());
    out.extend_from_slice(&bytes[..len]);
}

fn list_element_type(items: &[Tag]) -> TagType {
    items.first().map_or(TagType::End, Tag::tag_type)
}

fn is_homogeneous(items: &[Tag]) -> bool {
    let ty = list_element_type(items);
    items.iter().all(|t| t.tag_type() == ty)
}

fn write_payload(out: &mut Vec<u8>, tag: &Tag) {
    match tag {
        Tag::Byte(v) => out.push(*v as u8),
        Tag::Short(v) => out.extend_from_slice(&v.to_be_bytes()),
        Tag::Int(v) => out.extend_from_slice(&v.to_be_bytes()),
        Tag::Long(v) => out.extend_from_slice(&v.to_be_bytes()),
        Tag::Float(v) => out.extend_from_slice(&v.to_bits().to_be_bytes()),
        Tag::Double(v) => out.extend_from_slice(&v.to_bits().to_be_bytes()),
        Tag::ByteArray(v) => {
            out.extend_from_slice(&(v.len() as i32).to_be_bytes());
            out.extend_from_slice(v);
        }
        Tag::String(s) => write_string(out, s),
        Tag::List(items) => {
            if is_homogeneous(items) {
                out.push(list_element_type(items) as u8);
                out.extend_from_slice(&(items.len() as i32).to_be_bytes());
                for item in items {
                    write_payload(out, item);
                }
            } else {
                out.push(TagType::Compound as u8);
                out.extend_from_slice(&(items.len() as i32).to_be_bytes());
                for item in items {
                    match item {
                        Tag::Compound(c) => write_compound(out, c),
                        other => {
                            out.push(other.tag_type() as u8);
                            write_string(out, "");
                            write_payload(out, other);
                            out.push(TagType::End as u8);
                        }
                    }
                }
            }
        }
        Tag::Compound(c) => write_compound(out, c),
        Tag::IntArray(v) => {
            out.extend_from_slice(&(v.len() as i32).to_be_bytes());
            for x in v {
                out.extend_from_slice(&x.to_be_bytes());
            }
        }
        Tag::LongArray(v) => {
            out.extend_from_slice(&(v.len() as i32).to_be_bytes());
            for x in v {
                out.extend_from_slice(&x.to_be_bytes());
            }
        }
    }
}

fn write_compound(out: &mut Vec<u8>, compound: &Compound) {
    for (name, value) in compound.iter() {
        out.push(value.tag_type() as u8);
        write_string(out, name);
        write_payload(out, value);
    }
    out.push(TagType::End as u8);
}

pub fn write_named(out: &mut Vec<u8>, name: &str, compound: &Compound) {
    out.push(TagType::Compound as u8);
    write_string(out, name);
    write_compound(out, compound);
}

pub fn write_network(out: &mut Vec<u8>, tag: &Tag) {
    out.push(tag.tag_type() as u8);
    write_payload(out, tag);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Compound {
        let inner = Compound::new()
            .with("name", "Bananrama")
            .with("value", 0.5f32);
        Compound::new()
            .with("byte", 1i8)
            .with("long", i64::MIN)
            .with("double", 0.25f64)
            .with("ints", vec![1, 2, 3])
            .with("longs", vec![1i64 << 40])
            .with("nested", inner.clone())
            .with(
                "list",
                Tag::List(vec![Tag::Compound(inner.clone()), Tag::Compound(inner)]),
            )
            .with("bytes", Tag::ByteArray(vec![0, 255]))
            .with("empty", Tag::List(Vec::new()))
    }

    #[test]
    fn named_roundtrip() {
        let mut out = Vec::new();
        write_named(&mut out, "hello world", &sample());
        let mut input = out.as_slice();
        let (name, compound) = read_named(&mut input).unwrap();
        assert_eq!(name, "hello world");
        assert_eq!(compound, sample());
        assert!(input.is_empty());
    }

    #[test]
    fn network_roundtrip_with_string_root() {
        let mut out = Vec::new();
        write_network(&mut out, &Tag::String("hi".into()));
        assert_eq!(out, vec![8, 0, 2, b'h', b'i']);
        let mut input = out.as_slice();
        assert_eq!(
            read_network(&mut input).unwrap(),
            Some(Tag::String("hi".into()))
        );
    }

    #[test]
    fn heterogeneous_list_is_wrapped_in_compounds() {
        let mut out = Vec::new();
        let compound = Compound::new().with(
            "mixed",
            Tag::List(vec![Tag::Int(1), Tag::String("a".into())]),
        );
        write_named(&mut out, "", &compound);
        let mut input = out.as_slice();
        let (_, read) = read_named(&mut input).unwrap();
        let list = read.get_list("mixed").unwrap();
        assert_eq!(list[0].as_compound().unwrap().get_i32(""), Some(1));
        assert_eq!(list[1].as_compound().unwrap().get_str(""), Some("a"));
    }

    #[test]
    fn truncated_input_fails() {
        let mut out = Vec::new();
        write_named(&mut out, "x", &sample());
        out.truncate(out.len() - 3);
        let mut input = out.as_slice();
        assert_eq!(read_named(&mut input).unwrap_err(), NbtError::UnexpectedEof);
    }

    #[test]
    fn depth_limit() {
        let mut tag = Tag::Compound(Compound::new());
        for _ in 0..600 {
            tag = Tag::Compound(Compound::new().with("a", tag));
        }
        let mut out = Vec::new();
        write_network(&mut out, &tag);
        let mut input = out.as_slice();
        assert_eq!(read_network(&mut input).unwrap_err(), NbtError::TooDeep);
    }
}
