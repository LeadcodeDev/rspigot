use aes::Aes128;
use cfb8::cipher::consts::U1;
use cfb8::cipher::inout::InOutBuf;
use cfb8::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use flate2::{Compress, Decompress, FlushCompress, FlushDecompress, Status};

use crate::codec::{Decode, DecodeError, Encode, VarInt};

pub const MAX_FRAME_LEN: usize = 2_097_151;
pub const MAX_DECOMPRESSED_LEN: usize = 8_388_608;
const MAX_FRAME_LEN_BYTES: usize = 3;

type Encryptor = cfb8::Encryptor<Aes128>;
type Decryptor = cfb8::Decryptor<Aes128>;

#[derive(Debug, thiserror::Error)]
pub enum FrameError {
    #[error("frame length is wider than 21 bits")]
    LengthTooWide,
    #[error("frame length cannot be zero")]
    EmptyFrame,
    #[error("frame of {0} bytes exceeds the maximum of {MAX_FRAME_LEN}")]
    FrameTooLarge(usize),
    #[error("packet of {0} bytes exceeds the maximum of {MAX_DECOMPRESSED_LEN}")]
    PacketTooLarge(usize),
    #[error("badly compressed packet: size {size} is below the threshold {threshold}")]
    BelowThreshold { size: i32, threshold: usize },
    #[error("badly compressed packet: size {0} exceeds the maximum of {MAX_DECOMPRESSED_LEN}")]
    DecompressedTooLarge(usize),
    #[error("badly compressed packet: inflated {actual} bytes, expected {expected}")]
    DecompressedLengthMismatch { expected: usize, actual: usize },
    #[error("invalid data length: {0}")]
    DataLength(#[from] DecodeError),
    #[error("zlib compression failed: {0}")]
    Compress(#[from] flate2::CompressError),
    #[error("zlib decompression failed: {0}")]
    Decompress(#[from] flate2::DecompressError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Compression {
    pub threshold: usize,
    pub level: u32,
}

pub fn build_frame(
    packet: &[u8],
    compression: Option<Compression>,
    out: &mut Vec<u8>,
) -> Result<(), FrameError> {
    write_frame(
        packet,
        compression,
        &mut Deflater::default(),
        &mut Vec::new(),
        out,
    )
}

fn write_frame(
    packet: &[u8],
    compression: Option<Compression>,
    deflater: &mut Deflater,
    scratch: &mut Vec<u8>,
    out: &mut Vec<u8>,
) -> Result<(), FrameError> {
    let Some(compression) = compression else {
        return write_length_prefixed(&[], packet, out);
    };
    if packet.len() > MAX_DECOMPRESSED_LEN {
        return Err(FrameError::PacketTooLarge(packet.len()));
    }
    if packet.len() < compression.threshold {
        return write_length_prefixed(&[0], packet, out);
    }
    scratch.clear();
    VarInt(packet.len() as i32).encode(scratch);
    deflater.deflate(compression.level, packet, scratch)?;
    write_length_prefixed(&[], scratch, out)
}

fn write_length_prefixed(head: &[u8], body: &[u8], out: &mut Vec<u8>) -> Result<(), FrameError> {
    let len = head.len() + body.len();
    if len > MAX_FRAME_LEN {
        return Err(FrameError::FrameTooLarge(len));
    }
    out.reserve(MAX_FRAME_LEN_BYTES + len);
    VarInt(len as i32).encode(out);
    out.extend_from_slice(head);
    out.extend_from_slice(body);
    Ok(())
}

#[derive(Default)]
struct Deflater {
    state: Option<(u32, Compress)>,
}

impl Deflater {
    fn deflate(&mut self, level: u32, input: &[u8], out: &mut Vec<u8>) -> Result<(), FrameError> {
        let compress = match &mut self.state {
            Some((current, compress)) if *current == level => {
                compress.reset();
                compress
            }
            slot => {
                &mut slot
                    .insert((level, Compress::new(flate2::Compression::new(level), true)))
                    .1
            }
        };
        loop {
            let consumed = compress.total_in() as usize;
            out.reserve((input.len() - consumed) / 2 + 64);
            let status = compress.compress_vec(&input[consumed..], out, FlushCompress::Finish)?;
            if status == Status::StreamEnd {
                return Ok(());
            }
        }
    }
}

fn read_frame_length(available: &[u8]) -> Result<Option<(usize, usize)>, FrameError> {
    let mut len = 0usize;
    for (index, byte) in available.iter().take(MAX_FRAME_LEN_BYTES).enumerate() {
        len |= usize::from(byte & 0x7F) << (7 * index);
        if byte & 0x80 == 0 {
            if len == 0 {
                return Err(FrameError::EmptyFrame);
            }
            return Ok(Some((len, index + 1)));
        }
    }
    if available.len() >= MAX_FRAME_LEN_BYTES {
        Err(FrameError::LengthTooWide)
    } else {
        Ok(None)
    }
}

fn inflate(
    mut frame: &[u8],
    threshold: usize,
    inflater: &mut Option<Decompress>,
) -> Result<Vec<u8>, FrameError> {
    let data_len = VarInt::decode(&mut frame)?.0;
    if data_len == 0 {
        return Ok(frame.to_vec());
    }
    let expected = usize::try_from(data_len)
        .ok()
        .filter(|len| *len >= threshold)
        .ok_or(FrameError::BelowThreshold {
            size: data_len,
            threshold,
        })?;
    if expected > MAX_DECOMPRESSED_LEN {
        return Err(FrameError::DecompressedTooLarge(expected));
    }
    let inflater = inflater.get_or_insert_with(|| Decompress::new(true));
    inflater.reset(true);
    let mut packet = vec![0u8; expected];
    loop {
        let read = inflater.total_in() as usize;
        let written = inflater.total_out() as usize;
        let status = inflater.decompress(
            &frame[read..],
            &mut packet[written..],
            FlushDecompress::Finish,
        )?;
        let produced = inflater.total_out() as usize;
        let stalled = inflater.total_in() as usize == read && produced == written;
        if status == Status::StreamEnd || stalled || produced == expected {
            break;
        }
    }
    let actual = inflater.total_out() as usize;
    if actual != expected {
        return Err(FrameError::DecompressedLengthMismatch { expected, actual });
    }
    Ok(packet)
}

fn encrypt(encryptor: &mut Option<Encryptor>, data: &mut [u8]) {
    if let Some(encryptor) = encryptor {
        let (blocks, _) = InOutBuf::from(data).into_chunks::<U1>();
        encryptor.encrypt_blocks_inout_mut(blocks);
    }
}

fn decrypt(decryptor: &mut Option<Decryptor>, data: &mut [u8]) {
    if let Some(decryptor) = decryptor {
        let (blocks, _) = InOutBuf::from(data).into_chunks::<U1>();
        decryptor.decrypt_blocks_inout_mut(blocks);
    }
}

#[derive(Default)]
pub struct FrameEncoder {
    compression: Option<Compression>,
    encryptor: Option<Encryptor>,
    deflater: Deflater,
    scratch: Vec<u8>,
}

impl FrameEncoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_compression(&mut self, compression: Option<Compression>) {
        self.compression = compression;
    }

    pub fn enable_encryption(&mut self, shared_secret: &[u8; 16]) {
        self.encryptor = Some(Encryptor::new(shared_secret.into(), shared_secret.into()));
    }

    pub fn encode_packet(&mut self, packet: &[u8], out: &mut Vec<u8>) -> Result<(), FrameError> {
        let start = out.len();
        write_frame(
            packet,
            self.compression,
            &mut self.deflater,
            &mut self.scratch,
            out,
        )?;
        encrypt(&mut self.encryptor, &mut out[start..]);
        Ok(())
    }

    pub fn append_prebuilt(&mut self, frame: &[u8], out: &mut Vec<u8>) {
        let start = out.len();
        out.extend_from_slice(frame);
        encrypt(&mut self.encryptor, &mut out[start..]);
    }
}

#[derive(Default)]
pub struct FrameDecoder {
    buffer: Vec<u8>,
    consumed: usize,
    threshold: Option<usize>,
    decryptor: Option<Decryptor>,
    inflater: Option<Decompress>,
}

impl FrameDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_compression(&mut self, threshold: Option<usize>) {
        self.threshold = threshold;
    }

    pub fn enable_encryption(&mut self, shared_secret: &[u8; 16]) {
        self.decryptor = Some(Decryptor::new(shared_secret.into(), shared_secret.into()));
        decrypt(&mut self.decryptor, &mut self.buffer[self.consumed..]);
    }

    pub fn feed(&mut self, data: &[u8]) {
        self.buffer.drain(..self.consumed);
        self.consumed = 0;
        let start = self.buffer.len();
        self.buffer.extend_from_slice(data);
        decrypt(&mut self.decryptor, &mut self.buffer[start..]);
    }

    pub fn next_packet(&mut self) -> Result<Option<Vec<u8>>, FrameError> {
        let available = &self.buffer[self.consumed..];
        let Some((frame_len, prefix_len)) = read_frame_length(available)? else {
            return Ok(None);
        };
        if available.len() - prefix_len < frame_len {
            return Ok(None);
        }
        let start = self.consumed + prefix_len;
        let end = start + frame_len;
        self.consumed = end;
        let frame = &self.buffer[start..end];
        match self.threshold {
            None => Ok(Some(frame.to_vec())),
            Some(threshold) => inflate(frame, threshold, &mut self.inflater).map(Some),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::{Decode, Encode, VarInt};

    const KEY: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

    fn compression(threshold: usize) -> Option<Compression> {
        Some(Compression {
            threshold,
            level: 6,
        })
    }

    fn frame(packet: &[u8], compression: Option<Compression>) -> Vec<u8> {
        let mut out = Vec::new();
        build_frame(packet, compression, &mut out).unwrap();
        out
    }

    fn drain(decoder: &mut FrameDecoder) -> Vec<Vec<u8>> {
        let mut packets = Vec::new();
        while let Some(packet) = decoder.next_packet().unwrap() {
            packets.push(packet);
        }
        packets
    }

    fn sample_packet(len: usize) -> Vec<u8> {
        (0..len).map(|i| (i % 7) as u8).collect()
    }

    #[test]
    fn uncompressed_frame_is_length_prefixed() {
        assert_eq!(
            frame(&[0x00, 0x01, 0x02], None),
            vec![0x03, 0x00, 0x01, 0x02]
        );
        let big = sample_packet(300);
        let built = frame(&big, None);
        assert_eq!(&built[..2], &[0xAC, 0x02]);
        assert_eq!(&built[2..], big.as_slice());
    }

    #[test]
    fn compressed_frame_below_threshold_has_zero_data_length() {
        assert_eq!(
            frame(&[0x05, 0xAA], compression(256)),
            vec![0x03, 0x00, 0x05, 0xAA]
        );
    }

    #[test]
    fn compressed_frame_above_threshold_is_zlib() {
        let packet = sample_packet(300);
        let built = frame(&packet, compression(256));
        let mut input = built.as_slice();
        let frame_len = VarInt::decode(&mut input).unwrap().0;
        assert_eq!(frame_len as usize, input.len());
        let data_len = VarInt::decode(&mut input).unwrap().0;
        assert_eq!(data_len, 300);
        assert_eq!(input[0], 0x78);
        assert!(input.len() < packet.len());
        let mut decoder = FrameDecoder::new();
        decoder.set_compression(Some(256));
        decoder.feed(&built);
        assert_eq!(drain(&mut decoder), vec![packet]);
    }

    #[test]
    fn threshold_equal_to_length_compresses() {
        let packet = sample_packet(256);
        let built = frame(&packet, compression(256));
        let prefix = VarInt::decode_partial(&built).unwrap().unwrap().1;
        assert_eq!(&built[prefix..prefix + 2], &[0x80, 0x02]);
    }

    #[test]
    fn decoder_reads_reference_zlib_stream() {
        let mut packet = vec![0x2A];
        packet.extend((0..299).map(|i| (i % 7) as u8));
        let zlib = [
            0x78, 0x9C, 0xD3, 0x62, 0x60, 0x64, 0x62, 0x66, 0x61, 0x65, 0x1B, 0xA5, 0x08, 0x53,
            0x00, 0x3B, 0x67, 0x03, 0xA7,
        ];
        let mut wire = vec![0x15, 0xAC, 0x02];
        wire.extend_from_slice(&zlib);
        let mut decoder = FrameDecoder::new();
        decoder.set_compression(Some(256));
        decoder.feed(&wire);
        assert_eq!(drain(&mut decoder), vec![packet]);
    }

    #[test]
    fn decoder_passes_uncompressed_body_through_when_compression_is_on() {
        let mut decoder = FrameDecoder::new();
        decoder.set_compression(Some(256));
        decoder.feed(&[0x03, 0x00, 0x05, 0xAA]);
        assert_eq!(drain(&mut decoder), vec![vec![0x05, 0xAA]]);
    }

    #[test]
    fn decoder_handles_input_fed_byte_by_byte() {
        let packets = vec![
            vec![0x00],
            sample_packet(200),
            sample_packet(5000),
            vec![0x7F],
        ];
        for compression in [None, compression(64)] {
            let mut wire = Vec::new();
            for packet in &packets {
                build_frame(packet, compression, &mut wire).unwrap();
            }
            let mut decoder = FrameDecoder::new();
            decoder.set_compression(compression.map(|c| c.threshold));
            let mut received = Vec::new();
            for byte in &wire {
                decoder.feed(std::slice::from_ref(byte));
                received.extend(drain(&mut decoder));
            }
            assert_eq!(received, packets);
        }
    }

    #[test]
    fn partial_frame_waits_for_more_bytes() {
        let mut decoder = FrameDecoder::new();
        decoder.feed(&[0x05, 0x00, 0x01]);
        assert!(decoder.next_packet().unwrap().is_none());
        decoder.feed(&[0x02, 0x03, 0x04]);
        assert_eq!(
            decoder.next_packet().unwrap(),
            Some(vec![0x00, 0x01, 0x02, 0x03, 0x04])
        );
        assert!(decoder.next_packet().unwrap().is_none());
    }

    #[test]
    fn encryption_matches_aes_cfb8_reference() {
        let mut encoder = FrameEncoder::new();
        encoder.enable_encryption(&KEY);
        let mut out = Vec::new();
        encoder
            .encode_packet(&[0x00, 0x01, 0x02, 0x03, 0x04], &mut out)
            .unwrap();
        assert_eq!(out, vec![0x0F, 0xB8, 0xAE, 0x4C, 0x7D, 0x30]);
        encoder
            .encode_packet(&[0x01, 0x41, 0x42], &mut out)
            .unwrap();
        assert_eq!(&out[6..], &[0x9A, 0x25, 0x7D, 0x63]);
    }

    #[test]
    fn encrypted_stream_roundtrips() {
        let packets = vec![sample_packet(10), sample_packet(400), sample_packet(3)];
        let mut encoder = FrameEncoder::new();
        encoder.set_compression(compression(128));
        encoder.enable_encryption(&KEY);
        let mut wire = Vec::new();
        let mut plain = Vec::new();
        for packet in &packets {
            encoder.encode_packet(packet, &mut wire).unwrap();
            build_frame(packet, compression(128), &mut plain).unwrap();
        }
        assert_eq!(wire.len(), plain.len());
        assert_ne!(wire, plain);
        let mut decoder = FrameDecoder::new();
        decoder.set_compression(Some(128));
        decoder.enable_encryption(&KEY);
        for chunk in wire.chunks(7) {
            decoder.feed(chunk);
        }
        assert_eq!(drain(&mut decoder), packets);
    }

    #[test]
    fn prebuilt_frame_is_encrypted_like_an_encoded_packet() {
        let packets = vec![sample_packet(50), sample_packet(700)];
        let mut encoding = FrameEncoder::new();
        encoding.set_compression(compression(256));
        encoding.enable_encryption(&KEY);
        let mut prebuilt = FrameEncoder::new();
        prebuilt.set_compression(compression(256));
        prebuilt.enable_encryption(&KEY);
        let mut encoded = Vec::new();
        let mut appended = Vec::new();
        for packet in &packets {
            encoding.encode_packet(packet, &mut encoded).unwrap();
            let shared = frame(packet, compression(256));
            prebuilt.append_prebuilt(&shared, &mut appended);
        }
        assert_eq!(encoded, appended);
    }

    #[test]
    fn prebuilt_frame_without_encryption_is_copied() {
        let shared = frame(&[0x01, 0x02], None);
        let mut encoder = FrameEncoder::new();
        let mut out = vec![0xEE];
        encoder.append_prebuilt(&shared, &mut out);
        assert_eq!(out, vec![0xEE, 0x02, 0x01, 0x02]);
    }

    #[test]
    fn encryption_enabled_mid_buffer_decrypts_pending_bytes() {
        let first = vec![0x02, 0x10];
        let second = sample_packet(40);
        let mut wire = frame(&first, None);
        let mut encoder = FrameEncoder::new();
        encoder.enable_encryption(&KEY);
        encoder.encode_packet(&second, &mut wire).unwrap();
        let mut decoder = FrameDecoder::new();
        decoder.feed(&wire);
        assert_eq!(decoder.next_packet().unwrap(), Some(first));
        decoder.enable_encryption(&KEY);
        assert_eq!(decoder.next_packet().unwrap(), Some(second.clone()));
        let mut more = Vec::new();
        encoder.encode_packet(&second, &mut more).unwrap();
        decoder.feed(&more);
        assert_eq!(drain(&mut decoder), vec![second]);
    }

    #[test]
    fn zero_length_frame_is_rejected() {
        let mut decoder = FrameDecoder::new();
        decoder.feed(&[0x00]);
        assert!(matches!(decoder.next_packet(), Err(FrameError::EmptyFrame)));
    }

    #[test]
    fn frame_length_wider_than_three_bytes_is_rejected() {
        let mut decoder = FrameDecoder::new();
        decoder.feed(&[0x80, 0x80]);
        assert!(decoder.next_packet().unwrap().is_none());
        decoder.feed(&[0x80]);
        assert!(matches!(
            decoder.next_packet(),
            Err(FrameError::LengthTooWide)
        ));
    }

    #[test]
    fn maximum_frame_length_is_accepted() {
        let packet = vec![0x01; MAX_FRAME_LEN];
        let built = frame(&packet, None);
        assert_eq!(&built[..3], &[0xFF, 0xFF, 0x7F]);
        let mut decoder = FrameDecoder::new();
        decoder.feed(&built);
        assert_eq!(
            decoder.next_packet().unwrap().map(|p| p.len()),
            Some(MAX_FRAME_LEN)
        );
    }

    #[test]
    fn oversized_uncompressed_frame_is_rejected() {
        let packet = vec![0x01; MAX_FRAME_LEN + 1];
        let mut out = Vec::new();
        assert!(matches!(
            build_frame(&packet, None, &mut out),
            Err(FrameError::FrameTooLarge(len)) if len == MAX_FRAME_LEN + 1
        ));
        assert!(out.is_empty());
    }

    #[test]
    fn oversized_packet_is_rejected_when_compressing() {
        let packet = vec![0x01; MAX_DECOMPRESSED_LEN + 1];
        let mut out = Vec::new();
        assert!(matches!(
            build_frame(&packet, compression(256), &mut out),
            Err(FrameError::PacketTooLarge(_))
        ));
    }

    #[test]
    fn large_compressible_packet_fits_one_frame() {
        let packet = vec![0x01; MAX_DECOMPRESSED_LEN];
        let built = frame(&packet, compression(256));
        let mut decoder = FrameDecoder::new();
        decoder.set_compression(Some(256));
        decoder.feed(&built);
        assert!(decoder.next_packet().unwrap() == Some(packet));
    }

    #[test]
    fn compressed_size_below_threshold_is_rejected() {
        let mut decoder = FrameDecoder::new();
        decoder.set_compression(Some(256));
        decoder.feed(&[0x03, 0x05, 0x78, 0x9C]);
        assert!(matches!(
            decoder.next_packet(),
            Err(FrameError::BelowThreshold {
                size: 5,
                threshold: 256
            })
        ));
    }

    #[test]
    fn negative_data_length_is_rejected() {
        let mut decoder = FrameDecoder::new();
        decoder.set_compression(Some(0));
        decoder.feed(&[0x05, 0xFF, 0xFF, 0xFF, 0xFF, 0x0F]);
        assert!(matches!(
            decoder.next_packet(),
            Err(FrameError::BelowThreshold { size: -1, .. })
        ));
    }

    #[test]
    fn compressed_size_above_protocol_maximum_is_rejected() {
        let mut wire = Vec::new();
        let mut body = Vec::new();
        VarInt((MAX_DECOMPRESSED_LEN + 1) as i32).encode(&mut body);
        body.extend_from_slice(&[0x78, 0x9C]);
        wire.push(body.len() as u8);
        wire.extend_from_slice(&body);
        let mut decoder = FrameDecoder::new();
        decoder.set_compression(Some(256));
        decoder.feed(&wire);
        assert!(matches!(
            decoder.next_packet(),
            Err(FrameError::DecompressedTooLarge(_))
        ));
    }

    #[test]
    fn declared_size_larger_than_inflated_data_is_rejected() {
        let packet = sample_packet(300);
        let built = frame(&packet, compression(256));
        let prefix = VarInt::decode_partial(&built).unwrap().unwrap().1;
        let mut tampered = built.clone();
        tampered[prefix] = 0xAD;
        let mut decoder = FrameDecoder::new();
        decoder.set_compression(Some(256));
        decoder.feed(&tampered);
        assert!(matches!(
            decoder.next_packet(),
            Err(FrameError::DecompressedLengthMismatch {
                expected: 301,
                actual: 300
            })
        ));
    }

    #[test]
    fn corrupt_zlib_stream_is_rejected() {
        let mut decoder = FrameDecoder::new();
        decoder.set_compression(Some(1));
        decoder.feed(&[0x05, 0x0A, 0x00, 0x01, 0x02, 0x03]);
        assert!(matches!(
            decoder.next_packet(),
            Err(FrameError::Decompress(_))
        ));
    }

    #[test]
    fn compression_can_be_switched_off_again() {
        let mut encoder = FrameEncoder::new();
        encoder.set_compression(compression(0));
        let mut out = Vec::new();
        encoder.encode_packet(&[0x01], &mut out).unwrap();
        encoder.set_compression(None);
        encoder.encode_packet(&[0x01], &mut out).unwrap();
        assert_eq!(&out[out.len() - 2..], &[0x01, 0x01]);
        let mut decoder = FrameDecoder::new();
        decoder.set_compression(Some(0));
        decoder.feed(&out);
        assert_eq!(decoder.next_packet().unwrap(), Some(vec![0x01]));
        decoder.set_compression(None);
        assert_eq!(decoder.next_packet().unwrap(), Some(vec![0x01]));
    }

    proptest::proptest! {
        #[test]
        fn frames_roundtrip_through_any_chunking(
            packets in proptest::collection::vec(proptest::collection::vec(proptest::num::u8::ANY, 1..1500), 1..8),
            threshold in proptest::option::of(0usize..600),
            encrypted in proptest::bool::ANY,
            chunk in 1usize..97,
        ) {
            let mut encoder = FrameEncoder::new();
            encoder.set_compression(threshold.map(|threshold| Compression { threshold, level: 6 }));
            let mut decoder = FrameDecoder::new();
            decoder.set_compression(threshold);
            if encrypted {
                encoder.enable_encryption(&KEY);
                decoder.enable_encryption(&KEY);
            }
            let mut wire = Vec::new();
            for packet in &packets {
                encoder.encode_packet(packet, &mut wire).unwrap();
            }
            let mut received = Vec::new();
            for piece in wire.chunks(chunk) {
                decoder.feed(piece);
                received.extend(drain(&mut decoder));
            }
            proptest::prop_assert_eq!(received, packets);
        }
    }
}
