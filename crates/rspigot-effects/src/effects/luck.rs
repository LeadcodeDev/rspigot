use crate::{AttributeOperation, AttributeTemplate, EffectCategory, EffectId, MobEffect};

pub struct Luck;

impl MobEffect for Luck {
    const NAME: &'static str = "minecraft:luck";
    const TRANSLATION_KEY: &'static str = "effect.minecraft.luck";
    const ID: EffectId = EffectId(25);
    const CATEGORY: EffectCategory = EffectCategory::Beneficial;
    const COLOR: u32 = 0x59C106;
    const INSTANTANEOUS: bool = false;
    const MODIFIERS: &'static [AttributeTemplate] = &[AttributeTemplate { attribute: "minecraft:luck", modifier: "minecraft:effect.luck", amount: 1.0, operation: AttributeOperation::AddValue }];
    type Behavior = crate::family::Plain;
}
