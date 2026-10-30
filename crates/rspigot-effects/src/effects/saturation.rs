use crate::{AttributeTemplate, EffectCategory, EffectId, MobEffect};

pub struct Saturation;

impl MobEffect for Saturation {
    const NAME: &'static str = "minecraft:saturation";
    const TRANSLATION_KEY: &'static str = "effect.minecraft.saturation";
    const ID: EffectId = EffectId(22);
    const CATEGORY: EffectCategory = EffectCategory::Beneficial;
    const COLOR: u32 = 0xF82423;
    const INSTANTANEOUS: bool = true;
    const MODIFIERS: &'static [AttributeTemplate] = &[];
    type Behavior = crate::family::Instantaneous;
}
