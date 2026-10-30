use crate::{AttributeTemplate, EffectCategory, EffectId, MobEffect};

pub struct Blindness;

impl MobEffect for Blindness {
    const NAME: &'static str = "minecraft:blindness";
    const TRANSLATION_KEY: &'static str = "effect.minecraft.blindness";
    const ID: EffectId = EffectId(14);
    const CATEGORY: EffectCategory = EffectCategory::Harmful;
    const COLOR: u32 = 0x1F1F23;
    const INSTANTANEOUS: bool = false;
    const MODIFIERS: &'static [AttributeTemplate] = &[];
    type Behavior = crate::family::Plain;
}
