pub const LIGHT_ARRAY_LEN: usize = 2048;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LightArray(Box<[u8; LIGHT_ARRAY_LEN]>);

impl LightArray {
    pub fn filled(level: u8) -> Self {
        let nibble = level & 0x0F;
        Self(Box::new([nibble | (nibble << 4); LIGHT_ARRAY_LEN]))
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let array: [u8; LIGHT_ARRAY_LEN] = bytes.try_into().ok()?;
        Some(Self(Box::new(array)))
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> u8 {
        let index = (y << 8) | (z << 4) | x;
        (self.0[index >> 1] >> ((index & 1) * 4)) & 0x0F
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, level: u8) {
        let index = (y << 8) | (z << 4) | x;
        let shift = (index & 1) * 4;
        let byte = &mut self.0[index >> 1];
        *byte = (*byte & !(0x0F << shift)) | ((level & 0x0F) << shift);
    }

    pub fn as_bytes(&self) -> &[u8; LIGHT_ARRAY_LEN] {
        &self.0
    }

    pub fn is_uniform(&self, level: u8) -> bool {
        let nibble = level & 0x0F;
        let byte = nibble | (nibble << 4);
        self.0.iter().all(|b| *b == byte)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkLight {
    pub sky: Vec<Option<LightArray>>,
    pub block: Vec<Option<LightArray>>,
}

impl ChunkLight {
    pub fn new(section_count: usize) -> Self {
        Self {
            sky: vec![None; section_count + 2],
            block: vec![None; section_count + 2],
        }
    }

    pub fn light_section_count(&self) -> usize {
        self.sky.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nibble_roundtrip() {
        let mut a = LightArray::filled(0);
        a.set(3, 4, 5, 12);
        a.set(4, 4, 5, 7);
        assert_eq!(a.get(3, 4, 5), 12);
        assert_eq!(a.get(4, 4, 5), 7);
        assert_eq!(a.get(5, 4, 5), 0);
    }
}
