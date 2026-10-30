use crate::{AttributeTemplate, EffectCategory, EffectId, MobEffect};

pub struct Glowing;

impl MobEffect for Glowing {
    const NAME: &'static str = "minecraft:glowing";
    const TRANSLATION_KEY: &'static str = "effect.minecraft.glowing";
    const ID: EffectId = EffectId(23);
    const CATEGORY: EffectCategory = EffectCategory::Neutral;
    const COLOR: u32 = 0x94A061;
    const INSTANTANEOUS: bool = false;
    const MODIFIERS: &'static [AttributeTemplate] = &[];
    type Behavior = crate::family::Plain;
}
