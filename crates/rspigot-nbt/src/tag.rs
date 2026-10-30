use crate::compound::Compound;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TagType {
    End = 0,
    Byte = 1,
    Short = 2,
    Int = 3,
    Long = 4,
    Float = 5,
    Double = 6,
    ByteArray = 7,
    String = 8,
    List = 9,
    Compound = 10,
    IntArray = 11,
    LongArray = 12,
}

impl TagType {
    pub fn from_id(id: u8) -> Option<Self> {
        Some(match id {
            0 => Self::End,
            1 => Self::Byte,
            2 => Self::Short,
            3 => Self::Int,
            4 => Self::Long,
            5 => Self::Float,
            6 => Self::Double,
            7 => Self::ByteArray,
            8 => Self::String,
            9 => Self::List,
            10 => Self::Compound,
            11 => Self::IntArray,
            12 => Self::LongArray,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Tag {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<u8>),
    String(String),
    List(Vec<Tag>),
    Compound(Compound),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

impl Tag {
    pub fn tag_type(&self) -> TagType {
        match self {
            Self::Byte(_) => TagType::Byte,
            Self::Short(_) => TagType::Short,
            Self::Int(_) => TagType::Int,
            Self::Long(_) => TagType::Long,
            Self::Float(_) => TagType::Float,
            Self::Double(_) => TagType::Double,
            Self::ByteArray(_) => TagType::ByteArray,
            Self::String(_) => TagType::String,
            Self::List(_) => TagType::List,
            Self::Compound(_) => TagType::Compound,
            Self::IntArray(_) => TagType::IntArray,
            Self::LongArray(_) => TagType::LongArray,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Byte(v) => Some(i64::from(*v)),
            Self::Short(v) => Some(i64::from(*v)),
            Self::Int(v) => Some(i64::from(*v)),
            Self::Long(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Float(v) => Some(f64::from(*v)),
            Self::Double(v) => Some(*v),
            other => other.as_i64().map(|v| v as f64),
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_compound(&self) -> Option<&Compound> {
        match self {
            Self::Compound(c) => Some(c),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&[Tag]> {
        match self {
            Self::List(l) => Some(l),
            _ => None,
        }
    }
}

impl From<i8> for Tag {
    fn from(v: i8) -> Self {
        Self::Byte(v)
    }
}

impl From<bool> for Tag {
    fn from(v: bool) -> Self {
        Self::Byte(i8::from(v))
    }
}

impl From<i16> for Tag {
    fn from(v: i16) -> Self {
        Self::Short(v)
    }
}

impl From<i32> for Tag {
    fn from(v: i32) -> Self {
        Self::Int(v)
    }
}

impl From<i64> for Tag {
    fn from(v: i64) -> Self {
        Self::Long(v)
    }
}

impl From<f32> for Tag {
    fn from(v: f32) -> Self {
        Self::Float(v)
    }
}

impl From<f64> for Tag {
    fn from(v: f64) -> Self {
        Self::Double(v)
    }
}

impl From<&str> for Tag {
    fn from(v: &str) -> Self {
        Self::String(v.to_owned())
    }
}

impl From<String> for Tag {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}

impl From<Compound> for Tag {
    fn from(v: Compound) -> Self {
        Self::Compound(v)
    }
}

impl From<Vec<i64>> for Tag {
    fn from(v: Vec<i64>) -> Self {
        Self::LongArray(v)
    }
}

impl From<Vec<i32>> for Tag {
    fn from(v: Vec<i32>) -> Self {
        Self::IntArray(v)
    }
}
