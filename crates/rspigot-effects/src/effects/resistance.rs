use crate::{AttributeTemplate, EffectCategory, EffectId, MobEffect};

pub struct Resistance;

impl MobEffect for Resistance {
    const NAME: &'static str = "minecraft:resistance";
    const TRANSLATION_KEY: &'static str = "effect.minecraft.resistance";
    const ID: EffectId = EffectId(10);
    const CATEGORY: EffectCategory = EffectCategory::Beneficial;
    const COLOR: u32 = 0x9146F0;
    const INSTANTANEOUS: bool = false;
    const MODIFIERS: &'static [AttributeTemplate] = &[];
    type Behavior = crate::family::Plain;
}
