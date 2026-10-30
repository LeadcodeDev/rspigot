use crate::{AttributeOperation, AttributeTemplate, EffectCategory, EffectId, MobEffect};

pub struct Weakness;

impl MobEffect for Weakness {
    const NAME: &'static str = "minecraft:weakness";
    const TRANSLATION_KEY: &'static str = "effect.minecraft.weakness";
    const ID: EffectId = EffectId(17);
    const CATEGORY: EffectCategory = EffectCategory::Harmful;
    const COLOR: u32 = 0x484D48;
    const INSTANTANEOUS: bool = false;
    const MODIFIERS: &'static [AttributeTemplate] = &[AttributeTemplate { attribute: "minecraft:attack_damage", modifier: "minecraft:effect.weakness", amount: -4.0, operation: AttributeOperation::AddValue }];
    type Behavior = crate::family::Plain;
}
