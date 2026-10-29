use crate::codec::{Decode, DecodeError, Encode, Position, VarInt, decode_len, take};
use crate::play::wire::{FieldCodec, Var, read_var, wire_enum, wire_struct, write_var};
use crate::types::BitSet;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub const ZERO: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

impl Encode for Vec3 {
    fn encode(&self, out: &mut Vec<u8>) {
        self.x.encode(out);
        self.y.encode(out);
        self.z.encode(out);
    }
}

impl Decode for Vec3 {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self {
            x: f64::decode(input)?,
            y: f64::decode(input)?,
            z: f64::decode(input)?,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LpVec3(pub Vec3);

impl LpVec3 {
    const MIN_MAGNITUDE: f64 = 3.051944088384301e-5;
    const MAX_COMPONENT: f64 = 17179869183.0;
    const STEPS: f64 = 32766.0;

    fn sanitize(value: f64) -> f64 {
        if value.is_nan() {
            0.0
        } else {
            value.clamp(-Self::MAX_COMPONENT, Self::MAX_COMPONENT)
        }
    }

    fn pack(value: f64) -> i64 {
        ((value * 0.5 + 0.5) * Self::STEPS).round() as i64
    }

    fn unpack(value: i64) -> f64 {
        ((value & 0x7FFF) as f64).min(Self::STEPS) * 2.0 / Self::STEPS - 1.0
    }
}

impl Encode for LpVec3 {
    fn encode(&self, out: &mut Vec<u8>) {
        let x = Self::sanitize(self.0.x);
        let y = Self::sanitize(self.0.y);
        let z = Self::sanitize(self.0.z);
        let magnitude = x.abs().max(y.abs().max(z.abs()));
        if magnitude < Self::MIN_MAGNITUDE {
            out.push(0);
            return;
        }
        let scale = magnitude.ceil() as i64;
        let continued = (scale & 3) != scale;
        let markers = if continued { (scale & 3) | 4 } else { scale };
        let divisor = scale as f64;
        let packed = markers
            | (Self::pack(x / divisor) << 3)
            | (Self::pack(y / divisor) << 18)
            | (Self::pack(z / divisor) << 33);
        out.push(packed as u8);
        out.push((packed >> 8) as u8);
        ((packed >> 16) as i32).encode(out);
        if continued {
            write_var((scale >> 2) as i32, out);
        }
    }
}

impl Decode for LpVec3 {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        let lowest = u8::decode(input)?;
        if lowest == 0 {
            return Ok(Self(Vec3::ZERO));
        }
        let middle = u8::decode(input)?;
        let highest = u32::decode(input)?;
        let packed = (i64::from(highest) << 16) | (i64::from(middle) << 8) | i64::from(lowest);
        let mut scale = i64::from(lowest & 3);
        if lowest & 4 != 0 {
            scale |= i64::from(read_var(input)? as u32) << 2;
        }
        let scale = scale as f64;
        Ok(Self(Vec3 {
            x: Self::unpack(packed >> 3) * scale,
            y: Self::unpack(packed >> 18) * scale,
            z: Self::unpack(packed >> 33) * scale,
        }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ByteBitSet(pub BitSet);

impl ByteBitSet {
    pub fn from_bits(bits: impl IntoIterator<Item = usize>) -> Self {
        let mut set = BitSet::default();
        for bit in bits {
            set.set(bit);
        }
        Self(set)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = self.0.0.iter().flat_map(|w| w.to_le_bytes()).collect();
        while bytes.last() == Some(&0) {
            bytes.pop();
        }
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut words: Vec<i64> = bytes
            .chunks(8)
            .map(|chunk| {
                let mut word = [0u8; 8];
                word[..chunk.len()].copy_from_slice(chunk);
                i64::from_le_bytes(word)
            })
            .collect();
        while words.last() == Some(&0) {
            words.pop();
        }
        Self(BitSet(words))
    }
}

impl Encode for ByteBitSet {
    fn encode(&self, out: &mut Vec<u8>) {
        let bytes = self.to_bytes();
        crate::codec::encode_len(bytes.len(), out);
        out.extend_from_slice(&bytes);
    }
}

impl Decode for ByteBitSet {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        let len = decode_len(input, 1 << 21)?;
        Ok(Self::from_bytes(take(input, len)?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Holder<T> {
    Reference(i32),
    Direct(T),
}

impl<T: Encode> Encode for Holder<T> {
    fn encode(&self, out: &mut Vec<u8>) {
        match self {
            Self::Reference(id) => write_var(id.wrapping_add(1), out),
            Self::Direct(value) => {
                write_var(0, out);
                value.encode(out);
            }
        }
    }
}

impl<T: Decode> Decode for Holder<T> {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        match read_var(input)? {
            0 => T::decode(input).map(Self::Direct),
            id => Ok(Self::Reference(id.wrapping_sub(1))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HolderSet {
    Tag(String),
    Ids(Vec<i32>),
}

impl Encode for HolderSet {
    fn encode(&self, out: &mut Vec<u8>) {
        match self {
            Self::Tag(tag) => {
                write_var(0, out);
                tag.encode(out);
            }
            Self::Ids(ids) => {
                write_var(ids.len() as i32 + 1, out);
                for id in ids {
                    write_var(*id, out);
                }
            }
        }
    }
}

impl Decode for HolderSet {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        let marker = read_var(input)?;
        if marker == 0 {
            return String::decode(input).map(Self::Tag);
        }
        let len = usize::try_from(marker.wrapping_sub(1))
            .map_err(|_| DecodeError::NegativeLength(marker))?;
        if len > crate::codec::MAX_LIST_LEN {
            return Err(DecodeError::TooLong {
                len,
                max: crate::codec::MAX_LIST_LEN,
            });
        }
        let mut ids = Vec::with_capacity(len.min(input.len()));
        for _ in 0..len {
            ids.push(read_var(input)?);
        }
        Ok(Self::Ids(ids))
    }
}

wire_struct! {
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct GlobalPos {
        pub dimension: String,
        pub pos: Position,
    }
}

wire_struct! {
    #[derive(Debug, Clone, Copy, PartialEq, Default)]
    pub struct PositionMoveRotation {
        pub position: Vec3,
        pub delta: Vec3,
        pub y_rot: f32,
        pub x_rot: f32,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Relatives(pub u32);

impl Relatives {
    pub const NONE: Self = Self(0);
    pub const X: Self = Self(1);
    pub const Y: Self = Self(1 << 1);
    pub const Z: Self = Self(1 << 2);
    pub const Y_ROT: Self = Self(1 << 3);
    pub const X_ROT: Self = Self(1 << 4);
    pub const DELTA_X: Self = Self(1 << 5);
    pub const DELTA_Y: Self = Self(1 << 6);
    pub const DELTA_Z: Self = Self(1 << 7);
    pub const ROTATE_DELTA: Self = Self(1 << 8);

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl Encode for Relatives {
    fn encode(&self, out: &mut Vec<u8>) {
        (self.0 as i32).encode(out);
    }
}

impl Decode for Relatives {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        Ok(Self(i32::decode(input)? as u32))
    }
}

wire_struct! {
    #[derive(Debug, Clone, PartialEq)]
    pub struct SoundEvent {
        pub location: String,
        pub fixed_range: Option<f32>,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    pub const fn pack(self) -> i64 {
        (self.x as u32 as i64) | ((self.z as u32 as i64) << 32)
    }

    pub const fn unpack(packed: i64) -> Self {
        Self {
            x: packed as i32,
            z: (packed >> 32) as i32,
        }
    }
}

impl Encode for ChunkPos {
    fn encode(&self, out: &mut Vec<u8>) {
        self.pack().encode(out);
    }
}

impl Decode for ChunkPos {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        i64::decode(input).map(Self::unpack)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct SectionPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl SectionPos {
    pub const fn pack(self) -> i64 {
        ((self.x as i64 & 0x3F_FFFF) << 42)
            | (self.y as i64 & 0xF_FFFF)
            | ((self.z as i64 & 0x3F_FFFF) << 20)
    }

    pub const fn unpack(packed: i64) -> Self {
        Self {
            x: (packed >> 42) as i32,
            y: ((packed << 44) >> 44) as i32,
            z: ((packed << 22) >> 42) as i32,
        }
    }
}

impl Encode for SectionPos {
    fn encode(&self, out: &mut Vec<u8>) {
        self.pack().encode(out);
    }
}

impl Decode for SectionPos {
    fn decode(input: &mut &[u8]) -> Result<Self, DecodeError> {
        i64::decode(input).map(Self::unpack)
    }
}

wire_enum! {
    #[derive(Default)]
    pub enum GameMode: Zero {
        #[default]
        Survival,
        Creative,
        Adventure,
        Spectator,
    }
}

pub struct OptionalGameMode;

impl FieldCodec<Option<GameMode>> for OptionalGameMode {
    fn write(value: &Option<GameMode>, out: &mut Vec<u8>) {
        write_var(value.map_or(0, |mode| mode.id() + 1), out);
    }

    fn read(input: &mut &[u8]) -> Result<Option<GameMode>, DecodeError> {
        Ok(match read_var(input)? {
            0 => None,
            id => Some(GameMode::from_id(id.wrapping_sub(1)).unwrap_or(GameMode::Survival)),
        })
    }
}

wire_enum! {
    pub enum Difficulty: Wrap { Peaceful, Easy, Normal, Hard }
}

wire_enum! {
    pub enum Hand: Zero { MainHand, OffHand }
}

wire_enum! {
    pub enum Direction: Wrap { Down, Up, North, South, West, East }
}

wire_enum! {
    pub enum Pose: Zero {
        Standing,
        FallFlying,
        Sleeping,
        Swimming,
        SpinAttack,
        Crouching,
        LongJumping,
        Dying,
        Croaking,
        UsingTongue,
        Sitting,
        Roaring,
        Sniffing,
        Emerging,
        Digging,
        Sliding,
        Shooting,
        Inhaling,
    }
}

wire_enum! {
    pub enum SoundSource: Strict {
        Master,
        Music,
        Records,
        Weather,
        Blocks,
        Hostile,
        Neutral,
        Players,
        Ambient,
        Voice,
        Ui,
    }
}

pub struct OptionalVarInt;

impl FieldCodec<Option<i32>> for OptionalVarInt {
    fn write(value: &Option<i32>, out: &mut Vec<u8>) {
        write_var(value.map_or(0, |v| v.wrapping_add(1)), out);
    }

    fn read(input: &mut &[u8]) -> Result<Option<i32>, DecodeError> {
        Ok(match read_var(input)? {
            0 => None,
            v => Some(v.wrapping_sub(1)),
        })
    }
}

pub fn encode_var_ids(ids: &[i32], out: &mut Vec<u8>) {
    crate::codec::encode_len(ids.len(), out);
    for id in ids {
        Var::write(id, out);
    }
}

pub fn decode_var_ids(input: &mut &[u8], max: usize) -> Result<Vec<i32>, DecodeError> {
    let len = decode_len(input, max)?;
    let mut ids = Vec::with_capacity(len.min(input.len()));
    for _ in 0..len {
        ids.push(VarInt::decode(input)?.0);
    }
    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codec::decode_exact;
    use crate::play::wire::testing::{encoded, roundtrip};

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    #[test]
    fn lp_vec3_matches_vanilla_packing() {
        assert_eq!(encoded(&LpVec3(Vec3::ZERO)), vec![0]);
        assert_eq!(
            hex(&encoded(&LpVec3(Vec3::new(1.0, 0.0, 0.0)))),
            "f1ff7ffeffff"
        );
        assert_eq!(
            hex(&encoded(&LpVec3(Vec3::new(0.5, -0.25, 0.0)))),
            "f9ff7ffebffe"
        );
        assert_eq!(
            hex(&encoded(&LpVec3(Vec3::new(10.0, 0.0, -3.0)))),
            "f6ff5998ffff02"
        );
    }

    #[test]
    fn lp_vec3_decodes_within_precision() {
        for value in [
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.5, -0.25, 0.0),
            Vec3::new(10.0, 0.0, -3.0),
            Vec3::new(-123.5, 64.25, 0.125),
        ] {
            let decoded = decode_exact::<LpVec3>(&encoded(&LpVec3(value))).unwrap().0;
            let scale = value.x.abs().max(value.y.abs()).max(value.z.abs()).ceil();
            let tolerance = scale / 32766.0 * 1.01;
            assert!((decoded.x - value.x).abs() <= tolerance);
            assert!((decoded.y - value.y).abs() <= tolerance);
            assert!((decoded.z - value.z).abs() <= tolerance);
        }
        assert_eq!(decode_exact::<LpVec3>(&[0]).unwrap(), LpVec3(Vec3::ZERO));
    }

    #[test]
    fn byte_bit_set_trims_like_java_to_byte_array() {
        assert_eq!(encoded(&ByteBitSet::default()), vec![0]);
        assert_eq!(encoded(&ByteBitSet::from_bits([0, 9])), vec![2, 0x01, 0x02]);
        let all = ByteBitSet::from_bits(0..26);
        assert_eq!(encoded(&all), vec![4, 0xFF, 0xFF, 0xFF, 0x03]);
        roundtrip(&all);
        roundtrip(&ByteBitSet::from_bits([70]));
    }

    #[test]
    fn holder_and_holder_set_markers() {
        assert_eq!(roundtrip(&Holder::<SoundEvent>::Reference(4)), vec![5]);
        let direct = Holder::Direct(SoundEvent {
            location: "a:b".into(),
            fixed_range: Some(16.0),
        });
        assert_eq!(
            roundtrip(&direct),
            vec![0, 3, b'a', b':', b'b', 1, 0x41, 0x80, 0, 0]
        );
        assert_eq!(roundtrip(&HolderSet::Ids(vec![1, 2])), vec![3, 1, 2]);
        assert_eq!(
            roundtrip(&HolderSet::Tag("x:y".into())),
            vec![0, 3, b'x', b':', b'y']
        );
    }

    #[test]
    fn packed_positions_match_vanilla() {
        let chunk = ChunkPos { x: -1, z: 2 };
        assert_eq!(roundtrip(&chunk), vec![0, 0, 0, 2, 0xFF, 0xFF, 0xFF, 0xFF]);
        let section = SectionPos { x: -3, y: -4, z: 5 };
        assert_eq!(section.pack(), (0x3F_FFFDi64 << 42) | (5 << 20) | 0xF_FFFC);
        roundtrip(&section);
        roundtrip(&SectionPos {
            x: 2_097_151,
            y: -524_288,
            z: -2_097_152,
        });
    }

    #[test]
    fn relatives_and_optional_game_mode() {
        let flags = Relatives::X.union(Relatives::ROTATE_DELTA);
        assert_eq!(roundtrip(&flags), vec![0, 0, 1, 1]);
        assert!(flags.contains(Relatives::X));
        let mut out = Vec::new();
        OptionalGameMode::write(&None, &mut out);
        OptionalGameMode::write(&Some(GameMode::Spectator), &mut out);
        assert_eq!(out, vec![0, 4]);
        let mut input = out.as_slice();
        assert_eq!(OptionalGameMode::read(&mut input).unwrap(), None);
        assert_eq!(
            OptionalGameMode::read(&mut input).unwrap(),
            Some(GameMode::Spectator)
        );
    }

    #[test]
    fn enums_use_vanilla_ids() {
        assert_eq!(encoded(&Direction::East), vec![5]);
        assert_eq!(encoded(&Pose::Inhaling), vec![17]);
        assert_eq!(encoded(&SoundSource::Ui), vec![10]);
        assert_eq!(decode_exact::<Difficulty>(&[5]).unwrap(), Difficulty::Easy);
        assert!(decode_exact::<SoundSource>(&[11]).is_err());
    }
}
