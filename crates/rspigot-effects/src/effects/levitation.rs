use crate::{AttributeTemplate, EffectCategory, EffectId, MobEffect};

pub struct Levitation;

impl MobEffect for Levitation {
    const NAME: &'static str = "minecraft:levitation";
    const TRANSLATION_KEY: &'static str = "effect.minecraft.levitation";
    const ID: EffectId = EffectId(24);
    const CATEGORY: EffectCategory = EffectCategory::Harmful;
    const COLOR: u32 = 0xCEFFFF;
    const INSTANTANEOUS: bool = false;
    const MODIFIERS: &'static [AttributeTemplate] = &[];
    type Behavior = crate::family::Plain;
}
