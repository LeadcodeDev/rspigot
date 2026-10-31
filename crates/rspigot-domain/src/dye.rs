use crate::item::ItemStack;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DyeColor {
    pub name: &'static str,
    pub texture: i32,
    pub firework: i32,
}

pub const DYE_COLORS: [DyeColor; 16] = [
    DyeColor {
        name: "white",
        texture: 0xF9FFFE,
        firework: 0xF0F0F0,
    },
    DyeColor {
        name: "orange",
        texture: 0xF9801D,
        firework: 0xEB8844,
    },
    DyeColor {
        name: "magenta",
        texture: 0xC74EBD,
        firework: 0xC354CD,
    },
    DyeColor {
        name: "light_blue",
        texture: 0x3AB3DA,
        firework: 0x6689D3,
    },
    DyeColor {
        name: "yellow",
        texture: 0xFED83D,
        firework: 0xDECF2A,
    },
    DyeColor {
        name: "lime",
        texture: 0x80C71F,
        firework: 0x41CD34,
    },
    DyeColor {
        name: "pink",
        texture: 0xF38BAA,
        firework: 0xD88198,
    },
    DyeColor {
        name: "gray",
        texture: 0x474F52,
        firework: 0x434343,
    },
    DyeColor {
        name: "light_gray",
        texture: 0x9D9D97,
        firework: 0xABABAB,
    },
    DyeColor {
        name: "cyan",
        texture: 0x169C9C,
        firework: 0x287697,
    },
    DyeColor {
        name: "purple",
        texture: 0x8932B8,
        firework: 0x7B2FBE,
    },
    DyeColor {
        name: "blue",
        texture: 0x3C44AA,
        firework: 0x253192,
    },
    DyeColor {
        name: "brown",
        texture: 0x835432,
        firework: 0x51301A,
    },
    DyeColor {
        name: "green",
        texture: 0x5E7C16,
        firework: 0x3B511A,
    },
    DyeColor {
        name: "red",
        texture: 0xB02E26,
        firework: 0xB3312C,
    },
    DyeColor {
        name: "black",
        texture: 0x1D1D21,
        firework: 0x1E1B1B,
    },
];

const DYE_SUFFIX: &str = "_dye";
const NAMESPACE: &str = "minecraft:";

pub fn dye_color_id(name: &str) -> Option<usize> {
    let name = name.strip_prefix(NAMESPACE).unwrap_or(name);
    DYE_COLORS.iter().position(|color| color.name == name)
}

pub fn of_dye_item(stack: &ItemStack) -> Option<usize> {
    let name = stack.item.kind()?.name();
    let color = name.strip_prefix(NAMESPACE)?.strip_suffix(DYE_SUFFIX)?;
    dye_color_id(color)
}
