use crate::{AttributeTemplate, EffectCategory, EffectId, MobEffect};

pub struct Nausea;

impl MobEffect for Nausea {
    const NAME: &'static str = "minecraft:nausea";
    const TRANSLATION_KEY: &'static str = "effect.minecraft.nausea";
    const ID: EffectId = EffectId(8);
    const CATEGORY: EffectCategory = EffectCategory::Harmful;
    const COLOR: u32 = 0x551D4A;
    const INSTANTANEOUS: bool = false;
    const MODIFIERS: &'static [AttributeTemplate] = &[];
    type Behavior = crate::family::Plain;
}
