use crate::{AttributeTemplate, EffectCategory, EffectId, MobEffect};

pub struct NightVision;

impl MobEffect for NightVision {
    const NAME: &'static str = "minecraft:night_vision";
    const TRANSLATION_KEY: &'static str = "effect.minecraft.night_vision";
    const ID: EffectId = EffectId(15);
    const CATEGORY: EffectCategory = EffectCategory::Beneficial;
    const COLOR: u32 = 0xC2FF66;
    const INSTANTANEOUS: bool = false;
    const MODIFIERS: &'static [AttributeTemplate] = &[];
    type Behavior = crate::family::Plain;
}
