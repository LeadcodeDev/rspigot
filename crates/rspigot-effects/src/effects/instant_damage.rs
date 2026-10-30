use crate::{AttributeTemplate, EffectCategory, EffectId, MobEffect};

pub struct InstantDamage;

impl MobEffect for InstantDamage {
    const NAME: &'static str = "minecraft:instant_damage";
    const TRANSLATION_KEY: &'static str = "effect.minecraft.instant_damage";
    const ID: EffectId = EffectId(6);
    const CATEGORY: EffectCategory = EffectCategory::Harmful;
    const COLOR: u32 = 0xA9656A;
    const INSTANTANEOUS: bool = true;
    const MODIFIERS: &'static [AttributeTemplate] = &[];
    type Behavior = crate::family::Instantaneous;
}
