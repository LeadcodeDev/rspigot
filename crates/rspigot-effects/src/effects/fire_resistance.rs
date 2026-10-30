use crate::{AttributeTemplate, EffectCategory, EffectId, MobEffect};

pub struct FireResistance;

impl MobEffect for FireResistance {
    const NAME: &'static str = "minecraft:fire_resistance";
    const TRANSLATION_KEY: &'static str = "effect.minecraft.fire_resistance";
    const ID: EffectId = EffectId(11);
    const CATEGORY: EffectCategory = EffectCategory::Beneficial;
    const COLOR: u32 = 0xFF9900;
    const INSTANTANEOUS: bool = false;
    const MODIFIERS: &'static [AttributeTemplate] = &[];
    type Behavior = crate::family::Plain;
}
