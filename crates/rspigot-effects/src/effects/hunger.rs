use crate::{AttributeTemplate, EffectCategory, EffectId, MobEffect};

pub struct Hunger;

impl MobEffect for Hunger {
    const NAME: &'static str = "minecraft:hunger";
    const TRANSLATION_KEY: &'static str = "effect.minecraft.hunger";
    const ID: EffectId = EffectId(16);
    const CATEGORY: EffectCategory = EffectCategory::Harmful;
    const COLOR: u32 = 0x587653;
    const INSTANTANEOUS: bool = false;
    const MODIFIERS: &'static [AttributeTemplate] = &[];
    type Behavior = crate::family::EveryTick;
}
