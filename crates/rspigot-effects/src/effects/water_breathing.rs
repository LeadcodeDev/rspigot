use crate::{AttributeTemplate, EffectCategory, EffectId, MobEffect};

pub struct WaterBreathing;

impl MobEffect for WaterBreathing {
    const NAME: &'static str = "minecraft:water_breathing";
    const TRANSLATION_KEY: &'static str = "effect.minecraft.water_breathing";
    const ID: EffectId = EffectId(12);
    const CATEGORY: EffectCategory = EffectCategory::Beneficial;
    const COLOR: u32 = 0x98DAC0;
    const INSTANTANEOUS: bool = false;
    const MODIFIERS: &'static [AttributeTemplate] = &[];
    type Behavior = crate::family::Plain;
}
