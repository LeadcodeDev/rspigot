#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Color {
    pub const fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    pub const fn to_rgb(self) -> u32 {
        ((self.red as u32) << 16) | ((self.green as u32) << 8) | self.blue as u32
    }
}

#[cfg(test)]
mod tests {
    use super::Color;

    #[test]
    fn to_rgb_packs_red_green_blue() {
        assert_eq!(Color::rgb(0x12, 0x34, 0x56).to_rgb(), 0x12_34_56);
        assert_eq!(Color::rgb(255, 255, 255).to_rgb(), 0xff_ff_ff);
    }
}
