use crate::{AttributeOperation, AttributeTemplate, EffectCategory, EffectId, MobEffect};

pub struct Unluck;

impl MobEffect for Unluck {
    const NAME: &'static str = "minecraft:unluck";
    const TRANSLATION_KEY: &'static str = "effect.minecraft.unluck";
    const ID: EffectId = EffectId(26);
    const CATEGORY: EffectCategory = EffectCategory::Harmful;
    const COLOR: u32 = 0xC0A44D;
    const INSTANTANEOUS: bool = false;
    const MODIFIERS: &'static [AttributeTemplate] = &[AttributeTemplate { attribute: "minecraft:luck", modifier: "minecraft:effect.unluck", amount: -1.0, operation: AttributeOperation::AddValue }];
    type Behavior = crate::family::Plain;
}
