use crate::{AttributeTemplate, EffectCategory, EffectId, MobEffect};

pub struct Poison;

impl MobEffect for Poison {
    const NAME: &'static str = "minecraft:poison";
    const TRANSLATION_KEY: &'static str = "effect.minecraft.poison";
    const ID: EffectId = EffectId(18);
    const CATEGORY: EffectCategory = EffectCategory::Harmful;
    const COLOR: u32 = 0x87A363;
    const INSTANTANEOUS: bool = false;
    const MODIFIERS: &'static [AttributeTemplate] = &[];
    type Behavior = crate::family::Interval<25>;
}
