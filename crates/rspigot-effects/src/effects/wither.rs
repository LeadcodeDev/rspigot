use crate::{AttributeTemplate, EffectCategory, EffectId, MobEffect};

pub struct Wither;

impl MobEffect for Wither {
    const NAME: &'static str = "minecraft:wither";
    const TRANSLATION_KEY: &'static str = "effect.minecraft.wither";
    const ID: EffectId = EffectId(19);
    const CATEGORY: EffectCategory = EffectCategory::Harmful;
    const COLOR: u32 = 0x736156;
    const INSTANTANEOUS: bool = false;
    const MODIFIERS: &'static [AttributeTemplate] = &[];
    type Behavior = crate::family::Interval<40>;
}
