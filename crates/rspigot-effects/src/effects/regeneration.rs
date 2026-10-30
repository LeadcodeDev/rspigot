use crate::{AttributeTemplate, EffectCategory, EffectId, MobEffect};

pub struct Regeneration;

impl MobEffect for Regeneration {
    const NAME: &'static str = "minecraft:regeneration";
    const TRANSLATION_KEY: &'static str = "effect.minecraft.regeneration";
    const ID: EffectId = EffectId(9);
    const CATEGORY: EffectCategory = EffectCategory::Beneficial;
    const COLOR: u32 = 0xCD5CAB;
    const INSTANTANEOUS: bool = false;
    const MODIFIERS: &'static [AttributeTemplate] = &[];
    type Behavior = crate::family::Interval<50>;
}
