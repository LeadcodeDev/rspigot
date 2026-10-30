use crate::{AttributeTemplate, EffectCategory, EffectId, MobEffect};

pub struct SlowFalling;

impl MobEffect for SlowFalling {
    const NAME: &'static str = "minecraft:slow_falling";
    const TRANSLATION_KEY: &'static str = "effect.minecraft.slow_falling";
    const ID: EffectId = EffectId(27);
    const CATEGORY: EffectCategory = EffectCategory::Beneficial;
    const COLOR: u32 = 0xF3CFB9;
    const INSTANTANEOUS: bool = false;
    const MODIFIERS: &'static [AttributeTemplate] = &[];
    type Behavior = crate::family::Plain;
}
