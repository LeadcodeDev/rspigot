use std::io::{Read, Write};

use flate2::Compression as Level;
use flate2::read::{GzDecoder, ZlibDecoder};
use flate2::write::{GzEncoder, ZlibEncoder};

use crate::error::AnvilError;

const MAX_DECOMPRESSED: usize = 256 * 1024 * 1024;
const LZ4_MAGIC: &[u8; 8] = b"LZ4Block";
const LZ4_HEADER: usize = 21;
const LZ4_METHOD_RAW: u8 = 0x10;
const LZ4_METHOD_LZ4: u8 = 0x20;
const LZ4_LEVEL_BASE: u32 = 10;
const LZ4_BLOCK_SIZE: usize = 1 << 16;
const LZ4_LEVEL: u8 = 6;
const LZ4_SEED: u32 = 0x9747_B28C;
const LZ4_CHECKSUM_MASK: u32 = 0x0FFF_FFFF;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Compression {
    Gzip,
    #[default]
    Zlib,
    Uncompressed,
    Lz4,
}

impl Compression {
    pub const fn id(self) -> u8 {
        match self {
            Self::Gzip => 1,
            Self::Zlib => 2,
            Self::Uncompressed => 3,
            Self::Lz4 => 4,
        }
    }

    pub const fn from_id(id: u8) -> Option<Self> {
        Some(match id {
            1 => Self::Gzip,
            2 => Self::Zlib,
            3 => Self::Uncompressed,
            4 => Self::Lz4,
            _ => return None,
        })
    }

    pub fn from_property(name: &str) -> Option<Self> {
        Some(match name {
            "deflate" => Self::Zlib,
            "none" => Self::Uncompressed,
            "lz4" => Self::Lz4,
            _ => return None,
        })
    }

    pub fn compress(self, input: &[u8]) -> Result<Vec<u8>, AnvilError> {
        match self {
            Self::Gzip => {
                let mut encoder =
                    GzEncoder::new(Vec::with_capacity(input.len() / 2 + 32), Level::default());
                encoder.write_all(input)?;
                Ok(encoder.finish()?)
            }
            Self::Zlib => {
                let mut encoder =
                    ZlibEncoder::new(Vec::with_capacity(input.len() / 2 + 32), Level::default());
                encoder.write_all(input)?;
                Ok(encoder.finish()?)
            }
            Self::Uncompressed => Ok(input.to_vec()),
            Self::Lz4 => Ok(lz4_block_encode(input)),
        }
    }

    pub fn decompress(self, input: &[u8]) -> Result<Vec<u8>, AnvilError> {
        match self {
            Self::Gzip => read_capped(GzDecoder::new(input), input.len()),
            Self::Zlib => read_capped(ZlibDecoder::new(input), input.len()),
            Self::Uncompressed => Ok(input.to_vec()),
            Self::Lz4 => lz4_block_decode(input),
        }
    }
}

fn read_capped(reader: impl Read, hint: usize) -> Result<Vec<u8>, AnvilError> {
    let mut out = Vec::with_capacity(hint.saturating_mul(4).min(MAX_DECOMPRESSED));
    reader
        .take(MAX_DECOMPRESSED as u64 + 1)
        .read_to_end(&mut out)
        .map_err(|error| AnvilError::corrupt(format!("compressed stream: {error}")))?;
    if out.len() > MAX_DECOMPRESSED {
        return Err(AnvilError::corrupt(
            "decompressed chunk exceeds the size limit",
        ));
    }
    Ok(out)
}

fn lz4_block_encode(input: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len() / 2 + 2 * LZ4_HEADER);
    for block in input.chunks(LZ4_BLOCK_SIZE) {
        let compressed = lz4_flex::block::compress(block);
        let (method, body): (u8, &[u8]) = if compressed.len() >= block.len() {
            (LZ4_METHOD_RAW, block)
        } else {
            (LZ4_METHOD_LZ4, &compressed)
        };
        out.extend_from_slice(LZ4_MAGIC);
        out.push(method | LZ4_LEVEL);
        out.extend_from_slice(&(body.len() as u32).to_le_bytes());
        out.extend_from_slice(&(block.len() as u32).to_le_bytes());
        out.extend_from_slice(&(xxh32(block, LZ4_SEED) & LZ4_CHECKSUM_MASK).to_le_bytes());
        out.extend_from_slice(body);
    }
    out.extend_from_slice(LZ4_MAGIC);
    out.push(LZ4_METHOD_RAW | LZ4_LEVEL);
    out.extend_from_slice(&[0; 12]);
    out
}

fn lz4_block_decode(input: &[u8]) -> Result<Vec<u8>, AnvilError> {
    let mut out = Vec::with_capacity(input.len().saturating_mul(3).min(MAX_DECOMPRESSED));
    let mut rest = input;
    while !rest.is_empty() {
        if rest.len() < LZ4_HEADER {
            return Err(AnvilError::corrupt("truncated lz4 block header"));
        }
        if &rest[..8] != LZ4_MAGIC {
            return Err(AnvilError::corrupt("bad lz4 block magic"));
        }
        let token = rest[8];
        let method = token & 0xF0;
        let level = LZ4_LEVEL_BASE + u32::from(token & 0x0F);
        let compressed_len = i32::from_le_bytes([rest[9], rest[10], rest[11], rest[12]]);
        let original_len = i32::from_le_bytes([rest[13], rest[14], rest[15], rest[16]]);
        let checksum = u32::from_le_bytes([rest[17], rest[18], rest[19], rest[20]]);
        let (Ok(compressed_len), Ok(original_len)) = (
            usize::try_from(compressed_len),
            usize::try_from(original_len),
        ) else {
            return Err(AnvilError::corrupt("negative lz4 block length"));
        };
        if method != LZ4_METHOD_RAW && method != LZ4_METHOD_LZ4 {
            return Err(AnvilError::corrupt("unknown lz4 block method"));
        }
        if original_len > 1usize << level.min(30)
            || (method == LZ4_METHOD_RAW && compressed_len != original_len)
        {
            return Err(AnvilError::corrupt("inconsistent lz4 block lengths"));
        }
        if original_len == 0 {
            if compressed_len != 0 || checksum != 0 {
                return Err(AnvilError::corrupt("malformed lz4 end block"));
            }
            return Ok(out);
        }
        let body = rest
            .get(LZ4_HEADER..LZ4_HEADER + compressed_len)
            .ok_or_else(|| AnvilError::corrupt("truncated lz4 block body"))?;
        if out.len() + original_len > MAX_DECOMPRESSED {
            return Err(AnvilError::corrupt(
                "decompressed chunk exceeds the size limit",
            ));
        }
        let start = out.len();
        if method == LZ4_METHOD_RAW {
            out.extend_from_slice(body);
        } else {
            out.resize(start + original_len, 0);
            let written = lz4_flex::block::decompress_into(body, &mut out[start..])
                .map_err(|error| AnvilError::corrupt(format!("lz4 block: {error}")))?;
            if written != original_len {
                return Err(AnvilError::corrupt("lz4 block decoded to the wrong length"));
            }
        }
        if xxh32(&out[start..], LZ4_SEED) & LZ4_CHECKSUM_MASK != checksum {
            return Err(AnvilError::corrupt("lz4 block checksum mismatch"));
        }
        rest = &rest[LZ4_HEADER + compressed_len..];
    }
    Ok(out)
}

pub fn xxh32(data: &[u8], seed: u32) -> u32 {
    const P1: u32 = 0x9E37_79B1;
    const P2: u32 = 0x85EB_CA77;
    const P3: u32 = 0xC2B2_AE3D;
    const P4: u32 = 0x27D4_EB2F;
    const P5: u32 = 0x1656_67B1;
    let round = |acc: u32, lane: u32| {
        acc.wrapping_add(lane.wrapping_mul(P2))
            .rotate_left(13)
            .wrapping_mul(P1)
    };
    let word = |bytes: &[u8]| u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let mut rest = data;
    let mut hash = if data.len() >= 16 {
        let mut lanes = [
            seed.wrapping_add(P1).wrapping_add(P2),
            seed.wrapping_add(P2),
            seed,
            seed.wrapping_sub(P1),
        ];
        while rest.len() >= 16 {
            for (index, lane) in lanes.iter_mut().enumerate() {
                *lane = round(*lane, word(&rest[index * 4..]));
            }
            rest = &rest[16..];
        }
        lanes[0]
            .rotate_left(1)
            .wrapping_add(lanes[1].rotate_left(7))
            .wrapping_add(lanes[2].rotate_left(12))
            .wrapping_add(lanes[3].rotate_left(18))
    } else {
        seed.wrapping_add(P5)
    };
    hash = hash.wrapping_add(data.len() as u32);
    while rest.len() >= 4 {
        hash = hash
            .wrapping_add(word(rest).wrapping_mul(P3))
            .rotate_left(17)
            .wrapping_mul(P4);
        rest = &rest[4..];
    }
    for byte in rest {
        hash = hash
            .wrapping_add(u32::from(*byte).wrapping_mul(P5))
            .rotate_left(11)
            .wrapping_mul(P1);
    }
    hash ^= hash >> 15;
    hash = hash.wrapping_mul(P2);
    hash ^= hash >> 13;
    hash = hash.wrapping_mul(P3);
    hash ^ (hash >> 16)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unhex(text: &str) -> Vec<u8> {
        (0..text.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
            .collect()
    }

    const SMALL_IN: &str = "68656c6c6f2068656c6c6f2068656c6c6f2068656c6c6f2068656c6c6f2068656c6c6f2068656c6c6f2068656c6c6f";
    const SMALL_OUT: &str = "4c5a34426c6f636b26100000002f000000c62078086f68656c6c6f200600115068656c6c6f4c5a34426c6f636b16000000000000000000000000";
    const RANDOM_IN: &str =
        "f837828554d61ba6121517b16510d50c57ef245dfaf4131b569111587dfd60f911309acf4213b6c6";
    const RANDOM_OUT: &str = "4c5a34426c6f636b162800000028000000fcf3cb09f837828554d61ba6121517b16510d50c57ef245dfaf4131b569111587dfd60f911309acf4213b6c64c5a34426c6f636b16000000000000000000000000";
    const EMPTY_OUT: &str = "4c5a34426c6f636b16000000000000000000000000";
    const PERIODIC_OUT: &str = "4c5a34426c6f636b26110100000000010030ac66017f000102030405060700ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe15004050600014c5a34426c6f636b262200000070110000e01cfa097f020304050600010700ffffffffffffffffffffffffffffffffff625002030405064c5a34426c6f636b16000000000000000000000000";

    fn periodic() -> Vec<u8> {
        (0..70000).map(|i| (i % 7) as u8).collect()
    }

    #[test]
    fn xxh32_matches_reference_vectors() {
        assert_eq!(xxh32(b"", 0), 0x02CC_5D05);
        assert_eq!(xxh32(b"abc", 0), 0x32D1_53FF);
        assert_eq!(
            xxh32(b"Nobody inspects the spammish repetition", 0),
            0xE229_3B2F
        );
        assert_eq!(xxh32(&unhex(SMALL_IN), 0x9747_B28C), 0x5878_20C6);
    }

    #[test]
    fn ids_follow_region_file_version() {
        assert_eq!(Compression::Gzip.id(), 1);
        assert_eq!(Compression::Zlib.id(), 2);
        assert_eq!(Compression::Uncompressed.id(), 3);
        assert_eq!(Compression::Lz4.id(), 4);
        for compression in [
            Compression::Gzip,
            Compression::Zlib,
            Compression::Uncompressed,
            Compression::Lz4,
        ] {
            assert_eq!(Compression::from_id(compression.id()), Some(compression));
        }
        assert_eq!(Compression::from_id(0), None);
        assert_eq!(Compression::from_id(127), None);
    }

    #[test]
    fn property_names_match_server_properties() {
        assert_eq!(
            Compression::from_property("deflate"),
            Some(Compression::Zlib)
        );
        assert_eq!(
            Compression::from_property("none"),
            Some(Compression::Uncompressed)
        );
        assert_eq!(Compression::from_property("lz4"), Some(Compression::Lz4));
        assert_eq!(Compression::from_property("zstd"), None);
        assert_eq!(Compression::default(), Compression::Zlib);
    }

    #[test]
    fn every_codec_round_trips() {
        let samples: Vec<Vec<u8>> = vec![
            Vec::new(),
            b"x".to_vec(),
            unhex(SMALL_IN),
            unhex(RANDOM_IN),
            periodic(),
            (0..200_000u32)
                .map(|i| (i.wrapping_mul(2_654_435_761) >> 13) as u8)
                .collect(),
        ];
        for compression in [
            Compression::Gzip,
            Compression::Zlib,
            Compression::Uncompressed,
            Compression::Lz4,
        ] {
            for sample in &samples {
                let packed = compression.compress(sample).unwrap();
                assert_eq!(&compression.decompress(&packed).unwrap(), sample);
            }
        }
    }

    #[test]
    fn zlib_output_has_zlib_header() {
        let packed = Compression::Zlib.compress(b"hello hello hello").unwrap();
        assert_eq!(packed[0], 0x78);
        let packed = Compression::Gzip.compress(b"hello hello hello").unwrap();
        assert_eq!(&packed[..2], &[0x1f, 0x8b]);
    }

    #[test]
    fn lz4_reads_lz4_java_streams() {
        assert_eq!(
            Compression::Lz4.decompress(&unhex(SMALL_OUT)).unwrap(),
            unhex(SMALL_IN)
        );
        assert_eq!(
            Compression::Lz4.decompress(&unhex(RANDOM_OUT)).unwrap(),
            unhex(RANDOM_IN)
        );
        assert_eq!(
            Compression::Lz4.decompress(&unhex(EMPTY_OUT)).unwrap(),
            Vec::<u8>::new()
        );
        assert_eq!(
            Compression::Lz4.decompress(&unhex(PERIODIC_OUT)).unwrap(),
            periodic()
        );
    }

    #[test]
    fn lz4_accepts_a_stream_without_the_end_marker() {
        let full = unhex(SMALL_OUT);
        let truncated = &full[..full.len() - 21];
        assert_eq!(
            Compression::Lz4.decompress(truncated).unwrap(),
            unhex(SMALL_IN)
        );
    }

    #[test]
    fn lz4_writer_emits_the_lz4_java_framing() {
        let packed = Compression::Lz4.compress(&unhex(RANDOM_IN)).unwrap();
        assert_eq!(packed, unhex(RANDOM_OUT));
        let packed = Compression::Lz4.compress(&[]).unwrap();
        assert_eq!(packed, unhex(EMPTY_OUT));
        let packed = Compression::Lz4.compress(&periodic()).unwrap();
        assert_eq!(&packed[..9], b"LZ4Block\x26");
        assert_eq!(&packed[packed.len() - 21..], &unhex(EMPTY_OUT)[..]);
    }

    #[test]
    fn lz4_rejects_bad_checksum_and_magic() {
        let mut broken = unhex(SMALL_OUT);
        broken[14] ^= 1;
        assert!(Compression::Lz4.decompress(&broken).is_err());
        let mut broken = unhex(SMALL_OUT);
        broken[0] = b'X';
        assert!(Compression::Lz4.decompress(&broken).is_err());
        assert!(
            Compression::Lz4
                .decompress(&unhex(SMALL_OUT)[..30])
                .is_err()
        );
    }

    #[test]
    fn corrupt_zlib_is_an_error() {
        assert!(
            Compression::Zlib
                .decompress(&[0x78, 0x9c, 1, 2, 3])
                .is_err()
        );
    }
}
