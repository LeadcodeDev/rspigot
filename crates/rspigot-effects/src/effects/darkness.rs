use crate::{AttributeTemplate, EffectCategory, EffectId, MobEffect};

pub struct Darkness;

impl MobEffect for Darkness {
    const NAME: &'static str = "minecraft:darkness";
    const TRANSLATION_KEY: &'static str = "effect.minecraft.darkness";
    const ID: EffectId = EffectId(32);
    const CATEGORY: EffectCategory = EffectCategory::Harmful;
    const COLOR: u32 = 0x292721;
    const INSTANTANEOUS: bool = false;
    const MODIFIERS: &'static [AttributeTemplate] = &[];
    type Behavior = crate::family::Plain;
}
