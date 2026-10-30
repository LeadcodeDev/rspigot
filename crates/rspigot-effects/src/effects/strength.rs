use crate::{AttributeOperation, AttributeTemplate, EffectCategory, EffectId, MobEffect};

pub struct Strength;

impl MobEffect for Strength {
    const NAME: &'static str = "minecraft:strength";
    const TRANSLATION_KEY: &'static str = "effect.minecraft.strength";
    const ID: EffectId = EffectId(4);
    const CATEGORY: EffectCategory = EffectCategory::Beneficial;
    const COLOR: u32 = 0xFFC700;
    const INSTANTANEOUS: bool = false;
    const MODIFIERS: &'static [AttributeTemplate] = &[AttributeTemplate { attribute: "minecraft:attack_damage", modifier: "minecraft:effect.strength", amount: 3.0, operation: AttributeOperation::AddValue }];
    type Behavior = crate::family::Plain;
}
