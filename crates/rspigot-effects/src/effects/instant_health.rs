use crate::{AttributeTemplate, EffectCategory, EffectId, MobEffect};

pub struct InstantHealth;

impl MobEffect for InstantHealth {
    const NAME: &'static str = "minecraft:instant_health";
    const TRANSLATION_KEY: &'static str = "effect.minecraft.instant_health";
    const ID: EffectId = EffectId(5);
    const CATEGORY: EffectCategory = EffectCategory::Beneficial;
    const COLOR: u32 = 0xF82423;
    const INSTANTANEOUS: bool = true;
    const MODIFIERS: &'static [AttributeTemplate] = &[];
    type Behavior = crate::family::Instantaneous;
}
